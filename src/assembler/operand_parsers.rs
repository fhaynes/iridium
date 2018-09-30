use assembler::label_parsers::label_usage;
use assembler::register_parsers::register;
use assembler::Token;
use nom::digit;
use nom::types::CompleteStr;

/// Parser for all numbers, which have to be prefaced with `#` in our assembly language:
/// #100
named!(integer_operand<CompleteStr, Token>,
    ws!(
        do_parse!(
            tag!("#") >>
            sign: opt!(tag!("-")) >>
            reg_num: digit >>
            (
                {
                    let mut tmp = String::from("");
                    if sign.is_some() {
                        tmp.push_str("-");
                    }
                    tmp.push_str(&reg_num.to_string());
                    let converted = tmp.parse::<i32>().unwrap();
                    Token::IntegerOperand{value: converted}
                }
            )
        )
    )
);

/// Parser for all floats, which have to be prefaced with `#` in our assembly language:
/// #100.0
named!(float_operand<CompleteStr, Token>,
    ws!(
        do_parse!(
            tag!("#") >>
            sign: opt!(tag!("-")) >>
            left_nums: digit >>
            tag!(".") >>
            right_nums: digit >>
            (
                {
                    let mut tmp = String::from("");
                    if sign.is_some() {
                        tmp.push_str("-");
                    }
                    tmp.push_str(&left_nums.to_string());
                    tmp.push_str(".");
                    tmp.push_str(&right_nums.to_string());
                    Token::FloatOperand{value: tmp.parse::<f64>().unwrap()}
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

    use super::{float_operand, integer_operand, irstring};
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
        let test = vec!["#100.3", "#-100.3", "#1.0", "#0.0"];
        for i in &test {
            assert_eq!(float_operand(CompleteStr(i)).is_ok(), true);
        }
    }
}
