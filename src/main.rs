use std::env;

use linen::vm::{
    chunk::{debug::disassemble_chunk, Chunk, OpCode},
    VM,
};

fn main() -> Result<(), String> {
    let mut vm = VM::new();

    vm.init_vm();

    let mut chunk = Chunk::new();

    let a = chunk.add_constant(3.4);
    chunk.write(OpCode::OpConstant as u8, 123);
    chunk.write(a as u8, 123);

    let b = chunk.add_constant(5.6);
    chunk.write(OpCode::OpConstant as u8, 123);
    chunk.write(b as u8, 123);

    chunk.write(OpCode::OpAdd as u8, 123);

    chunk.write(OpCode::OpReturn as u8, 123);

    disassemble_chunk(&chunk, "test chunk");
    vm.interpret(&chunk);

    vm.free_vm();
    chunk.free_chunk();

    Ok(())
}
