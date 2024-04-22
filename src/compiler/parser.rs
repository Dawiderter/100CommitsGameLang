use std::ops::Range;

use log::warn;

use crate::bytecode::{chunk::CodeChunk, opcodes::*, value::Value};

use super::lexer::{Lexer, Token};

#[derive(Debug)]
pub struct Parser<'source, 'code> {
    lexer: Lexer<'source>,
    code: &'code mut CodeChunk,
}

#[derive(Debug, Clone)]
pub struct ParsingError {
    pub msg: String,
    pub span: Range<usize>
}

impl<'source, 'code> Parser<'source, 'code> {
    pub fn parse_source(source: &'source str, code: &'code mut CodeChunk) -> Result<(), Vec<ParsingError>> {
        
        let mut parser = Self { lexer: Lexer::lex(source), code };

        if let Err(err) = parser.expression() {
            return Err(vec![err]);
        }
        if let Err(err) = parser.consume(None) {
            return Err(vec![err]);
        }
        parser.code.push_code(OP_RETURN);

        Ok(())
    }

    fn expression(&mut self) -> Result<(), ParsingError> {
        self.expression_bp(0)  
    }
    
    fn expression_bp(&mut self, min_bp: u8) -> Result<(), ParsingError> {
        let Some(op) = self.lexer.peek() else { return Err(self.error_at_current("Expected expression".to_string())); };
        match op {
            Token::ParenOpen => {
                self.lexer.next();
                self.expression()?;
                self.consume_some(Token::ParenClose)?;
            }
            Token::Number => {
                self.number();
            }
            Token::False => {
                self.lexer.next();
                self.code.push_span_info(self.lexer.span());
                self.code.push_code(OP_FALSE)
            }
            Token::True => {
                self.lexer.next();
                self.code.push_span_info(self.lexer.span());
                self.code.push_code(OP_TRUE)
            }
            Token::Nil => {
                self.lexer.next();
                self.code.push_span_info(self.lexer.span());
                self.code.push_code(OP_NIL)
            }
            prefix_token => {
                match Self::prefix_bp(prefix_token) {
                    Some((_, r_bp)) => {
                        let op_span = self.lexer.span();
                        self.lexer.next();
                        self.expression_bp(r_bp)?;
                        self.code.push_span_info(op_span);
                        match prefix_token {
                            Token::Sub => self.code.push_code(OP_NEG),
                            Token::Not => self.code.push_code(OP_NOT),
                            _ => { warn!("Unsupported token parsed as prefix operator: {:?}", op) }
                        }
                    },
                    None => return Err(self.error_at_current(format!("Unexpected token: {:?}", op))),
                }
            }
        }

        loop {
            let Some(op) = self.lexer.peek() else { return Ok(()); };
            match Self::infix_bp(op) {
                Some((l_bp, r_bp)) => {
                    if l_bp < min_bp {
                        break;
                    }
                    let op_span = self.lexer.span();
                    self.lexer.next();
                    self.expression_bp(r_bp)?;
                    self.code.push_span_info(op_span);
                    match op {
                        Token::Add => self.code.push_code(OP_ADD),
                        Token::Sub => self.code.push_code(OP_SUB),
                        Token::Mul => self.code.push_code(OP_MUL),
                        Token::Div => self.code.push_code(OP_DIV),
                        Token::Eq => self.code.push_code(OP_EQUAL),
                        Token::Neq => { self.code.push_code(OP_EQUAL); self.code.push_code(OP_NOT) }
                        Token::Gr => self.code.push_code(OP_GREATER),
                        Token::Le => self.code.push_code(OP_LESS),
                        Token::Geq => { self.code.push_code(OP_LESS); self.code.push_code(OP_NOT) },
                        Token::Leq => { self.code.push_code(OP_GREATER); self.code.push_code(OP_NOT) },
                        Token::And => self.code.push_code(OP_AND), 
                        Token::Or => self.code.push_code(OP_OR), 
                        _ => { warn!("Unsupported token parsed as infix operator: {:?}", op) }
                    }
                },
                None => break,
            }
        }

        Ok(())
    }

    fn number(&mut self) {
        let slice = self.lexer.slice();
        let num = slice.parse().expect("Internal panic: Can't parse number");
        self.emit_constant(Value::Number(num));
        self.lexer.next();
    }

    fn infix_bp(token: Token) -> Option<(u8, u8)> {
        let bp = match token {
            Token::Or => (1, 2),
            Token::And => (3, 4),
            Token::Eq | Token::Neq | Token::Geq | Token::Leq | Token::Le | Token::Gr => (5, 6),
            Token::Add | Token::Sub => (7, 8),
            Token::Mul | Token::Div => (9, 10),
            _ => return None,
        };
        Some(bp)
    }

    fn prefix_bp(token: Token) -> Option<((), u8)> {
        let bp = match token {
            Token::Sub => ((), 11),
            Token::Not => ((), 11),
            _ => return None,
        };
        Some(bp)
    }

    fn consume_some(&mut self, token: Token) -> Result<(), ParsingError> {
        self.consume(Some(token))
    }

    fn consume(&mut self, token: Option<Token>) -> Result<(), ParsingError> {
        let peeked = self.lexer.peek();
        if peeked == token {
            self.lexer.next();
            Ok(())
        } else {
            Err(self.error_at_current(format!("Expected {token:?}, got {peeked:?}")))
        }
    }

    fn error_at_current(&self, msg: String) -> ParsingError {
        ParsingError::at(self.lexer.span(), msg)   
    }

    fn emit_constant(&mut self, value: Value) {
        self.code.push_span_info(self.lexer.span());
        let constant = self.code.push_constant(value);
        self.code.push_code(OP_CONSTANT);
        self.code.push_code(constant);
    }
}

impl ParsingError{
    fn at(span: Range<usize>, msg: String) -> ParsingError {
        Self { msg, span }
    }
}

#[cfg(test)]
mod tests {
    use crate::bytecode::vm::VM;

    use super::*;

    fn init_logger() {
        let _ = env_logger::builder()
            .filter_level(log::LevelFilter::Trace)
            .format_timestamp(None)
            .is_test(true)
            .try_init();
    }

    #[test]
    fn parse_test() {
        init_logger();

        let test_str = "(1 + 5) - - - (8 - 2)";
        let mut code = CodeChunk::new();
        Parser::parse_source(test_str, &mut code).unwrap();
        eprintln!("{}", code);

        VM::init(&code).run().unwrap();
    }
}