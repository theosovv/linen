use std::env;

use linen::{
    exec::Exec,
    vm::{
        chunk::{debug::disassemble_chunk, Chunk, OpCode},
        VM,
    },
};

fn main() -> Result<(), String> {
    let mut exec = Exec::new();

    exec.run(None);

    Ok(())
}
