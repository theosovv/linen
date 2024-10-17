use chunk::{
    debug::disassemble_instruction,
    object::{Obj, Object, ObjectType, StringObject},
    table::Table,
    value::{Val, ValueType},
    Chunk, OpCode,
};
use std::env;

use crate::compiler::Compiler;

pub mod chunk;

macro_rules! binary_op {
    ($self:expr, $operator:tt, $result_variant:ident) => {{
        let b = $self.pop();
        let a = $self.pop();

        match (&a.value_type, &b.value_type) {
            (ValueType::Number(_), ValueType::Number(_)) => {
                let a_num = a.as_number();
                let b_num = b.as_number();
                let result = a_num $operator b_num;
                $self.push(Val::$result_variant(result));
            }
            (ValueType::Boolean(_), ValueType::Boolean(_)) => {
                let a_bool = a.as_bool();
                let b_bool = b.as_bool();
                let result = match stringify!($operator) {
                    "&&" => a_bool && b_bool,
                    "||" => a_bool || b_bool,
                    "==" => a_bool == b_bool,
                    "!=" => a_bool != b_bool,
                    _ => {
                        $self.runtime_error("Invalid operator for boolean values.");
                        return InterpretResult::RuntimeError;
                    }
                };
                $self.push(Val::boolean(result));
            }
            (ValueType::Object(_), ValueType::Object(_)) => {
                let a_obj = a.as_string();
                let b_obj = b.as_string();
                let result = match stringify!($operator) {
                    "+" => a_obj + b_obj.as_str(),
                    _ => {
                        $self.runtime_error("Invalid operator for object values.");
                        return InterpretResult::RuntimeError;
                    }
                };
                $self.push(Val::object(Obj::String(StringObject::new(result.as_str()))));
            }
            _ => {
                $self.runtime_error("Operands must be two numbers or two booleans.");
                return InterpretResult::RuntimeError;
            }
        }
    }};
}

pub struct VM {
    chunk: Option<Chunk>,
    ip: Vec<u8>,
    stack: Vec<Val>,
    stack_top: usize,
    objects: Option<Vec<Object>>,
    table: Table,
    globals: Table,
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
            objects: None,
            table: Table::new(),
            globals: Table::new(),
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

    fn run(&mut self) -> InterpretResult {
        loop {
            if self.ip.is_empty() {
                return InterpretResult::Ok;
            }

            let instruction = self.read_byte();

            match OpCode::from(instruction) {
                OpCode::OpConstant => {
                    let constant = self.read_constant();
                    self.push(constant);
                }
                OpCode::OpTrue => self.push(Val::boolean(true)),
                OpCode::OpFalse => self.push(Val::boolean(false)),
                OpCode::OpPop => {
                    self.pop();
                }
                OpCode::OpGetGlobal => {
                    let name = self.read_string();
                    let value = self.globals.table_get(&name);

                    if value.is_none() {
                        self.runtime_error("Undefined variable.");
                        return InterpretResult::RuntimeError;
                    }

                    self.push(value.unwrap());
                }
                OpCode::OpSetGlobal => {
                    let name = self.read_string();
                    if self.globals.set_table(name.clone(), self.peek(0)) {
                        self.globals.table_delete(&name);

                        self.runtime_error("Undefined variable.");
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::OpEqual => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(self.values_equal(a, b));
                }
                OpCode::OpGreater => {
                    binary_op!(self, >, boolean);
                }
                OpCode::OpLess => {
                    binary_op!(self, <, boolean);
                }
                OpCode::OpNil => self.push(Val::nil()),
                OpCode::OpAdd => {
                    binary_op!(self, +, number);
                }
                OpCode::OpSubtract => {
                    binary_op!(self, -, number);
                }
                OpCode::OpMultiply => {
                    binary_op!(self, *, number);
                }
                OpCode::OpDivide => {
                    binary_op!(self, /, number);
                }
                OpCode::OpNot => {
                    let value = self.pop().is_truthy();
                    self.push(Val::boolean(!value));
                }
                OpCode::OpDefineGlobal => {
                    let name = self.read_string();
                    self.globals.set_table(name, self.peek(0));
                    self.pop();
                }
                OpCode::OpNegate => {
                    let val = self.peek(0);

                    if !val.is_number() {
                        self.runtime_error("Operand must be a number.");
                        return InterpretResult::RuntimeError;
                    }

                    let value = self.pop();
                    self.push(Val::number(-value.as_number()));
                }
                OpCode::OpPrint => {
                    println!("{}", self.pop());
                }
                _ => panic!("Unknown opcode {}", instruction),
            }
        }
    }

    fn read_string(&mut self) -> StringObject {
        let constant = self.read_constant();

        constant.as_object_string()
    }

    fn values_equal(&self, a: Val, b: Val) -> Val {
        if a.value_type != b.value_type {
            return Val::boolean(false);
        }

        match &a.value_type {
            ValueType::Number(_) => Val::boolean(a.as_number() == b.as_number()),
            ValueType::Boolean(_) => Val::boolean(a.as_bool() == b.as_bool()),
            ValueType::Object(value) => match value.object_type {
                ObjectType::String(_) => Val::boolean(a.as_string() == b.as_string()),
                _ => panic!("Unknown object type"),
            },
            ValueType::Nil => Val::boolean(true),
        }
    }

    fn peek(&self, distance: usize) -> Val {
        self.stack[self.stack_top - 1 - distance].clone()
    }

    fn runtime_error(&mut self, message: &str) {
        eprintln!("{}", message);
        self.free_vm();
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

    fn push(&mut self, value: Val) {
        self.stack.push(value);
        self.stack_top += 1;
    }

    fn pop(&mut self) -> Val {
        self.stack_top -= 1;
        self.stack.pop().unwrap()
    }

    fn read_constant(&mut self) -> Val {
        let index = self.read_byte() as usize;
        self.chunk.as_ref().unwrap().constants.values[index].clone()
    }

    pub fn free_vm(&mut self) {
        self.table.free_table();
        self.globals.free_table();
        self.objects = None;
    }
}
