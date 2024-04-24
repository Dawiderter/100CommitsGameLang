use std::fmt::Display;
use std::ops::Range;

use log::trace;

use super::chunk::CodeChunk;
use super::object::{HeapError, ObjectHeap};
use super::opcodes::*;
use super::value::{Value, ValueError};

#[derive(Debug)]
pub struct VM<'code, 'heap> {
    code: &'code CodeChunk,
    heap: &'heap mut ObjectHeap,
    stack: Stack,
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
    ConstantNotIdentifier,
    EmptyStack,
    HeapError(HeapError),
    ValueError(ValueError),
}

impl<'code, 'heap> VM<'code, 'heap> {
    pub fn init(code: &'code CodeChunk, heap: &'heap mut ObjectHeap) -> Self {
        Self {
            code,
            stack: Stack::with_capacity(256),
            heap,
            pc: 0,
        }
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

    pub fn current_span(&self) -> Range<usize> {
        self.code.find_span_of(self.pc - 1).1.clone()
    }

    fn step(&mut self) -> Result<RuntimeStep, RuntimeError> {
        macro_rules! bin_op {
            ($op:ident) => {{
                let b = self.stack.pop()?;
                let a = self.stack.pop()?;
                let value = a.$op(&b, self.heap)?;
                self.stack.push(value);
            }};
        }

        macro_rules! un_op {
            ($op:ident) => {{
                let value = self.stack.pop()?.$op(self.heap)?;
                self.stack.push(value);
            }};
        }
        
        {
            use owo_colors::OwoColorize;

            trace!(
                "{:12} L:{} M:{}{} S:{}",
                "",
                self.heap.live_count().blue().bold(),
                self.heap.dynamic_memory_used().blue().bold(),
                "B".blue().bold(),
                self.stack.print_stack_with_heap(self.heap)
            );
            trace!(
                "{}",
                self.code.dissasemble().at(self.pc).with_heap(self.heap)
            );
        }

        let op = self.read_u8()?;

        match op {
            OP_RETURN => {
                return Ok(RuntimeStep::Halt);
            }
            OP_PRINT => {
                let value = self.stack.pop()?;
                println!("{}", value.print_with_heap(self.heap));
            }
            OP_CONSTANT => {
                let value = self.read_constant()?;
                self.stack.push(value);
            }
            OP_POP => {
                self.stack.pop()?;
            }
            OP_DEF_GLOBAL => {
                let ident_value = self.read_constant()?;
                let Value::Object(ident) = ident_value else { return Err(RuntimeError::ConstantNotIdentifier) };
                let variable = self.stack.pop()?;
                self.heap.put_as_global(ident, variable);
            }
            OP_GET_GLOBAL => {
                let ident_value = self.read_constant()?;
                let Value::Object(ident) = ident_value else { return Err(RuntimeError::ConstantNotIdentifier) };
                let val = self.heap.get_global(ident)?;
                self.stack.push(val);
            }
            OP_SET_GLOBAL => {
                let ident_value = self.read_constant()?;
                let Value::Object(ident) = ident_value else { return Err(RuntimeError::ConstantNotIdentifier) };
                self.heap.get_global(ident)?;
                self.heap.put_as_global(ident, *self.stack.peek(0)?);
            }
            OP_GET_LOCAL => {
                let idx = self.read_u8()?;
                let local = self.stack.get_at(idx as usize)?;
                self.stack.push(*local);
            }
            OP_SET_LOCAL => {
                let idx = self.read_u8()?;
                let set = self.stack.peek(0)?;
                self.stack.set_at(idx as usize, *set)?;
            }
            OP_JUMP => {
                let pos = self.read_u16()?;
                self.pc += pos as usize;
            }
            OP_JUMP_F => {
                let pos = self.read_u16()?;
                let value = self.stack.peek(0)?;
                if value.is_falsey() {
                    self.pc += pos as usize;
                }
            }
            OP_TRUE => self.stack.push(Value::Bool(true)),
            OP_FALSE => self.stack.push(Value::Bool(false)),
            OP_NIL => self.stack.push(Value::Nil),
            OP_NEG => un_op!(neg),
            OP_NOT => un_op!(not),
            OP_AND => bin_op!(and),
            OP_OR => bin_op!(or),
            OP_ADD => bin_op!(add),
            OP_SUB => bin_op!(sub),
            OP_MUL => bin_op!(mul),
            OP_DIV => bin_op!(div),
            OP_EQUAL => bin_op!(equal),
            OP_LESS => bin_op!(less),
            OP_GREATER => bin_op!(greater),
            _ => return Err(RuntimeError::UnknownCode),
        }

        Ok(RuntimeStep::KeepGoing)
    }

    fn read_u8(&mut self) -> Result<u8, RuntimeError> {
        self.pc += 1;
        self.code
            .get_byte(self.pc - 1)
            .ok_or(RuntimeError::UnexpectedEnd)
    }

    fn read_u16(&mut self) -> Result<u16, RuntimeError> {
        self.pc += 2;
        let big = self.code
            .get_byte(self.pc - 2)
            .ok_or(RuntimeError::UnexpectedEnd)?;
        let little = self.code
            .get_byte(self.pc - 1)
            .ok_or(RuntimeError::UnexpectedEnd)?;
        Ok(u16::from_be_bytes([big, little]))
    }

    fn read_constant(&mut self) -> Result<Value, RuntimeError> {
        let constant_offset = self.read_u8()?;
        self.code
            .get_constant(constant_offset as usize)
            .ok_or(RuntimeError::ConstantNotFound)
            .copied()
    }
}

#[derive(Debug, Clone)]
struct Stack {
    stack: Vec<Value>,
}

impl Stack {
    fn with_capacity(capacity: usize) -> Self {
        Self {
            stack: Vec::with_capacity(capacity),
        }
    }

    fn pop(&mut self) -> Result<Value, RuntimeError> {
        self.stack.pop().ok_or(RuntimeError::EmptyStack)
    }

    fn peek(&self, dist: usize) -> Result<&Value, RuntimeError> {
        self.stack
            .get(self.stack.len() - 1 - dist)
            .ok_or(RuntimeError::EmptyStack)
    }

    fn get_at(&self, idx: usize) -> Result<&Value, RuntimeError> {
        self.stack.get(idx).ok_or(RuntimeError::EmptyStack)
    }

    fn set_at(&mut self, idx: usize, value: Value) -> Result<(), RuntimeError> {
        *self.stack.get_mut(idx).ok_or(RuntimeError::EmptyStack)? = value;
        Ok(())
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    fn print_stack_with_heap<'stack, 'heap>(
        &'stack self,
        heap: &'heap ObjectHeap,
    ) -> StackPrinter<'stack, 'heap> {
        StackPrinter { stack: self, heap }
    }
}

impl From<HeapError> for RuntimeError {
    fn from(value: HeapError) -> Self {
        Self::HeapError(value)
    }
}

impl From<ValueError> for RuntimeError {
    fn from(value: ValueError) -> Self {
        match value {
            ValueError::HeapError(h) => Self::HeapError(h),
            v => Self::ValueError(v)
        }
    }
}

#[derive(Debug)]
pub struct StackPrinter<'stack, 'heap> {
    stack: &'stack Stack,
    heap: &'heap ObjectHeap,
}

impl<'stack, 'heap> StackPrinter<'stack, 'heap> {
    fn write_stack(&self, f: &mut impl std::fmt::Write) -> std::fmt::Result {
        use owo_colors::OwoColorize;

        write!(f, "[")?;
        let mut stack_iter = self.stack.stack.iter();
        if let Some(first_val) = stack_iter.next() {
            write!(f, "'{}'", first_val.print_with_heap(self.heap).blue())?;
        }
        for val in stack_iter {
            write!(f, ", '{}'", val.print_with_heap(self.heap).blue())?;
        }
        write!(f, "]")?;
        Ok(())
    }
}

impl<'stack, 'heap> Display for StackPrinter<'stack, 'heap> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.write_stack(f)
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

        let mut heap = ObjectHeap::new();
        let mut vm = VM::init(&chunk, &mut heap);

        let res = vm.run();
        eprintln!("{:?}", res);
    }
}
