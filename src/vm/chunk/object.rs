use super::{value::Val, Chunk};

#[derive(Clone, Debug, PartialEq)]
pub struct StringObject {
    pub length: usize,
    pub chars: Vec<char>,
    pub hash: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionObject {
    pub arity: usize,
    pub name: StringObject,
    pub chunk: Chunk,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Obj {
    String(StringObject),
    Function(FunctionObject),
    Native(NativeObject),
}

fn hash_string(value: &str) -> u32 {
    let mut hash: u32 = 2166136261;
    for c in value.chars() {
        hash ^= c as u32;
        hash = hash.wrapping_mul(16777619);
    }
    hash
}

impl StringObject {
    pub fn new(value: &str) -> Self {
        let mut chars = Vec::new();
        for c in value.chars() {
            chars.push(c);
        }

        StringObject {
            length: value.len(),
            chars,
            hash: hash_string(value),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn as_str(&self) -> String {
        if self.is_empty() {
            return "<main>".to_string();
        }
        self.chars.iter().collect::<String>()
    }
}

impl Default for FunctionObject {
    fn default() -> Self {
        Self::new()
    }
}

impl FunctionObject {
    pub fn new() -> Self {
        FunctionObject {
            arity: 0,
            name: StringObject::new(""),
            chunk: Chunk::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct NativeObject {
    pub name: StringObject,
    pub function: fn(arg_count: usize, args: Vec<Val>) -> Val,
}
