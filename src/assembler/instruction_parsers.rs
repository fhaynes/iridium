use std;

use nom::types::CompleteStr;
use nom::multispace;

use assembler::opcode_parsers::*;
use assembler::operand_parsers::operand;
use assembler::register_parsers::register;
use assembler::directive_parsers::directive;
use assembler::label_parsers::label_declaration;
use assembler::{Token, SymbolTable};
#[derive(Debug, PartialEq)]
pub struct AssemblerInstruction {
    opcode: Option<Token>,
    label: Option<Token>,
    directive: Option<Token>,
    operand1: Option<Token>,
    operand2: Option<Token>,
    operand3: Option<Token>,
}

impl AssemblerInstruction {
    pub fn to_bytes(&self, symbols: &SymbolTable) -> Vec<u8> {
        let mut results = vec![];
        match self.opcode {
            Some(ref token) => {
                match token {
                    Token::Op { code } => match code {
                        _ => {
                            results.push(*code as u8);
                        }
                    },
                    _ => {
                        println!("Non-opcode found in opcode field");
                        std::process::exit(1);
                    }
                }
            }
            None => {},
        };

        for operand in &[&self.operand1, &self.operand2, &self.operand3] {
            if let Some(token) = operand {
                AssemblerInstruction::extract_operand(token, &mut results, symbols)
            }
        }

        results
    }

    pub fn is_label(&self) -> bool {
        self.label.is_some()
    }

    pub fn is_opcode(&self) -> bool {
        self.opcode.is_some()
    }

    pub fn is_directive(&self) -> bool {
        self.directive.is_some()
    }

    pub fn label_name(&self) -> Option<String> {
        match &self.label {
            Some(l) => {
                match l {
                    Token::LabelDeclaration{name} => {
                        Some(name.clone())
                    }
                    _ => None
                }
            },
            None => {
                None
            }
        }
    }

    fn extract_operand(t: &Token, results: &mut Vec<u8>, symbols: &SymbolTable) {
        match t {
            Token::Register { reg_num } => {
                results.push(*reg_num);
            }
            Token::IntegerOperand { value } => {
                let converted = *value as u16;
                let byte1 = converted;
                let byte2 = converted >> 8;
                results.push(byte2 as u8);
                results.push(byte1 as u8);
            }
            Token::LabelUsage { name } => {
                match symbols.symbol_value(name) {
                    Some(value) => {
                        let byte1 = value;
                        let byte2 = value >> 8;
                        results.push(byte2 as u8);
                        results.push(byte1 as u8);
                    },
                    None => {}
                }
            }
            _ => {
                println!("Opcode found in operand field");
                std::process::exit(1);
            }
        };
    }
}

/// Handles instructions of the following form:
/// <opcode> <register> <operand>
named!(instruction_one<CompleteStr, AssemblerInstruction>,
    do_parse!(
        l: opt!(label_declaration) >>
        o: opcode >>
        r: register >>
        i: operand >>
        (
            AssemblerInstruction{
                opcode: Some(o),
                label: l,
                directive: None,
                operand1: Some(r),
                operand2: Some(i),
                operand3: None
            }
        )
    )
);

/// Handles instructions of the following form:
/// <opcode>
named!(instruction_two<CompleteStr, AssemblerInstruction>,
    do_parse!(
        l: opt!(label_declaration) >>
        o: opcode >>
        opt!(multispace) >>
        (
            AssemblerInstruction{
                opcode: Some(o),
                label: l,
                directive: None,
                operand1: None,
                operand2: None,
                operand3: None,
            }
        )
    )
);

/// Handles instructions of the following form:
/// <opcode> <register> <register> <register>
named!(instruction_three<CompleteStr, AssemblerInstruction>,
    do_parse!(
        l: opt!(label_declaration) >>
        o: opcode >>
        r1: register >>
        r2: register >>
        r3: register >>
        (
            AssemblerInstruction{
                opcode: Some(o),
                label: l,
                directive: None,
                operand1: Some(r1),
                operand2: Some(r2),
                operand3: Some(r3),
            }
        )
    )
);

/// Handles instructions of the following form:
/// <directive>
named!(instruction_four<CompleteStr, AssemblerInstruction>,
    do_parse!(
        d: directive >>
        (
            AssemblerInstruction{
                label: None,
                opcode: None,
                directive: Some(d),
                operand1: None,
                operand2: None,
                operand3: None
            }
        )
    )
);

/// Will try to parse out any of the Instruction forms
named!(pub instruction<CompleteStr, AssemblerInstruction>,
    do_parse!(
        ins: alt!(
            instruction_one |
            instruction_three |
            instruction_two |
            instruction_four
        ) >>
        (
            ins
        )
    )
);

#[cfg(test)]
mod tests {
    use super::*;
    use instruction::Opcode;

    #[test]
    fn test_parse_instruction_form_one() {
        let result = instruction_one(CompleteStr("load $0 #100\n"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: Some(Token::Op { code: Opcode::LOAD }),
                    label: None,
                    directive: None,
                    operand1: Some(Token::Register { reg_num: 0 }),
                    operand2: Some(Token::IntegerOperand { value: 100 }),
                    operand3: None
                }
            ))
        );
    }

    #[test]
    fn test_parse_instruction_form_one_with_label() {
        let result = instruction_one(CompleteStr("load $0 @test1\n"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: Some(Token::Op { code: Opcode::LOAD }),
                    label: None,
                    directive: None,
                    operand1: Some(Token::Register { reg_num: 0 }),
                    operand2: Some(Token::LabelUsage { name: "test1".to_string() }),
                    operand3: None
                }
            ))
        );
    }

    #[test]
    fn test_parse_instruction_form_two() {
        let result = instruction_two(CompleteStr("hlt\n"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: Some(Token::Op { code: Opcode::HLT }),
                    label: None,
                    directive: None,
                    operand1: None,
                    operand2: None,
                    operand3: None
                }
            ))
        );
    }

    #[test]
    fn test_parse_instruction_form_three() {
        let result = instruction_three(CompleteStr("add $0 $1 $2\n"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: Some(Token::Op { code: Opcode::ADD }),
                    label: None,
                    directive: None,
                    operand1: Some(Token::Register { reg_num: 0 }),
                    operand2: Some(Token::Register { reg_num: 1 }),
                    operand3: Some(Token::Register { reg_num: 2 }),
                }
            ))
        );
    }

    #[test]
    fn test_parse_instruction_form_four() {
        let result = instruction_four(CompleteStr(".data\n"));
        assert_eq!(
            result,
            Ok((
                CompleteStr(""),
                AssemblerInstruction {
                    opcode: None,
                    label: None,
                    directive: Some(Token::Directive { name: "data".to_string() }),
                    operand1: None,
                    operand2: None,
                    operand3: None
                }
            ))
        );
    }
}
