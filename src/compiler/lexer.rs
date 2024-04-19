use std::ops::Range;

use logos::Logos;

#[derive(Debug, Default, PartialEq, Clone)]
pub struct LexError;

#[derive(Debug, Clone)]
pub struct Lexer<'source> {
    inner: logos::Lexer<'source, Token>,
    peeked: Option<Option<Result<Token, LexError>>>,
}

#[rustfmt::skip]
#[derive(Debug, Clone, PartialEq, Logos, strum_macros::Display)]
#[logos(skip r"[ \t\n\f]+")]
#[logos(error=LexError)]
pub enum Token {
    #[token("(")] ParenOpen, #[token(")")] ParenClose,
    #[token("{")] BraceOpen, #[token("}")] BraceClose,
    #[token("let")] Let, #[token("if")] If, #[token("else")] Else,
    #[token("=")] Assign,
    #[token("+")] Add, #[token("-")] Sub,
    #[token("*")] Mul, #[token("/")] Div, #[token("%")] Rem, 
    #[token("==")] Eq, #[token("!=")] Neq,
    #[token(">")] Gr, #[token("<")] Le,
    #[token(">=")] Geq, #[token("<=")] Leq,
    #[token("&&")] #[token("and")] And, #[token("||")] #[token("or")] Or, #[token("!")] #[token("not")] Not,
    #[token(";")] Semicolon, #[token(".")] Period,
    #[regex(r"[0-9]+\.?[0-9]*")] Number,
    #[regex(r"\p{Alphabetic}(\p{Alphabetic}|\d|_)*")] Identifier,
    #[regex(r#""[^"]*""#)] String,
    #[token("true")] #[token("false")] Bool,
    EOI,
}

impl<'source> Lexer<'source> {
    pub fn lex(source: &'source str) -> Self {
        Self {
            inner: Token::lexer(source),
            peeked: None,
        }
    }

    pub fn slice(&self) -> &str {
        self.inner.slice()
    }

    pub fn span(&self) -> Range<usize> {
        self.inner.span()
    }

    pub fn peek(&mut self) -> Option<&Result<Token, LexError>> {
        self.peeked.get_or_insert_with(|| self.inner.next()).as_ref()
    }
}

impl<'source> Iterator for Lexer<'source> {
    type Item = Result<Token, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.peeked.take() {
            Some(peeked) => peeked,
            None => self.inner.next(),
        }
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
