// ===== Opcodes
pub const OP_RETURN : u8 = 0;
pub const OP_CONSTANT : u8 = 1;
pub const OP_NEG : u8 = 2;
pub const OP_ADD : u8 = 3;
pub const OP_SUB : u8 = 4;
pub const OP_MUL : u8 = 5;
pub const OP_DIV : u8 = 6;
pub const OP_FALSE : u8 = 7;
pub const OP_TRUE : u8 = 8;
pub const OP_NOT : u8 = 9;
pub const OP_AND : u8 = 10;
pub const OP_OR : u8 = 11;
pub const OP_EQUAL : u8 = 12;
pub const OP_GREATER : u8 = 13;
pub const OP_LESS : u8 = 14;
pub const OP_NIL : u8 = 15;
pub const OP_PRINT : u8 = 16;
pub const OP_POP : u8 = 17;
pub const OP_DEF_GLOBAL : u8 = 18;
pub const OP_GET_GLOBAL : u8 = 19;
pub const OP_SET_GLOBAL : u8 = 20;
pub const OP_GET_LOCAL : u8 = 21;
pub const OP_SET_LOCAL : u8 = 22;
pub const OP_JUMP : u8 = 23;
pub const OP_JUMP_F : u8 = 24;