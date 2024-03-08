use std::{ops::Range, str::FromStr};

use logos::Logos;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct LexError;

#[derive(Debug, Clone)]
pub struct Lexer<'source> {
    inner: logos::Lexer<'source, Token<'source>>,
    peeked: Option<Result<Token<'source>, LexError>>,
    finished: bool,
}

#[derive(Debug, PartialEq, Clone, Copy, strum_macros::Display, strum_macros::EnumString)]
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
    #[strum(to_string = "(")]
    ParenOpen,
    #[strum(to_string = ")")]
    ParenClose,
}

#[derive(Debug, Clone, PartialEq, Logos, strum_macros::EnumDiscriminants)]
#[strum_discriminants(name(TokenType))]
#[logos(skip r"[ \t\n\f]+")]
#[logos(error=LexError)]
pub enum Token<'source> {
    #[token("{")]
    BraceOpen,
    #[token("}")]
    BraceClose,
    #[token("=")]
    Assign,
    #[token("let")]
    Let,
    #[token(";")]
    Semicolon,
    #[regex(r"[+\-*/%!><]|[=><!]=|(&&)|(\|\|)|\(|\)", |lex| Operator::from_str(lex.slice()).ok())]
    Operator(Operator),
    #[token(".")]
    Period,
    #[regex(r"[0-9]+\.?[0-9]*", |lex| lex.slice().parse().ok())]
    Number(f64),
    #[regex(r"\p{Alphabetic}(\p{Alphabetic}|\d|_)*")]
    Identifier(&'source str),
    #[regex(r#""[^"]*""#, |lex| { let s = lex.slice(); &s[1..(s.len() - 1)]  })]
    String(&'source str),
    #[token("true", |_| { true })]
    #[token("false", |_| { false })]
    Bool(bool),
    #[token("if")]
    If,
    #[token("else")]
    Else,
    EOF,
}

impl<'source> Lexer<'source> {
    pub fn lex(source: &'source str) -> Self {
        Self {
            inner: Token::lexer(source),
            peeked: None,
            finished: false,
        }
    }

    pub fn slice(&self) -> &str {
        self.inner.slice()
    }

    pub fn span(&self) -> Range<usize> {
        self.inner.span()
    }

    pub fn peek(&mut self) -> Option<&Result<Token<'source>, LexError>> {
        if self.peeked.is_none() {
            self.peeked = self.next();
        }

        self.peeked.as_ref()
    }
}

impl<'source> Iterator for Lexer<'source> {
    type Item = Result<Token<'source>, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(peeked) = self.peeked.take() {
            return Some(peeked);
        }

        if self.finished {
            return None;
        }

        let maybe_inner_next = self.inner.next();

        let Some(inner_next) = maybe_inner_next else {
            self.finished = true;
            return Some(Ok(Token::EOF));
        };

        Some(inner_next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_test() {
        let mut lex = Lexer::lex(r#"99aaa"#);

        while let Some(tok) = lex.next() {
            println!("{:?} {}", tok, lex.slice())
        }
    }

    #[test]
    fn op_test() {
        let mut lex = Lexer::lex(r#"+ - * / == != > < >= <= && || !"#);

        while let Some(tok) = lex.next() {
            println!("{:?} {}", tok, lex.slice())
        }
    }

    #[test]
    fn peek_test() {
        let mut lex = Lexer::lex(r#"arg bar 70.9 %"#);

        println!("{:?}", lex.peek());
        println!("{:?} {}", lex.span(), lex.slice());
        println!("{:?}", lex.peek());
        println!("{:?} {}", lex.span(), lex.slice());
        println!("{:?}", lex.next());
        println!("{:?} {}", lex.span(), lex.slice());

        println!("{:?}", lex.peek());
        println!("{:?} {}", lex.span(), lex.slice());
        println!("{:?}", lex.next());
        println!("{:?} {}", lex.span(), lex.slice());

        println!("{:?}", lex.peek());
        println!("{:?} {}", lex.span(), lex.slice());
        println!("{:?}", lex.next());
        println!("{:?} {}", lex.span(), lex.slice());

        println!("{:?}", lex.peek());
        println!("{:?} {}", lex.span(), lex.slice());
        println!("{:?}", lex.next());
        println!("{:?} {}", lex.span(), lex.slice());

        println!("{:?}", lex.peek());
        println!("{:?} {}", lex.span(), lex.slice());
        println!("{:?}", lex.next());
        println!("{:?} {}", lex.span(), lex.slice());

        println!("{:?}", lex.peek());
        println!("{:?} {}", lex.span(), lex.slice());
        println!("{:?}", lex.next());
        println!("{:?} {}", lex.span(), lex.slice());
    }
}
