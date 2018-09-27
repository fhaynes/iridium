use assembler::Token;
use instruction::Opcode;
use nom::*;
use nom::types::CompleteStr;

named!(pub opcode<CompleteStr, Token>,
  do_parse!(
      opcode: alpha1 >>
      (
        {
            Token::Op{code: Opcode::from(opcode)}
        }
      )
  )
);

mod tests {
    #![allow(unused_imports)]

    use assembler::Token;
    use instruction::Opcode;
    use nom::types::CompleteStr;
    use super::opcode;

    #[test]
    fn test_opcode() {
        let result = opcode(CompleteStr("load"));
        assert_eq!(result.is_ok(), true);
        let (rest, token) = result.unwrap();
        assert_eq!(token, Token::Op { code: Opcode::LOAD });
        assert_eq!(rest, CompleteStr(""));
        let result = opcode(CompleteStr("aold"));
        let (_, token) = result.unwrap();
        assert_eq!(token, Token::Op { code: Opcode::IGL });
    }
}
