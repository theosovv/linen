use chunk::{
    object::{FunctionObject, NativeObject, Obj, StringObject},
    table::Table,
    value::{Val, ValueType},
    Chunk, OpCode,
};
use native::clock_native;

use crate::compiler::{Compiler, FunctionType};

pub mod chunk;
pub mod native;

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
                println!("a - {}", a);
                println!("b - {}", b);
                $self.runtime_error("Operands must be two numbers or two booleans.");
                return InterpretResult::RuntimeError;
            }
        }
    }};
}

#[derive(Clone)]
pub struct CallFrame {
    pub function: FunctionObject,
    pub ip: usize,
    pub slot: usize,
}

pub struct VM {
    stack: Vec<Val>,
    objects: Option<Vec<Obj>>,
    table: Table,
    globals: Table,
    frames: Vec<CallFrame>,
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
            stack: Vec::new(),
            objects: None,
            table: Table::new(),
            globals: Table::new(),
            frames: Vec::new(),
        }
    }

    pub fn init_vm(&mut self) {
        self.stack = Vec::new();
        self.define_native(StringObject::new("clock"), clock_native);
    }

    pub fn interpret(&mut self, source: String) -> InterpretResult {
        let mut compiler = Compiler::new(&source, FunctionType::Script);
        let chunk = Chunk::new();

        let function = compiler.compile(chunk);

        if function.is_err() {
            return InterpretResult::CompileError;
        }

        let function = function.unwrap();

        self.push(Val::object(Obj::Function(function.clone())));

        let frame = CallFrame {
            function: function.clone(),
            ip: 0,
            slot: self.stack.len(),
        };
        self.frames.push(frame);

        self.run()
    }

    fn run(&mut self) -> InterpretResult {
        loop {
            if self.frames.is_empty() {
                return InterpretResult::RuntimeError;
            }

            let instruction = {
                let frame = self.frames.last_mut().unwrap();
                if frame.ip >= frame.function.chunk.code.len() {
                    return InterpretResult::RuntimeError;
                }
                // Считываем байт и увеличиваем ip внутри метода
                self.read_byte()
            };

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
                OpCode::OpGetLocal => {
                    let slot = self.read_byte() as usize;
                    let value = self
                        .stack
                        .get(slot + self.frames.last_mut().unwrap().slot)
                        .unwrap()
                        .clone();
                    self.push(value);
                }
                OpCode::OpSetLocal => {
                    let slot = self.read_byte() as usize;
                    let value = self.peek(0);
                    let index = self.frames.last().unwrap().slot + slot;
                    self.stack[index] = value;
                }
                OpCode::OpEqual => {
                    let b = self.pop();
                    let a = self.pop();
                    self.push(self.values_equal(a, b));
                }
                OpCode::OpGreater => {
                    binary_op!(self, >, boolean);
                }
                OpCode::OpGreaterEqual => {
                    binary_op!(self, >=, boolean);
                }
                OpCode::OpLessEqual => {
                    binary_op!(self, <=, boolean);
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
                OpCode::OpJumpFalse => {
                    let offset = self.read_short();
                    if !self.peek(0).is_truthy() {
                        self.frames.last_mut().unwrap().ip += offset as usize;
                    }
                }
                OpCode::OpLoop => {
                    let offset = self.read_short();
                    self.frames.last_mut().unwrap().ip -= offset as usize;
                }
                OpCode::OpJump => {
                    let offset = self.read_short();
                    self.frames.last_mut().unwrap().ip += offset as usize;
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
                OpCode::Call => {
                    let arg_count = self.read_byte();
                    if !self.call_value(self.peek(arg_count as usize), arg_count) {
                        return InterpretResult::RuntimeError;
                    }
                }
                OpCode::OpReturn => {
                    let result = self.pop();
                    let frame = self.frames.pop().unwrap();

                    self.stack.truncate(frame.slot - 1);

                    self.push(result);
                    if self.frames.is_empty() {
                        return InterpretResult::Ok;
                    }
                }
            }
        }
    }
    fn read_string(&mut self) -> StringObject {
        let constant = self.read_constant();

        constant.as_object_string()
    }

    fn call_value(&mut self, callee: Val, arg_count: u8) -> bool {
        match callee.value_type {
            ValueType::Object(Obj::Function(function)) => self.call(function, arg_count),
            ValueType::Object(Obj::Native(native)) => {
                let function = native.function;
                let result = function(arg_count as usize, self.pop_n(arg_count as usize));
                self.push(result);

                true
            }
            _ => {
                self.runtime_error("Can only call functions and classes.");
                false
            }
        }
    }

    fn define_native(
        &mut self,
        name: StringObject,
        function: fn(arg_count: usize, args: Vec<Val>) -> Val,
    ) {
        let native = NativeObject {
            name: name.clone(),
            function,
        };
        let value = Val::object(Obj::Native(native));
        self.globals.set_table(name, value);
    }

    fn pop_n(&mut self, count: usize) -> Vec<Val> {
        let mut values = Vec::new();
        for _ in 0..count {
            values.push(self.pop());
        }
        values
    }

    fn call(&mut self, function: FunctionObject, arg_count: u8) -> bool {
        if arg_count as usize != function.arity {
            self.runtime_error(
                format!(
                    "Expected {} arguments but got {}.",
                    function.arity, arg_count,
                )
                .as_str(),
            );
            return false;
        }

        let frame = CallFrame {
            function: function.clone(),
            ip: 0,
            slot: self.stack.len() - arg_count as usize,
        };
        self.frames.push(frame);
        true
    }

    fn values_equal(&self, a: Val, b: Val) -> Val {
        if a.value_type != b.value_type {
            return Val::boolean(false);
        }

        match &a.value_type {
            ValueType::Number(_) => Val::boolean(a.as_number() == b.as_number()),
            ValueType::Boolean(_) => Val::boolean(a.as_bool() == b.as_bool()),
            ValueType::Object(value) => match value {
                Obj::String(value) => Val::boolean(*value == b.as_object_string()),
                _ => Val::boolean(false),
            },
            ValueType::Nil => Val::boolean(true),
        }
    }

    fn peek(&self, distance: usize) -> Val {
        self.stack[self.stack.len() - distance - 1].clone()
    }

    fn runtime_error(&mut self, message: &str) {
        for frame in self.frames.iter().rev() {
            let function = &frame.function;
            let line = function.chunk.lines[frame.ip];
            print!("[line {}] in {}", line, function.name.as_str());
            println!(" -- {message}");
        }
        self.free_vm();
    }

    fn read_byte(&mut self) -> u8 {
        let frame = self.frames.last_mut().unwrap();
        let byte = frame.function.chunk.code[frame.ip];
        frame.ip += 1;
        byte
    }

    fn read_short(&mut self) -> u16 {
        let high = self.read_byte() as u16;
        let low = self.read_byte() as u16;
        (high << 8) | low
    }

    fn push(&mut self, value: Val) {
        self.stack.push(value);
    }

    fn pop(&mut self) -> Val {
        self.stack.pop().unwrap()
    }

    fn read_constant(&mut self) -> Val {
        let index = self.read_byte() as usize;
        self.frames
            .last_mut()
            .unwrap()
            .function
            .chunk
            .constants
            .values[index]
            .clone()
    }

    pub fn free_vm(&mut self) {
        self.table.free_table();
        self.globals.free_table();
        self.objects = None;
    }
}
