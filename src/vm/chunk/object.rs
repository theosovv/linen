#[derive(Clone, Debug, PartialEq)]
pub struct Object {
    pub object_type: ObjectType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ObjectType {
    String(String),
}

#[derive(Clone, Debug, PartialEq)]
pub struct StringObject {
    pub object: Object,
    pub length: usize,
    pub chars: Vec<char>,
    pub hash: u32,
}

pub enum Obj {
    String(StringObject),
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
            object: Object {
                object_type: ObjectType::String(value.to_string()),
            },
            length: value.len(),
            chars,
            hash: hash_string(value),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }
}
