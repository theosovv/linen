//! Core library for the Linen programming language.
//!
//! Provides AST definitions, type system, error handling, and source location tracking.

pub mod ast;
pub mod core_lang;
pub mod desugar;
pub mod error;
pub mod interpreter;
pub mod lowering;
pub mod signal_graph;
pub mod span;
pub mod type_checker;
pub mod types;
pub mod value;
