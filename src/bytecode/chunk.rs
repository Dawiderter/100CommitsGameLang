use std::{fmt::Display, ops::Range};

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
}

impl Default for CodeChunk {
    fn default() -> Self {
        Self::new()
    }
}

// ===== Disassembling

impl CodeChunk {

    pub fn dissasemble_at(&self, offset: usize) -> LocalDissasembler<'_> {
        LocalDissasembler { chunk: self, offset }
    }

    #[rustfmt::skip]
    fn dissasemble_instruction(&self, f: &mut impl std::fmt::Write, offset: usize) -> Result<usize, std::fmt::Error> {
        use owo_colors::OwoColorize;

        let instr = self.code[offset];
        let (span_code_offset, span) = self.find_span_of(offset);
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

        let constant = self.code[offset];
        let constant_value = &self.constants[constant as usize];
        write!(f, " {:>3} '{}'", constant.green(), constant_value.green().bold())
    }

    fn dissasemble_chunk(&self, f: &mut impl std::fmt::Write) -> Result<(), std::fmt::Error> {
        let mut offset = 0;
        while offset < self.code.len() {
            let len = self.dissasemble_instruction(f, offset)?;
            writeln!(f)?;
            offset += len;
        }
    
        Ok(())
    }

    fn find_span_offset_of(&self, offset: usize) -> usize {
        self.span_info.partition_point(|&(i,_)| i <= offset)
    }

    fn find_span_of(&self, offset: usize) -> &(usize, Range<usize>) {
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
        self.dissasemble_chunk(f)
    }
}

#[derive(Debug)]
pub struct LocalDissasembler<'code> {
    chunk: &'code CodeChunk,
    offset: usize,
}

impl<'code> Display for LocalDissasembler<'code> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.chunk.dissasemble_instruction(f, self.offset)?;
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