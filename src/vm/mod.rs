use chunk::{debug::disassemble_instruction, Chunk, OpCode};
use std::env;

use crate::compiler::Compiler;

pub mod chunk;

macro_rules! binary_op {
  ($self:ident, $op:tt) => {{
      let b = $self.pop();
      let a = $self.pop();
      $self.push(a $op b);
  }};
}

pub struct VM {
    chunk: Option<Chunk>,
    ip: Vec<u8>,
    stack: Vec<f64>,
    stack_top: usize,
}

#[derive(PartialEq)]
pub enum InterpretResult {
    Ok,
    CompileError,
    RuntimeError,
}

impl Default for VM {
    fn default() -> Self {
        VM::new()
    }
}

impl VM {
    pub fn new() -> Self {
        VM {
            chunk: None,
            ip: Vec::new(),
            stack: Vec::new(),
            stack_top: 0,
        }
    }

    pub fn init_vm(&mut self) {
        self.stack = Vec::new();
        self.stack_top = 0;
    }

    pub fn interpret(&mut self, source: String) -> InterpretResult {
        let mut compiler = Compiler::new(&source);
        let mut chunk = Chunk::new();

        if !compiler.compile(&mut chunk) {
            return InterpretResult::CompileError;
        }

        self.chunk = Some(chunk.clone());
        self.ip = self.chunk.as_ref().unwrap().code.clone();

        let result = self.run();

        chunk.free_chunk();

        result
    }

    // pub fn interpret(&mut self, chunk: &chunk::Chunk) -> InterpretResult {
    //     self.chunk = Some(chunk.clone());
    //     self.ip = self.chunk.as_ref().unwrap().code.clone();

    //     self.run()
    // }

    fn run(&mut self) -> InterpretResult {
        loop {
            if env::var("DEBUG_TRACE_EXECUTION").is_ok() {
                print!("          ");
                for i in 0..self.stack_top {
                    print!("[ {:>4.*} ] ", 4, self.stack[i]);
                }
                println!();
                disassemble_instruction(&self.chunk.clone().unwrap(), self.stack_top);
            }

            if self.ip.is_empty() {
                return InterpretResult::Ok;
            }

            let instruction = self.read_byte();

            match OpCode::from(instruction) {
                OpCode::OpConstant => {
                    let constant = self.read_constant();
                    self.push(constant);
                }
                OpCode::OpAdd => {
                    binary_op!(self, +);
                }
                OpCode::OpSubtract => {
                    binary_op!(self, -);
                }
                OpCode::OpMultiply => {
                    binary_op!(self, *);
                }
                OpCode::OpDivide => {
                    binary_op!(self, /);
                }
                OpCode::OpNegate => {
                    let value = self.pop();
                    self.push(-value);
                }
                OpCode::OpReturn => {
                    println!("{}", self.pop());
                    return InterpretResult::Ok;
                }
                _ => panic!("Unknown opcode {}", instruction),
            }
        }
    }

    fn read_byte(&mut self) -> u8 {
        let byte = self.ip[0];
        if self.ip.len() == 1 {
            self.ip = Vec::new();
        } else {
            self.ip = self.ip[1..].to_vec();
        }

        byte
    }

    fn push(&mut self, value: f64) {
        self.stack.push(value);
        self.stack_top += 1;
    }

    fn pop(&mut self) -> f64 {
        self.stack_top -= 1;
        self.stack.pop().unwrap()
    }

    fn read_constant(&mut self) -> f64 {
        let constant = self.read_byte();
        self.chunk.as_ref().unwrap().constants.values[constant as usize]
    }

    pub fn free_vm(&mut self) {}
}
