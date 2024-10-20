use std::fmt;

use super::object::{Obj, StringObject};

#[derive(Clone, Debug, PartialEq)]
pub enum ValueType {
    Boolean(bool),
    Nil,
    Number(f64),
    Object(Obj),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Val {
    pub value_type: ValueType,
    value_as: ValueAs,
}

#[derive(Clone, Debug, PartialEq)]
struct ValueAs {
    boolean: bool,
    number: f64,
    object: Obj,
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value_type.clone() {
            ValueType::Boolean(value) => write!(f, "{}", value),
            ValueType::Nil => write!(f, "nil"),
            ValueType::Number(value) => write!(f, "{}", value),
            ValueType::Object(object) => match object {
                Obj::String(value) => write!(f, "{}", value.as_str()),
                Obj::Function(function) => write!(f, "<{}>", function.name.as_str()),
                Obj::Native(function) => write!(f, "<native {}>", function.name.as_str()),
            },
        }
    }
}

impl Val {
    pub fn boolean(value: bool) -> Self {
        Val {
            value_type: ValueType::Boolean(value),
            value_as: ValueAs {
                boolean: value,
                number: 0.0,
                object: Obj::String(StringObject::new("false")),
            },
        }
    }

    pub fn object(value: Obj) -> Self {
        match value.clone() {
            Obj::String(_) => Val {
                value_as: ValueAs {
                    boolean: false,
                    number: 0.0,
                    object: value.clone(),
                },
                value_type: ValueType::Object(value),
            },
            Obj::Function(_) => Val {
                value_as: ValueAs {
                    boolean: false,
                    number: 0.0,
                    object: value.clone(),
                },
                value_type: ValueType::Object(value),
            },
            Obj::Native(_) => Val {
                value_as: ValueAs {
                    boolean: false,
                    number: 0.0,
                    object: value.clone(),
                },
                value_type: ValueType::Object(value),
            },
        }
    }

    pub fn nil() -> Self {
        Val {
            value_type: ValueType::Nil,
            value_as: ValueAs {
                boolean: false,
                number: 0.0,
                object: Obj::String(StringObject::new("nil")),
            },
        }
    }

    pub fn is_falsey(&self) -> bool {
        match self.value_type.clone() {
            ValueType::Boolean(value) => !value,
            ValueType::Nil => true,
            ValueType::Number(value) => value == 0.0,
            ValueType::Object(value) => match value {
                Obj::String(value) => value.is_empty(),
                Obj::Function(function) => function.chunk.constants.values.is_empty(),
                Obj::Native(_) => false,
            },
        }
    }

    pub fn number(value: f64) -> Self {
        Val {
            value_type: ValueType::Number(value),
            value_as: ValueAs {
                boolean: false,
                number: value,
                object: Obj::String(StringObject::new("0.0")),
            },
        }
    }

    pub fn as_bool(&self) -> bool {
        match self.value_type {
            ValueType::Boolean(value) => value,
            ValueType::Number(value) => value != 0.0,
            _ => panic!("Value is not a boolean"),
        }
    }

    pub fn as_number(&self) -> f64 {
        match self.value_type.clone() {
            ValueType::Boolean(value) => value as i64 as f64,
            ValueType::Number(value) => value,
            _ => panic!("Value is not a number"),
        }
    }

    pub fn as_object(&self) -> Obj {
        self.value_as.object.clone()
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self.value_type, ValueType::Boolean(_))
    }

    pub fn is_truthy(&self) -> bool {
        match self.value_type.clone() {
            ValueType::Boolean(value) => value,
            ValueType::Number(value) => value != 0.0,
            ValueType::Nil => false,
            ValueType::Object(obj) => match obj {
                Obj::String(value) => !value.is_empty(),
                Obj::Function(function) => !function.chunk.constants.values.is_empty(),
                Obj::Native(_) => true,
            },
        }
    }

    pub fn is_object(&self) -> bool {
        matches!(self.value_type, ValueType::Object(_))
    }

    pub fn is_number(&self) -> bool {
        matches!(self.value_type, ValueType::Number(_))
    }

    pub fn is_nil(&self) -> bool {
        matches!(self.value_type, ValueType::Nil)
    }

    pub fn as_object_string(&self) -> StringObject {
        match self.value_type.clone() {
            ValueType::Object(value) => match value {
                Obj::String(value) => value,
                Obj::Function(function) => function.name,
                Obj::Native(_) => panic!("Value is not a string"),
            },
            _ => panic!("Value is not a string"),
        }
    }

    pub fn as_string(&self) -> String {
        match self.value_type.clone() {
            ValueType::Object(value) => match value {
                Obj::String(value) => value.as_str(),
                Obj::Function(function) => function.name.as_str(),
                Obj::Native(_) => panic!("Value is not a string"),
            },
            _ => panic!("Value is not a string"),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Value {
    pub values: Vec<Val>,
}

impl Default for Value {
    fn default() -> Self {
        Self::new()
    }
}

impl Value {
    pub fn new() -> Self {
        Self { values: Vec::new() }
    }

    pub fn write(&mut self, value: Val) {
        self.values.push(value);
    }

    pub fn free_value(&mut self) {
        self.values.clear();
    }
}
