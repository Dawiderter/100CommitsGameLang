use std::fmt::Display;

use log::trace;

use super::chunk::{CodeChunk, OP_CONSTANT, OP_RETURN};
use super::value::Value;

#[derive(Debug, Clone)]
pub struct VM<'code> {
    code: &'code CodeChunk,
    stack: Vec<Value>,
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
        Self { code, stack: Vec::new(), pc: 0 }
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
        trace!("{:4} {}", "", self.print_stack());
        trace!("{}", self.code.dissasemble_at(self.pc));

        let op = self.read_byte()?;

        match op {
            OP_RETURN => return Ok(RuntimeStep::Halt),
            OP_CONSTANT => {
                let value = self.read_constant()?.clone();
                self.stack.push(value);
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

// Execution Tracing =====
impl<'code> VM<'code> {
    fn write_stack(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        use owo_colors::OwoColorize;

        write!(f,"[")?;
        let mut stack_iter = self.stack.iter();
        if let Some(first_val) = stack_iter.next() {
            write!(f, "{}", first_val.blue())?;
        }
        for val in stack_iter {
            write!(f, ", {}", val.blue())?;
        }
        write!(f, "]")?;
        Ok(())
    }

    fn print_stack(&self) -> StackPrinter<'_,'code> {
        StackPrinter { vm: self }
    }
}

#[derive(Debug)]
pub struct StackPrinter<'vm, 'code> {
    vm: &'vm VM<'code>, 
}

impl<'vm, 'code> Display for StackPrinter<'vm, 'code> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.vm.write_stack(f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn init_logger() {
        let _ = env_logger::builder()
            // Include all events in tests
            .filter_level(log::LevelFilter::Trace)
            // Ensure events are captured by `cargo test`
            .is_test(true)
            // Ignore errors initializing the logger if tests race to configure it
            .try_init();
    }

    #[test]
    fn test() {
        init_logger();

        let mut chunk = CodeChunk::new();
        chunk.push_span_info(0..10);
        let constant1 = chunk.push_constant(Value::Number(1.2));
        let constant2 = chunk.push_constant(Value::Number(10.0));
        chunk.push_code(OP_CONSTANT);
        chunk.push_code(constant1);
        chunk.push_code(OP_CONSTANT);
        chunk.push_code(constant2);
        chunk.push_span_info(10..20);
        chunk.push_code(OP_RETURN);

        let mut vm = VM::init(&chunk);

        let res = vm.run();
        eprintln!("{:?}", res);
    }
}
