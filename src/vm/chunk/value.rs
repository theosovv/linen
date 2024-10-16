use std::fmt;

#[derive(Clone, Debug, PartialEq)]
pub enum ValueType {
    Boolean(bool),
    Nil,
    Number(f64),
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
}

impl fmt::Display for Val {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value_type {
            ValueType::Boolean(value) => write!(f, "{}", value),
            ValueType::Nil => write!(f, "nil"),
            ValueType::Number(value) => write!(f, "{}", value),
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
            },
        }
    }

    pub fn nil() -> Self {
        Val {
            value_type: ValueType::Nil,
            value_as: ValueAs {
                boolean: false,
                number: 0.0,
            },
        }
    }

    pub fn number(value: f64) -> Self {
        Val {
            value_type: ValueType::Number(value),
            value_as: ValueAs {
                boolean: false,
                number: value,
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
        match self.value_type {
            ValueType::Boolean(value) => value as i64 as f64,
            ValueType::Number(value) => value,
            _ => panic!("Value is not a number"),
        }
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self.value_type, ValueType::Boolean(_))
    }

    pub fn is_truthy(&self) -> bool {
        match self.value_type {
            ValueType::Boolean(value) => value,
            ValueType::Number(value) => value != 0.0,
            ValueType::Nil => false,
        }
    }

    pub fn is_number(&self) -> bool {
        matches!(self.value_type, ValueType::Number(_))
    }

    pub fn is_nil(&self) -> bool {
        matches!(self.value_type, ValueType::Nil)
    }
}

#[derive(Clone, Debug)]
pub struct Value {
    pub values: Vec<f64>,
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

    pub fn write(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn free_value(&mut self) {
        self.values.clear();
    }
}
