use crate::vm::chunk::OpCode;

use super::Chunk;

pub fn disassemble_chunk(chunk: &Chunk, name: &str) {
    println!("== {} ==", name);

    let mut offset = 0;
    while offset < chunk.code.len() {
        offset = disassemble_instruction(chunk, offset);
    }
}

pub fn disassemble_instruction(chunk: &Chunk, offset: usize) -> usize {
    print!("{:04} ", offset);

    if offset > 0 && chunk.lines[offset] == chunk.lines[offset - 1] {
        print!("   | ");
    } else {
        print!("{:4} ", chunk.lines[offset]);
    }

    let instruction = chunk.code[offset];
    match OpCode::from(instruction) {
        OpCode::OpReturn => simple_instruction("OP_RETURN", offset),
        OpCode::OpNegate => simple_instruction("OP_NEGATE", offset),
        OpCode::OpConstant => {
            let constant = chunk.code[offset + 1];
            print!("{:16} {:4}", "OP_CONSTANT", constant);
            println!(" '{}'", chunk.constants.values[constant as usize]);
            offset + 2
        }
        OpCode::OpDefineGlobal => {
            let constant = chunk.code[offset + 1];
            print!("{:16} {:4}", "OP_DEFINE_GLOBAL", constant);
            println!(" '{}'", chunk.constants.values[constant as usize]);
            offset + 2
        }
        OpCode::OpGetGlobal => {
            let constant = chunk.code[offset + 1];
            print!("{:16} {:4}", "OP_GET_GLOBAL", constant);
            println!(" '{}'", chunk.constants.values[constant as usize]);
            offset + 2
        }
        OpCode::OpSetGlobal => {
            let constant = chunk.code[offset + 1];
            print!("{:16} {:4}", "OP_SET_GLOBAL", constant);
            println!(" '{}'", chunk.constants.values[constant as usize]);
            offset + 2
        }
        OpCode::OpPop => simple_instruction("OP_POP", offset),
        OpCode::OpPrint => simple_instruction("OP_PRINT", offset),
        OpCode::OpNil => simple_instruction("OP_NIL", offset),
        OpCode::OpTrue => simple_instruction("OP_TRUE", offset),
        OpCode::OpFalse => simple_instruction("OP_FALSE", offset),
        OpCode::OpEqual => simple_instruction("OP_EQUAL", offset),
        OpCode::OpGreater => simple_instruction("OP_GREATER", offset),
        OpCode::OpLess => simple_instruction("OP_LESS", offset),
        OpCode::OpAdd => simple_instruction("OP_ADD", offset),
        OpCode::OpSubtract => simple_instruction("OP_SUBTRACT", offset),
        OpCode::OpMultiply => simple_instruction("OP_MULTIPLY", offset),
        OpCode::OpDivide => simple_instruction("OP_DIVIDE", offset),
        OpCode::OpNot => simple_instruction("OP_NOT", offset),
        _ => {
            println!("Unknown opcode {}", instruction);
            offset + 1
        }
    }
}

fn simple_instruction(name: &str, offset: usize) -> usize {
    println!("{}", name);
    offset + 1
}
