use std::ops::Range;

use ecow::EcoString;
use log::warn;

use crate::bytecode::{chunk::CodeChunk, object::ObjectHeap, opcodes::*, value::Value};

use super::lexer::{Lexer, Token};

#[derive(Debug)]
pub struct Parser<'source, 'code, 'heap> {
    lexer: Lexer<'source>,
    code: &'code mut CodeChunk,
    heap: &'heap mut ObjectHeap,
    locals: Locals,
}

#[derive(Debug, Clone)]
pub struct ParsingError {
    pub msg: String,
    pub span: Range<usize>,
}

impl<'source, 'code, 'heap> Parser<'source, 'code, 'heap> {
    pub fn parse_source(
        source: &'source str,
        code: &'code mut CodeChunk,
        heap: &'heap mut ObjectHeap,
    ) -> Result<(), Vec<ParsingError>> {
        let mut errors = Vec::new();

        let mut parser = Self {
            lexer: Lexer::lex(source),
            code,
            heap,
            locals: Locals::new(),
        };

        while parser.lexer.peek().is_some() {
            let res = parser.statement();
            if let Err(err) = res {
                errors.push(err);
                loop {
                    let maybe_peeked = parser.lexer.peek();
                    let Some(peeked) = maybe_peeked else {
                        break;
                    };
                    match peeked {
                        Token::Semicolon => {
                            parser.lexer.next();
                            break;
                        }
                        Token::Class | Token::Fn | Token::Let | Token::For | Token::If | Token::While | Token::Print | Token::Return => {
                            break;
                        }
                        _ => {
                            parser.lexer.next();
                        }
                    }
                }
            }
        }

        if errors.is_empty() {
            parser.code.push_code(OP_RETURN);
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn statement(&mut self) -> Result<(), ParsingError> {
        let Some(op) = self.lexer.peek() else {
            return Err(self.error_at_current("Expected statement".to_string()));
        };

        match op {
            Token::Print => {
                self.lexer.next();
                self.expression()?;
                self.code.push_code(OP_PRINT);
                self.consume_some(Token::Semicolon)?;
            }
            Token::If => {
                self.lexer.next();
                self.expression()?;
                let els_jmp = self.emit_jump(OP_JUMP_F);
                self.code.push_code(OP_POP);
                self.block()?;
                let then_end_jmp = self.emit_jump(OP_JUMP);

                self.patch_jump(els_jmp, self.code.size())?;   
                self.code.push_code(OP_POP);
                             
                if self.lexer.peek() == Some(Token::Else) {
                    self.lexer.next();
                    self.block()?;
                }
                self.patch_jump(then_end_jmp, self.code.size())?;

            }
            Token::Let => {
                self.lexer.next();
                self.expect_some(Token::Identifier)?;
                let identifier = self.identifier();
                if self.lexer.peek() == Some(Token::Assign) {
                    self.lexer.next();
                    self.expression()?;
                } else {
                    self.code.push_code(OP_NIL);
                }
                if self.locals.depth == 0 {
                    self.emit_global_definition(identifier);
                } else {
                    self.locals.push_local(identifier);
                }
                self.consume_some(Token::Semicolon)?;
            }
            Token::BraceOpen => {
                self.locals.enter_scope();
                self.block()?;
                let dropped_locals = self.locals.exit_scope();
                for _ in 0..dropped_locals {
                    self.code.push_code(OP_POP);
                }
            }
            _ => {
                self.expression()?;
                self.code.push_code(OP_POP);
                self.consume_some(Token::Semicolon)?;
            }
        }

        Ok(())
    }

    fn block(&mut self) -> Result<(), ParsingError> {
        self.consume_some(Token::BraceOpen)?;
        while !matches!(self.lexer.peek(), None | Some(Token::BraceClose)) {
            self.statement()?;
        }
        self.consume_some(Token::BraceClose)?;
        Ok(())
    }

    fn expression(&mut self) -> Result<(), ParsingError> {
        self.expression_bp(0)
    }

    fn expression_bp(&mut self, min_bp: u8) -> Result<(), ParsingError> {
        let Some(op) = self.lexer.peek() else {
            return Err(self.error_at_current("Expected expression".to_string()));
        };
        match op {
            Token::ParenOpen => {
                self.lexer.next();
                self.expression()?;
                self.consume_some(Token::ParenClose)?;
            }
            Token::Identifier => {
                let identifier = self.identifier();
                let maybe_local = self.locals.find_local(identifier.as_str());

                let (set, get, arg) = if let Some(local) = maybe_local {
                    (OP_SET_LOCAL, OP_GET_LOCAL, local)
                } else {
                    let constant = self.push_string_constant(identifier);
                    (OP_SET_GLOBAL, OP_GET_GLOBAL, constant)
                };

                if self.lexer.peek() == Some(Token::Assign) {
                    self.lexer.next();
                    if min_bp > 0 {
                        return Err(
                            self.error_at_current("Invalid left side of assignment".to_string())
                        );
                    }
                    self.expression()?;
                    self.code.push_code(set);
                    self.code.push_code(arg);
                } else {
                    self.code.push_code(get);
                    self.code.push_code(arg);
                }
            }
            Token::Number => {
                self.number();
            }
            Token::String => {
                self.string();
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
            prefix_token => match Self::prefix_bp(prefix_token) {
                Some((_, r_bp)) => {
                    let op_span = self.lexer.span();
                    self.lexer.next();
                    self.expression_bp(r_bp)?;
                    self.code.push_span_info(op_span);
                    match prefix_token {
                        Token::Sub => self.code.push_code(OP_NEG),
                        Token::Not => self.code.push_code(OP_NOT),
                        _ => {
                            warn!("Unsupported token parsed as prefix operator: {:?}", op)
                        }
                    }
                }
                None => return Err(self.error_at_current(format!("Unexpected token: {:?}", op))),
            },
        }

        loop {
            let Some(op) = self.lexer.peek() else {
                return Ok(());
            };
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
                        Token::Neq => {
                            self.code.push_code(OP_EQUAL);
                            self.code.push_code(OP_NOT)
                        }
                        Token::Gr => self.code.push_code(OP_GREATER),
                        Token::Le => self.code.push_code(OP_LESS),
                        Token::Geq => {
                            self.code.push_code(OP_LESS);
                            self.code.push_code(OP_NOT)
                        }
                        Token::Leq => {
                            self.code.push_code(OP_GREATER);
                            self.code.push_code(OP_NOT)
                        }
                        Token::And => self.code.push_code(OP_AND),
                        Token::Or => self.code.push_code(OP_OR),
                        _ => {
                            warn!("Unsupported token parsed as infix operator: {:?}", op)
                        }
                    }
                }
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

    fn string(&mut self) {
        let slice = self.lexer.slice();
        let string = EcoString::from(&slice[1..slice.len() - 1]);
        let id = self.heap.intern_string(string);
        self.emit_constant(Value::Object(id));
        self.lexer.next();
    }

    fn identifier(&mut self) -> EcoString {
        self.lexer.next();
        let slice = self.lexer.slice();
        EcoString::from(slice)
    }

    fn infix_bp(token: Token) -> Option<(u8, u8)> {
        let bp = match token {
            Token::Or => (4, 5),
            Token::And => (6, 7),
            Token::Eq | Token::Neq | Token::Geq | Token::Leq | Token::Le | Token::Gr => (10, 11),
            Token::Add | Token::Sub => (15, 16),
            Token::Mul | Token::Div => (17, 18),
            _ => return None,
        };
        Some(bp)
    }

    fn prefix_bp(token: Token) -> Option<((), u8)> {
        let bp = match token {
            Token::Sub => ((), 25),
            Token::Not => ((), 25),
            _ => return None,
        };
        Some(bp)
    }

    fn consume_some(&mut self, token: Token) -> Result<(), ParsingError> {
        self.consume(Some(token))
    }

    fn expect_some(&mut self, token: Token) -> Result<(), ParsingError> {
        self.expect(Some(token))
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

    fn expect(&mut self, token: Option<Token>) -> Result<(), ParsingError> {
        let peeked = self.lexer.peek();
        if peeked == token {
            Ok(())
        } else {
            Err(self.error_at_current(format!("Expected {token:?}, got {peeked:?}")))
        }
    }

    fn error_at_current(&self, msg: String) -> ParsingError {
        ParsingError::at(self.lexer.span(), msg)
    }

    fn push_string_constant(&mut self, string: EcoString) -> u8 {
        self.code.push_span_info(self.lexer.span());
        let obj = self.heap.intern_string(string);
        self.code.push_constant(Value::Object(obj))
    }

    fn emit_constant(&mut self, value: Value) {
        self.code.push_span_info(self.lexer.span());
        let constant = self.code.push_constant(value);
        self.code.push_code(OP_CONSTANT);
        self.code.push_code(constant);
    }

    fn emit_global_definition(&mut self, identifier: EcoString) {
        let constant = self.push_string_constant(identifier);
        self.code.push_code(OP_DEF_GLOBAL);
        self.code.push_code(constant);
    }

    fn emit_jump(&mut self, instr: u8) -> usize {
        self.code.push_code(instr);
        self.code.push_code(0xFF);
        self.code.push_code(0xFF);
        self.code.size() - 2
    }

    fn patch_jump(&mut self, offset: usize, to: usize) -> Result<(), ParsingError> {
        let relative_jump : u16 = (to - offset - 2).try_into().map_err(|_| self.error_at_current("Jump too long".to_owned()))?;
        let [big,little] = relative_jump.to_be_bytes();
        self.code.patch(offset, big);
        self.code.patch(offset + 1, little);
        Ok(())
    }
}

impl ParsingError {
    fn at(span: Range<usize>, msg: String) -> ParsingError {
        Self { msg, span }
    }
}

#[derive(Debug)]
pub struct Locals {
    stack: Vec<(EcoString, u8)>,
    depth: u8,
}

impl Locals {
    pub fn new() -> Self {
        Self {
            stack: Vec::with_capacity(256),
            depth: 0,
        }
    }

    pub fn find_local(&self, identifier: &str) -> Option<u8> {
        self.stack
            .iter()
            .rev()
            .position(|(p, _)| p.as_str() == identifier)
            .map(|idx| (self.stack.len() - 1 - idx) as u8)
    }

    pub fn push_local(&mut self, identifier: EcoString) -> bool {
        if self.stack.len() >= 256 {
            false
        } else {
            self.stack.push((identifier, self.depth));
            true
        }
    }

    pub fn enter_scope(&mut self) {
        self.depth += 1;
    }

    pub fn exit_scope(&mut self) -> u8 {
        let len = self.stack.len();
        let partition_point = self.stack.partition_point(|&(_, d)| d < self.depth);
        self.stack.truncate(partition_point);
        self.depth -= 1;
        (len - partition_point) as u8
    }

    pub fn depth(&self) -> u8 {
        self.depth
    }
}

impl Default for Locals {
    fn default() -> Self {
        Self::new()
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
        let mut heap = ObjectHeap::new();
        Parser::parse_source(test_str, &mut code, &mut heap).unwrap();
        eprintln!("{}", code);

        VM::init(&code, &mut heap).run().unwrap();
    }
}
