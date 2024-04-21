use std::fmt::Display;

use log::trace;

use super::chunk::CodeChunk;
use super::opcodes::*;
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
    EmptyStack,
    UnsupportedOp
}

impl<'code> VM<'code> {
    pub fn init(code: &'code CodeChunk) -> Self {
        Self { code, stack: Vec::with_capacity(256), pc: 0 }
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

    fn step(&mut self) -> Result<RuntimeStep, RuntimeError> {
        macro_rules! bin_op {
            ($op:ident) => {
                {
                    let b = self.stack_peek(0)?;
                    let a = self.stack_peek(1)?;
                    match a.$op(b) {
                        Some(value) => {
                            self.stack_pop()?;
                            self.stack_pop()?;
                            self.stack_push(value);
                        }
                        None => return Err(RuntimeError::UnsupportedOp),
                    }
                }
            };
        }

        trace!("{:12} {}", "", self.print_stack());
        trace!("{}", self.code.dissasemble_at(self.pc));

        let op = self.read_byte()?;

        match op {
            OP_RETURN => {
                if let Some(value) = self.stack.pop() {
                    eprintln!("Returned: {}", value);
                } else {
                    eprintln!("Empty stack");
                }
                return Ok(RuntimeStep::Halt);
            },
            OP_CONSTANT => {
                let value = self.read_constant()?.clone();
                self.stack_push(value);
            }
            OP_NEG => {
                let value = self.stack_peek(0)?.neg();
                match value {
                    Some(value) => {
                        self.stack_pop()?;
                        self.stack_push(value);
                    },
                    None => return Err(RuntimeError::UnsupportedOp),
                }
            }
            OP_ADD => bin_op!(add),
            OP_SUB => bin_op!(sub),
            OP_MUL => bin_op!(mul),
            OP_DIV => bin_op!(div),
            _ => return Err(RuntimeError::UnknownCode),
        }

        Ok(RuntimeStep::KeepGoing)
    }

    fn read_byte(&mut self) -> Result<u8, RuntimeError> {
        self.pc += 1;
        self.code
            .get_byte(self.pc - 1)
            .ok_or(RuntimeError::UnexpectedEnd)
    }

    fn read_constant(&mut self) -> Result<&'code Value, RuntimeError> {
        let constant_offset = self.read_byte()?;
        self.code
            .get_constant(constant_offset as usize)
            .ok_or(RuntimeError::ConstantNotFound)
    }

    fn stack_pop(&mut self) -> Result<Value, RuntimeError> {
        self.stack.pop().ok_or(RuntimeError::EmptyStack)
    }

    fn stack_peek(&self, dist: usize) -> Result<&Value, RuntimeError> {
        self.stack.get(self.stack.len() - 1 - dist).ok_or(RuntimeError::EmptyStack)
    }

    fn stack_push(&mut self, value: Value) {
        self.stack.push(value);
    }
}

// Execution Tracing =====
impl<'code> VM<'code> {
    fn write_stack(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        use owo_colors::OwoColorize;

        write!(f,"└→[")?;
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
            .filter_level(log::LevelFilter::Trace)
            .format_timestamp(None)
            .is_test(true)
            .try_init();
    }

    #[test]
    fn vm_test() {
        init_logger();

        let mut chunk = CodeChunk::new();
        chunk.push_span_info(0..10);
        let constant = chunk.push_constant(Value::Number(1.2));
        chunk.push_code(OP_CONSTANT);
        chunk.push_code(constant);

        let constant = chunk.push_constant(Value::Number(3.4));
        chunk.push_code(OP_CONSTANT);
        chunk.push_code(constant);

        chunk.push_code(OP_ADD);

        chunk.push_span_info(10..20);

        let constant = chunk.push_constant(Value::Number(5.6));
        chunk.push_code(OP_CONSTANT);
        chunk.push_code(constant);

        chunk.push_code(OP_DIV);
        chunk.push_code(OP_NEG);

        chunk.push_code(OP_RETURN);

        let mut vm = VM::init(&chunk);

        let res = vm.run();
        eprintln!("{:?}", res);
    }
}
