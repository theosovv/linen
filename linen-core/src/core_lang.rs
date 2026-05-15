//! Core Language AST — simplified intermediate representation.
//!
//! Core Lang is ANF (A-Normal Form): all intermediates are let-bound,
//! and arguments to applications are atoms (variables or literals).

use crate::span::Span;

/// Core expression.
#[derive(Debug, Clone, PartialEq)]
pub enum CoreExpr {
    Atom(Atom, Span),
    Let(String, Box<CoreExpr>, Box<CoreExpr>, Span),
    Apply(Atom, Vec<Atom>, Span),
    Lambda(Vec<String>, Box<CoreExpr>, Span),
    If(Atom, Box<CoreExpr>, Box<CoreExpr>, Span),
    Match(Atom, Vec<(CorePattern, CoreExpr)>, Span),
}

/// Atomic expression: no nested computation.
#[derive(Debug, Clone, PartialEq)]
pub enum Atom {
    Var(String),
    Literal(crate::ast::Literal),
}

/// Simplified pattern for Core Lang.
#[derive(Debug, Clone, PartialEq)]
pub enum CorePattern {
    Wildcard,
    Literal(crate::ast::Literal),
    Var(String),
    Tuple(Vec<CorePattern>),
    Variant(String, Vec<CorePattern>),
    Record(Vec<(String, CorePattern)>),
}

/// Core module.
#[derive(Debug, Clone, PartialEq)]
pub struct CoreModule {
    pub bindings: Vec<(String, CoreExpr)>,
}
