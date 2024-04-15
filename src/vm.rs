pub struct CodeChunk {
    code: Vec<u8>
}

impl CodeChunk {
    pub fn push_code(&mut self, code: u8) {
        self.code.push(code);
    }
}

pub const OP_RETURN : u8 = 0;

