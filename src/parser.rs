use std::ops::Range;

use crate::{
    ast::{Expr, Stmt, Value, Var},
    lexer::{Lexer, Operator, Token, TokenKind},
};

#[derive(Debug)]
pub struct ParserError {
    kind: ParserErrorKind,
    span: Range<usize>,
}

#[derive(Debug)]
pub enum ParserErrorKind {
    EndOfInput,
    UnexpectedToken { expected: Vec<TokenKind> },
    UnexpectedNotPrefixOp,
    WrongAssignment,
    LexerError,
}

impl ParserErrorKind {
    pub fn with_span(self, span: Range<usize>) -> ParserError {
        ParserError { kind: self, span }
    }
}

#[derive(Debug, Clone)]
pub struct Parser<'source> {
    lexer: Lexer<'source>,
}

impl<'source> Parser<'source> {
    pub fn parse(lexer: Lexer<'source>) -> Self {
        Self { lexer }
    }

    // pub fn block(&mut self) -> Result<Vec<Stmt>, Vec<ParserError>> {
        
    // }

    pub fn stmt(&mut self) -> Result<Stmt, ParserError> {
        if self.matches(TokenKind::Let)? {
            self.next_token()?;
            let Token::Identifier(id) = self.next_token()? else { return Err(ParserErrorKind::UnexpectedToken { expected: vec![TokenKind::Identifier] }.with_span(self.token_span())); };

            self.expect(TokenKind::Assign)?;

            let expr = self.expr()?;

            self.expect(TokenKind::Semicolon)?;

            return Ok(Stmt::Declaration(id.to_owned(), expr));
        }

        let left = self.expr()?;
        if self.matches(TokenKind::Assign)? {
            self.next_token()?;
            let Expr::Variable(var) = left else { return Err(ParserErrorKind::WrongAssignment.with_span(self.token_span())); };

            let right = self.expr()?;

            self.expect(TokenKind::Semicolon)?;

            return Ok(Stmt::Assign(var, right));
        }

        self.expect(TokenKind::Semicolon)?;

        Ok(Stmt::Expr(left))
    }

    pub fn expr(&mut self) -> Result<Expr, ParserError> {
        self.expr_bp(0)
    }

    fn expr_bp(&mut self, min_bp: u8) -> Result<Expr, ParserError> {
        let mut left = match self.next_token()? {
            Token::Number(n) => Expr::Value(Value::Number(n)),
            Token::String(s) => Expr::Value(Value::String(s.to_owned())),
            Token::Bool(b) => Expr::Value(Value::Bool(b)),
            Token::Identifier(s) => Expr::Variable(Var { name: s.to_owned() }),
            Token::If => {
                let cond = self.expr()?;

                self.expect(TokenKind::BraceOpen)?;
                let then = self.expr()?;
                self.expect(TokenKind::BraceClose)?;

                let els = if self.matches(TokenKind::Else)? {
                    self.next_token()?;

                    self.expect(TokenKind::BraceOpen)?;
                    let els = self.expr()?;
                    self.expect(TokenKind::BraceClose)?;

                    Some(els)
                } else {
                    None
                };

                Expr::If(Box::new(cond), Box::new(then), els.map(Box::new))
            }
            Token::ParenOpen => {
                let left = self.expr()?;
                self.expect(TokenKind::ParenClose)?;
                left
            }
            Token::Operator(op) => {
                let (_, r_bp) = Self::prefix_binding_power(&Token::Operator(op))
                    .ok_or(ParserErrorKind::UnexpectedNotPrefixOp.with_span(self.token_span()))?;
                let right = self.expr_bp(r_bp)?;

                Expr::Unary(op, Box::new(right))
            }
            _ => {
                return Err(ParserErrorKind::UnexpectedToken {
                    expected: vec![
                        TokenKind::Number,
                        TokenKind::String,
                        TokenKind::Bool,
                        TokenKind::Identifier,
                        TokenKind::Operator,
                        TokenKind::ParenOpen,
                    ],
                }
                .with_span(self.token_span()));
            }
        };

        loop {
            let tok = self.peek_token()?;

            if let Some((l_bp, _)) = Self::postfix_binding_power(tok) {
                if l_bp < min_bp {
                    break;
                }

                let &Token::Operator(op) = tok else { unreachable!() };

                self.next_token()?;
                left = Expr::Unary(op, Box::new(left));

                continue;
            }

            if let Some((l_bp, r_bp)) = Self::infix_binding_power(tok) {
                if l_bp < min_bp {
                    break;
                }

                let &Token::Operator(op) = tok else { unreachable!() };

                self.next_token()?;
                let right = self.expr_bp(r_bp)?;
                left = Expr::Binary(op, Box::new(left), Box::new(right));

                continue;
            }

            break;
        }

        Ok(left)
    }

    fn prefix_binding_power(tok: &Token) -> Option<((), u8)> {
        let bp = match tok {
            Token::Operator(op) => match op {
                Operator::Add | Operator::Sub | Operator::Not => ((), 201),
                _ => return None,
            },
            _ => return None,
        };
        Some(bp)
    }

    fn infix_binding_power(tok: &Token) -> Option<(u8, u8)> {
        let bp = match tok {
            Token::Operator(op) => match op {
                Operator::Eq
                | Operator::Neq
                | Operator::Leq
                | Operator::Geq
                | Operator::Gr
                | Operator::Le => (50, 51),
                Operator::Add | Operator::Sub | Operator::Or => (100, 101),
                Operator::Mul | Operator::Div | Operator::Rem | Operator::And => (150, 151),
                _ => return None,
            },
            _ => return None,
        };
        Some(bp)
    }

    fn postfix_binding_power(tok: &Token) -> Option<(u8, ())> {
        let bp = match tok {
            // Operator::ParenOpen => (220, ()),
            _ => return None,
        };
        Some(bp)
    }

    fn matches(&mut self, ttype: TokenKind) -> Result<bool, ParserError> {
        let peeked = self.peek_token()?;

        Ok(TokenKind::from(peeked) == ttype)
    }

    fn expect(&mut self, ttype: TokenKind) -> Result<(), ParserError> {
        if self.matches(ttype)? {
            self.next_token()?;
            Ok(())
        } else {
            Err(ParserErrorKind::UnexpectedToken {
                expected: vec![ttype],
            }.with_span(self.token_span()))
        }
    }

    fn next_token(&mut self) -> Result<Token<'source>, ParserError> {
        self.lexer
            .next()
            .ok_or(ParserErrorKind::EndOfInput.with_span(self.token_span()))?
            .map_err(|_| ParserErrorKind::LexerError.with_span(self.lexer.span()))
    }

    fn peek_token(&mut self) -> Result<&Token<'source>, ParserError> {
        self.lexer.peek();

        let peeked_span = self.lexer.span();

        self.lexer
            .peek()
            .ok_or(ParserErrorKind::EndOfInput.with_span(peeked_span.clone()))?
            .as_ref()
            .map_err(|_| ParserErrorKind::LexerError.with_span(peeked_span))
    }

    fn token_span(&self) -> Range<usize> {
        self.lexer.span()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_expr_test() {
        let input = "-(1 + 10) * 20 / ((+(5 + 9)))";

        let parsed = Parser::parse(Lexer::lex(input)).expr();

        println!("{:?}", parsed);
    }
}
