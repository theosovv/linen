use std::fmt;

use super::object::{Obj, Object, ObjectType, StringObject};

#[derive(Clone, Debug, PartialEq)]
pub enum ValueType {
    Boolean(bool),
    Nil,
    Number(f64),
    Object(Object),
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
    object: Object,
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value_type.clone() {
            ValueType::Boolean(value) => write!(f, "{}", value),
            ValueType::Nil => write!(f, "nil"),
            ValueType::Number(value) => write!(f, "{}", value),
            ValueType::Object(value) => match value.object_type {
                ObjectType::String(value) => write!(f, "{}", value),
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
                object: Object {
                    object_type: ObjectType::String(value.to_string()),
                },
            },
        }
    }

    pub fn object(value: Obj) -> Self {
        match value {
            Obj::String(value) => Val {
                value_type: ValueType::Object(value.object.clone()),
                value_as: ValueAs {
                    boolean: false,
                    number: 0.0,
                    object: value.object,
                },
            },
        }
    }

    pub fn nil() -> Self {
        Val {
            value_type: ValueType::Nil,
            value_as: ValueAs {
                boolean: false,
                number: 0.0,
                object: Object {
                    object_type: ObjectType::String("nil".to_string()),
                },
            },
        }
    }

    pub fn number(value: f64) -> Self {
        Val {
            value_type: ValueType::Number(value),
            value_as: ValueAs {
                boolean: false,
                number: value,
                object: Object {
                    object_type: ObjectType::String(value.to_string()),
                },
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

    pub fn as_object(&self) -> ObjectType {
        match self.value_type.clone() {
            ValueType::Object(value) => match value.object_type {
                ObjectType::String(value) => ObjectType::String(value),
            },
            _ => panic!("Value is not an object"),
        }
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self.value_type, ValueType::Boolean(_))
    }

    pub fn is_truthy(&self) -> bool {
        match self.value_type.clone() {
            ValueType::Boolean(value) => value,
            ValueType::Number(value) => value != 0.0,
            ValueType::Nil => false,
            ValueType::Object(value) => match value.object_type {
                ObjectType::String(value) => !value.is_empty(),
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
        let value = match self.value_type.clone() {
            ValueType::Object(value) => match value.object_type {
                ObjectType::String(value) => value,
            },
            _ => panic!("Value is not a string"),
        };

        StringObject::new(&value)
    }

    pub fn as_string(&self) -> String {
        match self.value_type.clone() {
            ValueType::Object(value) => match value.object_type {
                ObjectType::String(value) => value,
            },
            _ => panic!("Value is not a string"),
        }
    }
}

#[derive(Clone, Debug)]
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
