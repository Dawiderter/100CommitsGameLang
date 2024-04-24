use std::{fmt::Display, ops::Range};

use super::object::ObjectHeap;
use super::opcodes::*;

use super::value::Value;

/// An executable chunk of code
#[derive(Debug, Clone)]
pub struct CodeChunk {
    code: Vec<u8>,
    constants: Vec<Value>,
    span_info: Vec<(usize, Range<usize>)>
}

// ===== Public interface
impl CodeChunk {
    pub fn new() -> Self {
        Self { code: Vec::new(), constants: Vec::new(), span_info: vec![(0, 0..0)] }
    }

    pub fn push_code(&mut self, code: u8) {
        self.code.push(code);
    }

    pub fn push_constant(&mut self, constant: Value) -> u8 {
        self.constants.push(constant);
        (self.constants.len() - 1).try_into().expect("Exceeded maximum numbers of constants in a pool (256)")
    }

    pub fn push_span_info(&mut self, span: Range<usize>) {
        self.span_info.push((self.code.len(), span))
    }

    pub fn get_byte(&self, offset: usize) -> Option<u8> {
        self.code.get(offset).copied()
    }

    pub fn get_constant(&self, constant: usize) -> Option<&Value> {
        self.constants.get(constant)
    }

    pub fn patch(&mut self, offset : usize, code: u8) {
        self.code[offset] = code;
    }

    pub fn size(&self) -> usize {
        self.code.len()
    }

}

impl Default for CodeChunk {
    fn default() -> Self {
        Self::new()
    }
}

// ===== Disassembling

impl CodeChunk {
    pub fn dissasemble(&self) -> Dissasembler<'_,'_> {
        Dissasembler { chunk: self, offset: None, heap: None }
    }

    fn find_span_offset_of(&self, offset: usize) -> usize {
        self.span_info.partition_point(|&(i,_)| i <= offset)
    }

    pub fn find_span_of(&self, offset: usize) -> &(usize, Range<usize>) {
        let span_offset = self.find_span_offset_of(offset);
        &self.span_info[span_offset - 1]
    }

    // fn next_span_offset(&self, current_span: usize, current_offset: usize) -> usize {
    //     let mut i = current_span;
    //     while i < self.span_info.len() && self.span_info[i].0 <= current_offset {
    //         i += 1;
    //     }
    //     i - 1
    // }
}

impl Display for CodeChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.dissasemble())
    }
}

#[derive(Debug)]
pub struct Dissasembler<'code, 'heap> {
    chunk: &'code CodeChunk,
    offset: Option<usize>,
    heap: Option<&'heap ObjectHeap>
}

impl<'code, 'heap> Dissasembler<'code, 'heap> {
    pub fn at(mut self, offset: usize) -> Self {
        self.offset = Some(offset);
        self
    }
    pub fn with_heap(mut self, heap: &'heap ObjectHeap) -> Self {
        self.heap = Some(heap);
        self
    }
    #[rustfmt::skip]
    fn dissasemble_instruction(&self, f: &mut impl std::fmt::Write, offset: usize) -> Result<usize, std::fmt::Error> {
        use owo_colors::OwoColorize;

        let instr = self.chunk.code[offset];
        let (span_code_offset, span) = self.chunk.find_span_of(offset);
        write!(f, "{:04} ", offset.red())?;
        if *span_code_offset == offset {
            write!(f, "{:>3}:{:<3} ", span.start, span.end)?;
        } else {
            write!(f, "{:^7} ", "|")?;
        }
        let len = match instr {
            OP_RETURN => { self.dissasemble_op(f, "RETURN")?; 1 }
            OP_CONSTANT => { self.dissasemble_op(f, "CONSTANT")?; self.dissasemble_constant(f, offset + 1)?; 2 }
            OP_NEG => { self.dissasemble_op(f, "NEG")?; 1 }
            OP_ADD => { self.dissasemble_op(f, "ADD")?; 1 }
            OP_SUB => { self.dissasemble_op(f, "SUB")?; 1 }
            OP_MUL => { self.dissasemble_op(f, "MUL")?; 1 }
            OP_DIV => { self.dissasemble_op(f, "DIV")?; 1 }
            OP_NOT => { self.dissasemble_op(f, "NOT")?; 1 }
            OP_AND => { self.dissasemble_op(f, "AND")?; 1 }
            OP_OR => { self.dissasemble_op(f, "OR")?; 1 }
            OP_EQUAL => { self.dissasemble_op(f, "EQUAL")?; 1 }
            OP_GREATER => { self.dissasemble_op(f, "GREATER")?; 1 }
            OP_LESS => { self.dissasemble_op(f, "LESS")?; 1 }
            OP_TRUE => { self.dissasemble_op(f, "TRUE")?; 1 }
            OP_FALSE => { self.dissasemble_op(f, "FALSE")?; 1 }
            OP_NIL => { self.dissasemble_op(f, "NIL")?; 1 }
            OP_PRINT => { self.dissasemble_op(f, "PRINT")?; 1 }
            OP_POP => { self.dissasemble_op(f, "POP")?; 1 }
            OP_DEF_GLOBAL => { self.dissasemble_op(f, "DEF GLOBAL")?; self.dissasemble_constant(f, offset + 1)?; 2 }
            OP_GET_GLOBAL => { self.dissasemble_op(f, "GET GLOBAL")?; self.dissasemble_constant(f, offset + 1)?; 2 }
            OP_SET_GLOBAL => { self.dissasemble_op(f, "SET GLOBAL")?; self.dissasemble_constant(f, offset + 1)?; 2 }
            OP_GET_LOCAL => { self.dissasemble_op(f, "GET LOCAL")?; self.dissasemble_arg(f, offset + 1)?; 2 }
            OP_SET_LOCAL => { self.dissasemble_op(f, "SET LOCAL")?; self.dissasemble_arg(f, offset + 1)?; 2 }
            OP_JUMP => { self.dissasemble_op(f, "JUMP")?; self.dissasemble_jump_target(f, offset + 1)?; 3 }
            OP_JUMP_F => { self.dissasemble_op(f, "JUMPF")?; self.dissasemble_jump_target(f, offset + 1)?; 3 }
            _ => { self.dissasemble_op(f, "UNKNOWN")?; 1 }
        };
    
        Ok(len)
    }

    fn dissasemble_op(&self, f: &mut impl std::fmt::Write, name: &str) -> Result<(), std::fmt::Error> {
        use owo_colors::OwoColorize;

        write!(f, "{:<10}", name.bold())
    }

    fn dissasemble_constant(&self, f: &mut impl std::fmt::Write, offset: usize) -> Result<(), std::fmt::Error> {
        use owo_colors::OwoColorize;

        let constant = self.chunk.code[offset];
        let constant_value = &self.chunk.constants[constant as usize];
        match self.heap {
            Some(heap) => write!(f, " {:>3} '{}'", constant.green(), constant_value.print_with_heap(heap).green().bold()),
            None => write!(f, " {:>3} '{}'", constant.green(), constant_value.green().bold()),
        }
    }

    fn dissasemble_arg(&self, f: &mut impl std::fmt::Write, offset: usize) -> Result<(), std::fmt::Error> {
        use owo_colors::OwoColorize;
        
        let arg = self.chunk.code[offset];
        write!(f, " {:>3}", arg.green())
    }

    fn dissasemble_jump_target(&self, f: &mut impl std::fmt::Write, offset: usize) -> Result<(), std::fmt::Error> {
        use owo_colors::OwoColorize;

        let arg = u16::from_be_bytes([self.chunk.code[offset], self.chunk.code[offset+1]]);
        write!(f, " {:>3} -> {:>04}", arg.green(), (arg as usize + offset + 2).red())
    }
    
    fn dissasemble_chunk(&self, f: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error> {
        let mut offset = 0;
        while offset < self.chunk.code.len() {
            let len = self.dissasemble_instruction(f, offset)?;
            writeln!(f)?;
            offset += len;
        }
    
        Ok(())
    }
}

impl<'code,'heap> Display for Dissasembler<'code, 'heap> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.offset {
            Some(offset) => { self.dissasemble_instruction(f, offset)?; },
            None => { self.dissasemble_chunk(f)?; },
        };
        Ok(())
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
        eprintln!("{chunk}");
    }
}