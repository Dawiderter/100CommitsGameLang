use std::str::FromStr;

use logos::Logos;

#[derive(Debug, Default)]
pub struct LexError;

#[derive(Debug, PartialEq, strum_macros::Display, strum_macros::EnumString)]
pub enum Operator {
    #[strum(to_string = "+")]
    Add,
    #[strum(to_string = "-")]
    Sub,
    #[strum(to_string = "*")]
    Mul,
    #[strum(to_string = "/")]
    Div,
    #[strum(to_string = "%")]
    Rem,
    #[strum(to_string = "==")]
    Eq,
    #[strum(to_string = "!=")]
    Neq,
    #[strum(to_string = ">")]
    Gr,
    #[strum(to_string = "<")]
    Le,
    #[strum(to_string = ">=")]
    Geq,
    #[strum(to_string = "<=")]
    Leq,
    #[strum(to_string = "&&")]
    And,
    #[strum(to_string = "||")]
    Or,
    #[strum(to_string = "!")]
    Not,
}

#[derive(Debug, PartialEq, Logos)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token<'source> {
    #[token("{")]
    BraceOpen,
    #[token("}")]
    BraceClose,
    #[token("(")]
    ParenOpen,
    #[token(")")]
    ParenClose,
    #[token("=")]
    Assign,
    #[regex(r"[+\-*/%!><]|[=><!]=|(&&)|(\|\|)", |lex| Operator::from_str(lex.slice()).ok())]
    Operator(Operator),
    #[token(".")]
    Period,
    #[regex(r"[0-9]+\.?[0-9]*", |lex| lex.slice().parse().ok())]
    Number(f64),
    #[regex(r"\p{Alphabetic}(\p{Alphabetic}|\d|_)*")]
    Identifier(&'source str),
    #[regex(r#""[^"]*""#, |lex| { let s = lex.slice(); &s[1..(s.len() - 1)]  })]
    String(&'source str),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_test() {
        let mut lex = Token::lexer(r#"99aaa"#);

        while let Some(tok) = lex.next() {
            println!("{:?} {}", tok, lex.slice())
        }
    }

    #[test]
    fn op_test() {
        let mut lex = Token::lexer(r#"+ - * / == != > < >= <= && || !"#);

        while let Some(tok) = lex.next() {
            println!("{:?} {}", tok, lex.slice())
        }
    }
}
