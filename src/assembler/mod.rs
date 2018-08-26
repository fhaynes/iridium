pub mod instruction_parsers;
pub mod opcode_parsers;
pub mod operand_parsers;
pub mod program_parsers;
pub mod register_parsers;
pub mod label_parsers;
pub mod directive_parsers;

use nom::types::CompleteStr;

use instruction::Opcode;
use assembler::program_parsers::{program, Program};

#[derive(Debug, PartialEq)]
pub enum Token {
    Op { code: Opcode },
    Register { reg_num: u8 },
    IntegerOperand { value: i32 },
    LabelDeclaration { name: String },
    LabelUsage { name: String },
    Directive { name: String }
}


#[derive(Debug)]
pub struct Assembler {
    phase: AssemblerPhase,
    labels: SymbolTable
}

impl Assembler {
    pub fn new() -> Assembler {
        Assembler {
            phase: AssemblerPhase::First,
            labels: SymbolTable::new(),
        }
    }

    pub fn assemble(&mut self, raw: &str) -> Option<Vec<u8>> {
        let result = program(CompleteStr(raw));
        if !result.is_ok() { return None; }

        let (_, p) = result.unwrap();
        self.process_first_phase(&p);
        Some(self.process_second_phase(&p))
    }

    fn process_first_phase(&mut self, p: &Program) {
        self.extract_labels(p);
        self.phase = AssemblerPhase::Second;
    }

    fn process_second_phase(&mut self, p: &Program) -> Vec<u8> {
        let mut program = vec![];
        for i in &p.instructions {
            let mut bytes = i.to_bytes(&self.labels);
            program.append(&mut bytes);
        }
        println!("Program is: {:?}", program);
        program
    }

    fn extract_labels(&mut self, p: &Program) {
        let mut c = 0;
        for i in &p.instructions {
            if i.is_label() {
                match i.label_name() {
                    Some(name) => {
                        let symbol = Symbol::new(name, SymbolType::Label, c);
                        self.labels.add_symbol(symbol);
                    },
                    None => {}
                };
            }
            c += 4;
        }
    }
}

#[derive(Debug)]
pub enum AssemblerPhase {
    First,
    Second
}

#[derive(Debug)]
pub enum AssemblerSection {
    Data,
    Code
}

#[derive(Debug)]
pub struct Symbol {
    name: String,
    offset: u32,
    symbol_type: SymbolType,
}

impl Symbol {
    pub fn new(name: String, symbol_type: SymbolType, offset: u32) -> Symbol {
        Symbol{
            name,
            symbol_type,
            offset
        }
    }
}

#[derive(Debug)]
pub enum SymbolType {
    Label,
    Integer,
}

/// Holds all of the symbols
#[derive(Debug)]
pub struct SymbolTable {
    symbols: Vec<Symbol>
}

impl SymbolTable {
    pub fn new() -> SymbolTable {
        SymbolTable{
            symbols: vec![]
        }
    }

    pub fn add_symbol(&mut self, s: Symbol) {
        self.symbols.push(s);
    }

    pub fn symbol_value(&self, s: &str) -> Option<u32> {
        for symbol in &self.symbols {
            if symbol.name == s {
                return Some(symbol.offset);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assemble_program() {
        let mut asm = Assembler::new();
        let test_string = "test: load $0 #100\n";
        let program = asm.assemble(test_string);
        println!("{:?}", program);
        println!("{:?}", asm.labels);
    }
}
