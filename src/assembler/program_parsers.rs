use assembler::directive_parsers::directive;
use assembler::instruction_parsers::{instruction, AssemblerInstruction};
use assembler::SymbolTable;
use nom::types::CompleteStr;

#[derive(Debug, PartialEq)]
pub struct Program {
    pub instructions: Vec<AssemblerInstruction>,
}

impl Program {
    pub fn to_bytes(&self, symbols: &SymbolTable) -> Vec<u8> {
        let mut program = vec![];
        for instruction in &self.instructions {
            program.append(&mut instruction.to_bytes(symbols));
        }
        program
    }
}

named!(pub program<CompleteStr, Program>,
    do_parse!(
        instructions: many1!(alt!(instruction | directive)) >>
        (
            Program {
                instructions
            }
        )
    )
);

#[cfg(test)]
mod tests {
    use super::*;
    use assembler::Assembler;
    use vm::VM;

    #[test]
    fn test_parse_program() {
        let result = program(CompleteStr("load $0 #100\n"));
        assert_eq!(result.is_ok(), true);
        let (leftover, p) = result.unwrap();
        assert_eq!(leftover, CompleteStr(""));
        assert_eq!(1, p.instructions.len());
        // TODO: Figure out an ergonomic way to test the AssemblerInstruction returned
    }

    #[test]
    fn test_program_to_bytes() {
        let result = program(CompleteStr("load $0 #100\n"));
        assert_eq!(result.is_ok(), true);
        let (_, program) = result.unwrap();
        let symbols = SymbolTable::new();
        let bytecode = program.to_bytes(&symbols);
        assert_eq!(bytecode.len(), 4);
    }

    #[test]
    fn test_complete_program() {
        let test_program = CompleteStr(".data\nhello: .asciiz 'Hello everyone!'\n.code\nhlt");
        let result = program(test_program);
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_parse_load_greater_than_i16() {
        let mut test_assembler = Assembler::new();
        let mut test_vm = VM::new();
        let result = test_assembler.assemble(".data\n.code\nload $0 #-50000");
        assert!(result.is_ok());
        let result = result.unwrap();
        test_vm.program = result;
        test_vm.run();
    }

    #[test]
    fn test_parse_cloop() {
        let mut test_assembler = Assembler::new();
        let mut test_vm = VM::new();
        let result = test_assembler.assemble(".data\n.code\ncloop #10");
        assert!(result.is_ok());
        let result = result.unwrap();
        test_vm.program = result;
        test_vm.run();
    }
}
