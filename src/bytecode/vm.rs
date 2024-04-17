use owo_colors::OwoColorize;

use super::chunk::{CodeChunk, OP_CONSTANT, OP_RETURN};
use super::value::Value;

#[derive(Debug, Clone)]
pub struct VM<'code> {
    code: &'code CodeChunk,
    pc: usize,
}

#[derive(Debug, Clone)]
pub enum RuntimeStep {
    KeepGoing,
    Halt,
}

#[derive(Debug, Clone)]
pub enum RuntimeError {
    UnknownError,
    UnexpectedEnd,
    UnknownCode,
    ConstantNotFound,
}

impl<'code> VM<'code> {
    pub fn init(code: &'code CodeChunk) -> Self {
        Self { code, pc: 0 }
    }

    pub fn run(&mut self) -> Result<(), RuntimeError> {
        loop {
            match self.step() {
                Ok(RuntimeStep::Halt) => return Ok(()),
                Err(err) => return Err(err),
                Ok(RuntimeStep::KeepGoing) => {}
            }
        }
    }

    pub fn step(&mut self) -> Result<RuntimeStep, RuntimeError> {
        eprintln!("{}", self.code.dissasemble_at(self.pc));

        let op = self.read_byte()?;

        match op {
            OP_RETURN => return Ok(RuntimeStep::Halt),
            OP_CONSTANT => {
                let value = self.read_constant()?;
                eprintln!("{}", value.green().bold())
            }
            _ => return Err(RuntimeError::UnknownCode),
        }

        Ok(RuntimeStep::KeepGoing)
    }

    pub fn read_byte(&mut self) -> Result<u8, RuntimeError> {
        self.pc += 1;
        self.code
            .get_byte(self.pc - 1)
            .ok_or(RuntimeError::UnexpectedEnd)
    }

    pub fn read_constant(&mut self) -> Result<&'code Value, RuntimeError> {
        let constant_offset = self.read_byte()?;
        self.code
            .get_constant(constant_offset as usize)
            .ok_or(RuntimeError::ConstantNotFound)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let mut chunk = CodeChunk::new();
        chunk.push_span_info(0..10);
        let constant = chunk.push_constant(Value::Number(1.2));
        chunk.push_code(OP_CONSTANT);
        chunk.push_code(constant);
        chunk.push_span_info(10..20);
        chunk.push_code(OP_RETURN);
        eprint!("{chunk}");

        let mut vm = VM::init(&chunk);

        let res = vm.run();
        eprintln!("{:?}", res);
    }
}
