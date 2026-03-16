//! Linen Compiler
//!
//! This crate contains the compiler frontend for the Linen programming language:
//! - Lexer
//! - Parser
//! - Type checker (Hindley-Milner with extensions)
//! - Intermediate representations
//! - Code generation to VM bytecode

#![warn(missing_docs)]

/// Lexer module - tokenizes source code
pub mod lexer;

/// Parser module - builds AST from tokens
pub mod parser;

/// Type checker module - HM type inference
pub mod typeck;

/// Intermediate representation
pub mod ir;

/// Bytecode generation
pub mod codegen;

/// Error types and reporting
pub mod error;

pub use error::{CompilerError, Result};

/// Compiler version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Compile source code to bytecode
pub fn compile(_source: &str) -> Result<Vec<u8>> {
    todo!("Implement compilation pipeline")
}
