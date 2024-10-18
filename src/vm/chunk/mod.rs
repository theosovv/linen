use value::{Val, Value};

pub mod debug;
pub mod object;
pub mod table;
pub mod value;

pub enum OpCode {
    OpConstant = 0,
    OpNil = 1,
    OpTrue = 2,
    OpFalse = 3,
    OpEqual = 4,
    OpGreater = 5,
    OpLess = 6,
    OpAdd = 7,
    OpSubtract = 8,
    OpMultiply = 9,
    OpDivide = 10,
    OpNot = 11,
    OpNegate = 12,
    OpReturn = 13,
    OpPrint = 14,
    OpPop = 15,
    OpDefineGlobal = 16,
    OpGetGlobal = 17,
    OpSetGlobal = 18,
    OpGetLocal = 19,
    OpSetLocal = 20,
    OpJumpFalse = 21,
    OpJump = 22,
    OpLoop = 23,
    OpGreaterEqual = 24,
    OpLessEqual = 25,
}

impl From<u8> for OpCode {
    fn from(value: u8) -> Self {
        match value {
            0 => OpCode::OpConstant,
            1 => OpCode::OpNil,
            2 => OpCode::OpTrue,
            3 => OpCode::OpFalse,
            4 => OpCode::OpEqual,
            5 => OpCode::OpGreater,
            6 => OpCode::OpLess,
            7 => OpCode::OpAdd,
            8 => OpCode::OpSubtract,
            9 => OpCode::OpMultiply,
            10 => OpCode::OpDivide,
            11 => OpCode::OpNot,
            12 => OpCode::OpNegate,
            13 => OpCode::OpReturn,
            14 => OpCode::OpPrint,
            15 => OpCode::OpPop,
            16 => OpCode::OpDefineGlobal,
            17 => OpCode::OpGetGlobal,
            18 => OpCode::OpSetGlobal,
            19 => OpCode::OpGetLocal,
            20 => OpCode::OpSetLocal,
            21 => OpCode::OpJumpFalse,
            22 => OpCode::OpJump,
            23 => OpCode::OpLoop,
            24 => OpCode::OpGreaterEqual,
            25 => OpCode::OpLessEqual,
            _ => panic!("Unknown opcode {}", value),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Chunk {
    pub code: Vec<u8>,
    pub lines: Vec<usize>,
    pub constants: Value,
}

impl Default for Chunk {
    fn default() -> Self {
        Self::new()
    }
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            code: Vec::new(),
            constants: Value::new(),
            lines: Vec::new(),
        }
    }

    pub fn write(&mut self, byte: u8, line: usize) {
        self.code.push(byte);
        self.lines.push(line);
    }

    pub fn add_constant(&mut self, value: Val) -> usize {
        self.constants.write(value);
        self.constants.values.len() - 1
    }

    pub fn free_chunk(&mut self) {
        self.constants.free_value();
        self.code.clear();
    }
}
