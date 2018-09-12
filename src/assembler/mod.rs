pub mod instruction_parsers;
pub mod opcode_parsers;
pub mod operand_parsers;
pub mod program_parsers;
pub mod register_parsers;
pub mod label_parsers;
pub mod directive_parsers;
pub mod assembler_errors;
pub mod symbols;

use nom::types::CompleteStr;

use instruction::Opcode;
use assembler::program_parsers::{program, Program};
use assembler::instruction_parsers::{AssemblerInstruction};
use assembler::assembler_errors::AssemblerError;
use assembler::symbols::{Symbol, SymbolTable, SymbolType};

/// Magic number that begins every bytecode file prefix. These spell out EPIE in ASCII, if you were wondering.
pub const PIE_HEADER_PREFIX: [u8; 4] = [45, 50, 49, 45];

/// Constant that determines how long the header is. There are 60 zeros left after the prefix, for later
/// usage if needed.
pub const PIE_HEADER_LENGTH: usize = 64;

#[derive(Debug, PartialEq)]
pub enum Token {
    Op { code: Opcode },
    Register { reg_num: u8 },
    IntegerOperand { value: i32 },
    LabelDeclaration { name: String },
    LabelUsage { name: String },
    Directive { name: String },
    IrString { name: String }
}


#[derive(Debug, Default)]
pub struct Assembler {
    /// Tracks which phase the assember is in
    phase: AssemblerPhase,
    /// Symbol table for constants and variables
    pub symbols: SymbolTable,
    /// The read-only data section constants are put in
    pub ro: Vec<u8>,
    /// The compiled bytecode generated from the assembly instructions
    pub bytecode: Vec<u8>,
    /// Tracks the current offset of the read-only section
    ro_offset: u32,
    /// A list of all the sections we've seen in the code
    sections: Vec<AssemblerSection>,
    /// The current section the assembler is in
    current_section: Option<AssemblerSection>,
    /// The current instruction the assembler is converting to bytecode
    current_instruction: u32,
    /// Any errors we find along the way. At the end, we'll present them to the user.
    errors: Vec<AssemblerError>
}

impl Assembler {
    pub fn new() -> Assembler {
        Assembler {
            current_instruction: 0,
            ro_offset: 0,
            ro: vec![],
            bytecode: vec![],
            sections: vec![],
            errors: vec![],
            phase: AssemblerPhase::First,
            symbols: SymbolTable::new(),
            current_section: None
        }
    }

    pub fn assemble(&mut self, raw: &str) -> Result<Vec<u8>, Vec<AssemblerError>> {
        match program(CompleteStr(raw)) {
            Ok((_remainder, program)) => {
                // First get the header so we can smush it into the bytecode letter
                let mut assembled_program = self.write_pie_header();

                // Start processing the AssembledInstructions
                self.process_first_phase(&program);
                if !self.errors.is_empty() {
                    // TODO: Can we avoid a clone here?
                    return Err(self.errors.clone());
                };

                // Make sure that we have at least one data section and one code section
                if self.sections.len() != 2 {
                    // TODO: Detail out which one(s) are missing
                    println!("Did not find at least two sections.");
                    self.errors.push(AssemblerError::InsufficientSections);
                    // TODO: Can we avoid a clone here?
                    return Err(self.errors.clone());
                }
                // Run the second pass, which translates opcodes and associated operands into the bytecode
                let mut body = self.process_second_phase(&program);

                // Merge the header with the populated body vector
                assembled_program.append(&mut body);
                Ok(assembled_program)
            },
            // If there were parsing errors, bad syntax, etc, this arm is run
            Err(e) => {
                println!("There was an error parsing the code: {:?}", e);
                Err(vec![AssemblerError::ParseError{ error: e.to_string() }])
            }
        }
    }

    /// Runs the first pass of the two-pass assembling process. It looks for labels and puts them in the symbol table
    fn process_first_phase(&mut self, p: &Program) {
        // Iterate over every instruction, even though in the first phase we only care about labels and directives
        for i in &p.instructions {
            if i.is_label() {
                // TODO: Factor this out into another function? Put it in `process_label_declaration` maybe?
                if self.current_section.is_some() {
                    // If we have hit a segment header already (e.g., `.code`) then we are ok
                    self.process_label_declaration(&i);
                } else {
                    // If we have *not* hit a segment header yet, then we have a label outside of a segment, which is not allowed
                    self.errors.push(AssemblerError::NoSegmentDeclarationFound{instruction: self.current_instruction});
                }
            }

            if i.is_directive() {
                self.process_directive(i);
            }
            // This is used to keep track of which instruction we hit an error on
            self.current_instruction += 1;
        }
        self.phase = AssemblerPhase::Second;
    }

    /// Runs the second pass of the assembler
    fn process_second_phase(&mut self, p: &Program) -> Vec<u8> {
        self.current_instruction = 0;
        // We're going to put the bytecode meant to be executed in a separate Vec so we can do some post-processing and then merge it with the header and read-only sections
        let mut program = vec![];
        // Same as in first pass, except in the second pass we care about opcodes and directives
        for i in &p.instructions {
            if i.is_opcode() {
                // Opcodes know how to properly transform themselves into 32-bits, so we can just call `to_bytes` and append to our program
                let mut bytes = i.to_bytes(&self.symbols);
                program.append(&mut bytes);
            }
            if i.is_directive() {
                // In this phase, we can have directives but of different types than we care about in the first pass. The Directive itself can check which pass the Assembler
                // is in and decide what to do about it
                self.process_directive(i);
            }
            self.current_instruction += 1
        }
        program
    }

    fn process_label_declaration(&mut self, i: &AssemblerInstruction) {
        // Check if the label is None or String
        let name = match i.get_label_name() {
            Some(name) => { name },
            None => {
                self.errors.push(AssemblerError::StringConstantDeclaredWithoutLabel{instruction: self.current_instruction});
                return;
            }
        };

        // Check if label is already in use (has an entry in the symbol table)
        // TODO: Is there a cleaner way to do this?
        if self.symbols.has_symbol(&name) {
            self.errors.push(AssemblerError::SymbolAlreadyDeclared);
            return;
        }

        // If we make it here, it isn't a symbol we've seen before, so stick it in the table
        let symbol = Symbol::new(name, SymbolType::Label);
        self.symbols.add_symbol(symbol);
    }

    fn process_directive(&mut self, i: &AssemblerInstruction) {
        // First let's make sure we have a parseable name
        let directive_name = match i.get_directive_name() {
            Some(name) => {
                name
            },
            None => {
                println!("Directive has an invalid name: {:?}", i);
                return;
            }
        };

        // Now check if there were any operands.
        if i.has_operands() {
            // If it _does_ have operands, we need to figure out which directive it was
            match directive_name.as_ref() {
                "asciiz" => {
                    self.handle_asciiz(i);
                }
                _ => {
                    self.errors.push(AssemblerError::UnknownDirectiveFound{ directive: directive_name.clone() });
                    return;
                }
            }
        } else {
            self.process_section_header(&directive_name);
        }
    }

    /// Handles a declaration of a null-terminated string:
    /// hello: .asciiz 'Hello!'
    fn handle_asciiz(&mut self, i: &AssemblerInstruction) {
        // Being a constant declaration, this is only meaningful in the first pass
        if self.phase != AssemblerPhase::First { return; }

        // In this case, operand1 will have the entire string we need to read in to RO memory
        match i.get_string_constant() {
            Some(s) => {
                match i.get_label_name() {
                    Some(name) => { self.symbols.set_symbol_offset(&name, self.ro_offset); }
                    None => {
                        // This would be someone typing:
                        // .asciiz 'Hello'
                        println!("Found a string constant with no associated label!");
                        return;
                    }
                };
                // We'll read the string into the read-only section byte-by-byte
                for byte in s.as_bytes() {
                    self.ro.push(*byte);
                    self.ro_offset += 1;
                }
                // This is the null termination bit we are using to indicate a string has ended
                self.ro.push(0);
                self.ro_offset += 1;
            }
            None => {
                // This just means someone typed `.asciiz` for some reason
                println!("String constant following an .asciiz was empty");
            }
        }
    }

    /// Handles a declaration of a section header, such as:
    /// .code
    fn process_section_header(&mut self, header_name: &str) {
        let new_section: AssemblerSection = header_name.into();
        // Only specific section names are allowed
        if new_section == AssemblerSection::Unknown {
            println!("Found an section header that is unknown: {:#?}", header_name);
            return;
        }
        // TODO: Check if we really need to keep a list of all sections seen
        self.sections.push(new_section.clone());
        self.current_section = Some(new_section);
    }

    /// Convenience function to write the executable header
    fn write_pie_header(&self) -> Vec<u8> {
        let mut header = vec![];
        for byte in &PIE_HEADER_PREFIX {
            header.push(byte.clone());
        }
        while header.len() < PIE_HEADER_LENGTH {
            header.push(0 as u8);
        }
        header
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssemblerPhase {
    First,
    Second
}

impl Default for AssemblerPhase {
    fn default() -> Self {
        AssemblerPhase::First
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum AssemblerSection {
    Data{starting_instruction: Option<u32>},
    Code{starting_instruction: Option<u32>},
    Unknown
}

impl Default for AssemblerSection {
    fn default() -> Self {
        AssemblerSection::Unknown
    }
}

impl<'a> From<&'a str> for AssemblerSection {
    fn from(name: &str) -> AssemblerSection {
        match name {
            "data" => {
                AssemblerSection::Data{starting_instruction: None}
            }
            "code" => {
                AssemblerSection::Code{starting_instruction: None}
            }
            _ => {
                AssemblerSection::Unknown
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vm::VM;

    #[test]
    /// Tests assembly a small but correct program
    fn test_assemble_program() {
        let mut asm = Assembler::new();
        let test_string = ".data\n.code\nload $0 #100\nload $1 #1\nload $2 #0\ntest: inc $0\nneq $0 $2\njmpe @test\nhlt";
        let program = asm.assemble(test_string).unwrap();
        let mut vm = VM::new();
        assert_eq!(program.len(), 92);
        vm.add_bytes(program);
        assert_eq!(vm.program.len(), 92);
    }

    #[test]
    /// Tests that we can add things to the symbol table
    fn test_symbol_table() {
        let mut sym = SymbolTable::new();
        let new_symbol = Symbol::new_with_offset("test".to_string(), SymbolType::Label, 12);
        sym.add_symbol(new_symbol);
        assert_eq!(sym.symbols.len(), 1);
        let v = sym.symbol_value("test");
        assert_eq!(true, v.is_some());
        let v = v.unwrap();
        assert_eq!(v, 12);
        let v = sym.symbol_value("does_not_exist");
        assert_eq!(v.is_some(), false);
    }

    #[test]
    /// Simple test of data that goes into the read only section
    fn test_ro_data() {
        let mut asm = Assembler::new();
        let test_string = ".data\ntest: .asciiz 'This is a test'\n.code\n";
        let program = asm.assemble(test_string);
        assert_eq!(program.is_ok(), true);
    }

    #[test]
    /// This tests that a section name that isn't `code` or `data` throws an error
    fn test_bad_ro_data() {
        let mut asm = Assembler::new();
        let test_string = ".code\ntest: .asciiz 'This is a test'\n.wrong\n";
        let program = asm.assemble(test_string);
        assert_eq!(program.is_ok(), false);
    }

    #[test]
    /// Tests that code which does not declare a segment first does not work
    fn test_first_phase_no_segment() {
        let mut asm = Assembler::new();
        let test_string = "hello: .asciiz 'Fail'";
        let result = program(CompleteStr(test_string));
        assert_eq!(result.is_ok(), true);
        let (_, p) = result.unwrap();
        asm.process_first_phase(&p);
        assert_eq!(asm.errors.len(), 1);
    }

    #[test]
    /// Tests that code inside a proper segment works
    fn test_first_phase_inside_segment() {
        let mut asm = Assembler::new();
        let test_string = ".data\ntest: .asciiz 'Hello'";
        let result = program(CompleteStr(test_string));
        assert_eq!(result.is_ok(), true);
        let (_, p) = result.unwrap();
        asm.process_first_phase(&p);
        assert_eq!(asm.errors.len(), 0);
    }
}
