use value::{Val, Value};

pub mod debug;
pub mod memory;
pub mod object;
pub mod table;
pub mod value;

pub enum OpCode {
    OpConstant,
    OpNil,
    OpTrue,
    OpFalse,
    OpEqual,
    OpGreater,
    OpLess,
    OpAdd,
    OpSubtract,
    OpMultiply,
    OpDivide,
    OpNot,
    OpNegate,
    OpReturn,
    OpPrint,
    OpPop,
    OpDefineGlobal,
    OpGetGlobal,
    OpSetGlobal,
    OpGetLocal,
    OpSetLocal,
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
