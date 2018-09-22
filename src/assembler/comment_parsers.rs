use nom::types::CompleteStr;
use assembler::Token;

/// Looks for a comment start
/// Examples:
named!(pub comment<CompleteStr, Token>,
    ws!(
        do_parse!(
            tag!(";") >>
            take_until!("\n") >>
            (
                Token::Comment
            )
        )
    )
);
