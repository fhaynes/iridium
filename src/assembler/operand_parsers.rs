use nom::digit;
use nom::types::CompleteStr;

use assembler::label_parsers::label_usage;
use assembler::register_parsers::register;
use assembler::Token;

/// Parser for all numbers, which have to be prefaced with `#` in our assembly language:
/// #100
named!(integer_operand<CompleteStr, Token>,
    ws!(
        do_parse!(
            tag!("#") >>
            reg_num: digit >>
            (
                Token::IntegerOperand{value: reg_num.parse::<i32>().unwrap()}
            )
        )
    )
);

named!(float_operand<CompleteStr, Token>,
    ws!(
        do_parse!(
            tag!("#") >>
            reg_num: digit >>
            tag!(".") >>
            post_num: digit >>
            (
                {

                    Token::FloatOperand{value: (reg_num.to_string() + "." + &post_num).parse::<f64>().unwrap()}
                }
            )
        )
    )
);

named!(irstring<CompleteStr, Token>,
    do_parse!(
        tag!("'") >>
        content: take_until!("'") >>
        tag!("'") >>
        (
            Token::IrString{ name: content.to_string() }
        )
    )
);

named!(pub operand<CompleteStr, Token>,
    alt!(
        integer_operand |
        float_operand |
        label_usage |
        register |
        irstring
    )
);

mod tests {
    #![allow(unused_imports)]
    use super::{integer_operand, irstring, float_operand};
    use assembler::Token;
    use nom::types::CompleteStr;

    #[test]
    fn test_parse_integer_operand() {
        let result = integer_operand(CompleteStr("#10"));
        assert_eq!(result.is_ok(), true);
        let (rest, value) = result.unwrap();
        assert_eq!(rest, CompleteStr(""));
        assert_eq!(value, Token::IntegerOperand { value: 10 });

        let result = integer_operand(CompleteStr("10"));
        assert_eq!(result.is_ok(), false);
    }

    #[test]
    fn test_parse_string_operand() {
        let result = irstring(CompleteStr("'This is a test'"));
        assert_eq!(result.is_ok(), true);
    }

    #[test]
    fn test_parse_float_operand() {
        let result = float_operand(CompleteStr("#100.3"));
        assert_eq!(result.is_ok(), true);
        let (_, value) = result.unwrap();
        assert_eq!(value, Token::FloatOperand{value: 100.3});
    }
}
