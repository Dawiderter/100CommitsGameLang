use std::ops::Range;

use crate::{
    ast::{Expr, Stmt, Value, Var},
    lexer::{Lexer, Operator, Token, TokenType},
};

#[derive(Debug)]
pub enum ParserError {
    EndOfInput,
    UnexpectedToken {
        span: Range<usize>,
        expected: Vec<TokenType>,
    },
    UnexpectedOperator(Range<usize>),
    WrongAssignment(Range<usize>),
    MissingClosingParen(Range<usize>),
    LexerError(Range<usize>),
}

#[derive(Debug, Clone)]
pub struct Parser<'source> {
    lexer: Lexer<'source>,
}

impl<'source> Parser<'source> {
    pub fn parse(lexer: Lexer<'source>) -> Self {
        Self { lexer }
    }

    pub fn stmt(&mut self) -> Result<Stmt, ParserError> {
        if self.matches(TokenType::Let)? {
            self.next_token()?;
            let Token::Identifier(id) = self.next_token()? else { return Err(ParserError::UnexpectedToken { span: self.token_span(), expected: vec![TokenType::Identifier] }); };

            self.expect(TokenType::Assign)?;

            let expr = self.expr()?;

            self.expect(TokenType::Semicolon)?;

            return Ok(Stmt::Declaration(id.to_owned(), expr));
        }

        let left = self.expr()?;
        if self.matches(TokenType::Assign)? {
            self.next_token()?;
            let Expr::Variable(var) = left else { return Err(ParserError::WrongAssignment(self.token_span())); };

            let right = self.expr()?;

            self.expect(TokenType::Semicolon)?;

            return Ok(Stmt::Assign(var, right));
        }

        self.expect(TokenType::Semicolon)?;

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
            Token::Operator(Operator::ParenOpen) => {
                let left = self.expr()?;
                let end_token = self.next_token()?;
                if end_token != Token::Operator(Operator::ParenClose) {
                    return Err(ParserError::MissingClosingParen(self.token_span()));
                };
                left
            }
            Token::Operator(op) => {
                let (_, r_bp) = Self::prefix_binding_power(op)
                    .ok_or(ParserError::UnexpectedOperator(self.token_span()))?;
                let right = self.expr_bp(r_bp)?;

                Expr::Unary(op, Box::new(right))
            }
            _ => {
                return Err(ParserError::UnexpectedToken {
                    span: self.token_span(),
                    expected: vec![
                        TokenType::Number,
                        TokenType::String,
                        TokenType::Bool,
                        TokenType::Identifier,
                        TokenType::Operator,
                    ],
                })
            }
        };

        loop {
            let &op = match self.peek_token()? {
                Token::EOF => break,
                Token::Operator(op) => op,
                _ => break,
            };

            if let Some((l_bp, _)) = Self::postfix_binding_power(op) {
                if l_bp < min_bp {
                    break;
                }

                self.next_token()?;
                left = Expr::Unary(op, Box::new(left));

                continue;
            }

            if let Some((l_bp, r_bp)) = Self::infix_binding_power(op) {
                if l_bp < min_bp {
                    break;
                }

                self.next_token()?;
                let right = self.expr_bp(r_bp)?;
                left = Expr::Binary(op, Box::new(left), Box::new(right));

                continue;
            }

            break;
        }

        Ok(left)
    }

    fn prefix_binding_power(op: Operator) -> Option<((), u8)> {
        let bp = match op {
            Operator::Add | Operator::Sub | Operator::Not => ((), 201),
            _ => return None,
        };
        Some(bp)
    }

    fn infix_binding_power(op: Operator) -> Option<(u8, u8)> {
        let bp = match op {
            Operator::Eq
            | Operator::Neq
            | Operator::Leq
            | Operator::Geq
            | Operator::Gr
            | Operator::Le => (50, 51),
            Operator::Add | Operator::Sub | Operator::Or => (100, 101),
            Operator::Mul | Operator::Div | Operator::Rem | Operator::And => (150, 151),
            _ => return None,
        };
        Some(bp)
    }

    fn postfix_binding_power(op: Operator) -> Option<(u8, ())> {
        let bp = match op {
            // Operator::ParenOpen => (220, ()),
            _ => return None,
        };
        Some(bp)
    }

    fn matches(&mut self, ttype: TokenType) -> Result<bool, ParserError> {
        let peeked = self.peek_token()?;

        Ok(TokenType::from(peeked) == ttype)
    }

    fn expect(&mut self, ttype: TokenType) -> Result<(), ParserError> {
        if self.matches(ttype)? {
            self.next_token()?;
            Ok(())
        } else {
            Err(ParserError::UnexpectedToken {
                span: self.token_span(),
                expected: vec![ttype],
            })
        }
    }

    fn next_token(&mut self) -> Result<Token<'source>, ParserError> {
        self.lexer
            .next()
            .ok_or(ParserError::EndOfInput)?
            .map_err(|_| ParserError::LexerError(self.lexer.span()))
    }

    fn peek_token(&mut self) -> Result<&Token<'source>, ParserError> {
        self.lexer.peek();

        let peeked_span = self.lexer.span();

        self.lexer
            .peek()
            .ok_or(ParserError::EndOfInput)?
            .as_ref()
            .map_err(|_| ParserError::LexerError(peeked_span))
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
