use nom::alpha;
use nom::types::CompleteStr;

use assembler::Token;

named!(pub directive <CompleteStr, Token>,
    ws!(
        do_parse!(
            tag!(".") >>
            d: alpha >>
            (
                Token::Directive{
                  name: d.to_string(),
                }
            )
        )
    )
);

mod tests {
    #![allow(unused_imports)]
    use nom::types::CompleteStr;
    use super::directive;
    use assembler::Token;

    #[test]
    fn test_parser_directive() {
        let result = directive(CompleteStr(".data"));
        assert_eq!(result.is_ok(), true);
        let (_, directive) = result.unwrap();
        assert_eq!(directive, Token::Directive{name: "data".to_string() })
    }
}
