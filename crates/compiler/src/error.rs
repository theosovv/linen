//! Compiler error types

use thiserror::Error;

/// Result type for compiler operations
pub type Result<T> = std::result::Result<T, CompilerError>;

/// Errors that can occur during compilation
#[derive(Error, Debug, Clone)]
pub enum CompilerError {
    /// Generic error with message
    #[error("Compiler error: {0}")]
    Generic(String),
}
