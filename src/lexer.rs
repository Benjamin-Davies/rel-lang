use std::fmt;

use chumsky::prelude::*;

use crate::{Span, Spanned};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token<'src> {
    Op(char),
    Ctrl(char),
    Ident(&'src str),
    Decl,
    Beg,
    End,
    While,
    Do,
    Od,
    Return,
    If,
    Then,
    Else,
    Fi,
}

pub fn lexer<'src>()
-> impl Parser<'src, &'src str, Vec<Spanned<Token<'src>>>, extra::Err<Rich<'src, char, Span>>> {
    let op = one_of("=+-|&^*").map(Token::Op);

    let ctrl = one_of("(),.").map(Token::Ctrl);

    let ident = text::ident().map(|s| match s {
        "DECL" => Token::Decl,
        "BEG" => Token::Beg,
        "END" => Token::End,
        "WHILE" => Token::While,
        "DO" => Token::Do,
        "OD" => Token::Od,
        "RETURN" => Token::Return,
        "IF" => Token::If,
        "THEN" => Token::Then,
        "ELSE" => Token::Else,
        "FI" => Token::Fi,
        _ => Token::Ident(s),
    });

    let token = op.or(ctrl).or(ident);

    let comment = just('{')
        .then(any().and_is(just('}').not()).repeated())
        .then(just('}'))
        .padded();

    token
        .map_with(|tok, e| (tok, e.span()))
        .padded_by(comment.repeated())
        .padded()
        .recover_with(skip_then_retry_until(any().ignored(), end()))
        .repeated()
        .collect()
}

impl fmt::Display for Token<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Op(c) => write!(f, "{}", c),
            Token::Ctrl(c) => write!(f, "{}", c),
            Token::Ident(s) => write!(f, "{}", s),
            Token::Decl => write!(f, "DECL"),
            Token::Beg => write!(f, "BEG"),
            Token::End => write!(f, "END"),
            Token::While => write!(f, "WHILE"),
            Token::Do => write!(f, "DO"),
            Token::Od => write!(f, "OD"),
            Token::Return => write!(f, "RETURN"),
            Token::If => write!(f, "IF"),
            Token::Then => write!(f, "THEN"),
            Token::Else => write!(f, "ELSE"),
            Token::Fi => write!(f, "FI"),
        }
    }
}

#[cfg(test)]
mod tests {
    use chumsky::Parser;

    use crate::lexer::{Token, lexer};

    #[test]
    fn test_lex_examples() {
        let tokens = lexer()
            .parse(include_str!("../examples/Examples.prog"))
            .unwrap();

        assert_eq!(tokens.len(), 951);
        assert_eq!(tokens[0].0, Token::Ident("RTC1"));
        assert_eq!(tokens[1].0, Token::Ctrl('('));
        assert_eq!(tokens[2].0, Token::Ident("R"));
        assert_eq!(tokens[3].0, Token::Ctrl(')'));
        assert_eq!(tokens[4].0, Token::Decl);
        assert_eq!(tokens[5].0, Token::Ident("P"));
        assert_eq!(tokens[6].0, Token::Ctrl(','));
        assert_eq!(tokens[7].0, Token::Ident("Q"));
        assert_eq!(tokens[8].0, Token::Ctrl(','));
        assert_eq!(tokens[9].0, Token::Ident("S"));
        assert_eq!(tokens[10].0, Token::Beg);
    }
}
