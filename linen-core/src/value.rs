//! Runtime values for the Linen interpreter.

use crate::ast::Literal;
use crate::core_lang::CoreExpr;
use std::collections::HashMap;

/// A runtime value.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Literal(Literal),
    Tuple(Vec<Value>),
    Record(HashMap<String, Value>),
    Variant(String, Vec<Value>),
    Closure {
        params: Vec<String>,
        body: CoreExpr,
        env: HashMap<String, Value>,
    },
    NativeFn(fn(&[Value]) -> Value),
    // Placeholders for DSP values (to be expanded)
    Signal(SignalValue),
    Event(EventValue),
}

impl Value {
    pub fn from_literal(lit: Literal) -> Self {
        Value::Literal(lit)
    }

    pub fn int(n: i64) -> Self {
        Value::Literal(Literal::Int(n))
    }

    pub fn float(x: f64) -> Self {
        Value::Literal(Literal::Float(x))
    }

    pub fn bool(b: bool) -> Self {
        Value::Literal(Literal::Bool(b))
    }

    pub fn string(s: String) -> Self {
        Value::Literal(Literal::String(s))
    }

    pub fn unit() -> Self {
        Value::Literal(Literal::Unit)
    }
}

/// Placeholder for signal values.
#[derive(Debug, Clone, PartialEq)]
pub struct SignalValue {
    pub description: String,
}

/// Placeholder for event stream values.
#[derive(Debug, Clone, PartialEq)]
pub struct EventValue {
    pub description: String,
}
