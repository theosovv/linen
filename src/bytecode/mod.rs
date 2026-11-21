mod codegen;
mod instruction;
mod verify;

pub use codegen::codegen;
pub use instruction::{
    AudioRegister, BinaryOp, Bytecode, Instruction, Operand, ScalarRegister, StateId,
};
pub use verify::{BytecodeVerifyError, verify, verify_origins};
