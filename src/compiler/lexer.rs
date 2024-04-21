use std::ops::Range;

use logos::Logos;

#[derive(Debug, Clone)]
pub struct Lexer<'source> {
    inner: logos::Lexer<'source, Token>,
    peeked: Option<Option<Token>>,
}

#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Logos, strum_macros::Display)]
#[logos(skip r"[ \t\n\f]+")]
pub enum Token {
    #[token("(")] ParenOpen, #[token(")")] ParenClose,
    #[token("{")] BraceOpen, #[token("}")] BraceClose,
    #[token("let")] Let, #[token("if")] If, #[token("else")] Else,
    #[token("for")] For, #[token("while")] While,
    #[token("return")] Return, #[token("fn")] Fn, 
    #[token("class")] Class, #[token("super")] Super, #[token("this")] This,
    #[token("=")] Assign,
    #[token("+")] Add, #[token("-")] Sub,
    #[token("*")] Mul, #[token("/")] Div, #[token("%")] Rem, 
    #[token("==")] Eq, #[token("!=")] Neq,
    #[token(">")] Gr, #[token("<")] Le, #[token(">=")] Geq, #[token("<=")] Leq,
    #[token("&&")] #[token("and")] And, #[token("||")] #[token("or")] Or, #[token("!")] #[token("not")] Not,
    #[token(";")] Semicolon, #[token(".")] Dot, #[token(",")] Comma,
    #[regex(r"[0-9]+\.?[0-9]*")] Number,
    #[regex(r"\p{Alphabetic}(\p{Alphabetic}|\d|_)*")] Identifier,
    #[regex(r#""[^"]*""#)] String,
    #[token("true")] True, #[token("false")] False,
    #[token("nil")] Nil,
    Error,
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

    pub fn peek(&mut self) -> Option<Token> {
        *self.peeked.get_or_insert_with(|| Self::next_unwrapped(&mut self.inner))
    }

    fn next_unwrapped(inner: &mut logos::Lexer<'source, Token>) -> Option<Token> {
        inner.next().map(|res| res.unwrap_or(Token::Error))
    }
}

impl<'source> Iterator for Lexer<'source> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        match self.peeked.take() {
            Some(peeked) => peeked,
            None => Self::next_unwrapped(&mut self.inner),
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
        let mut lex = Lexer::lex(r#"arg bar 70.9 % $$"#);

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
