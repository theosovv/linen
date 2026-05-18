//! Error types and diagnostics for the Linen compiler.

use crate::span::Span;
use thiserror::Error;

/// Top-level compiler error.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum CompileError {
    #[error("{0}")]
    Lexer(LexError),

    #[error("{0}")]
    Parser(ParseError),

    #[error("{0}")]
    Type(TypeError),
}

/// Lexical error.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum LexError {
    #[error("Unexpected character '{0}' at {1:?}")]
    UnexpectedChar(char, Span),

    #[error("Unterminated string literal at {0:?}")]
    UnterminatedString(Span),

    #[error("Invalid number literal at {0:?}")]
    InvalidNumber(Span),
}

/// Parse error.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum ParseError {
    #[error("Unexpected token: expected {expected}, found {found} at {span:?}")]
    UnexpectedToken {
        expected: String,
        found: String,
        span: Span,
    },

    #[error("Unexpected end of file at {0:?}")]
    UnexpectedEof(Span),

    #[error("Invalid pattern at {0:?}")]
    InvalidPattern(Span),
}

/// Type error.
#[derive(Error, Debug, Clone, PartialEq)]
pub enum TypeError {
    #[error("Type mismatch: expected {expected}, found {found} at {span:?}")]
    TypeMismatch {
        expected: String,
        found: String,
        span: Span,
    },

    #[error("Undefined variable '{name}' at {span:?}")]
    UndefinedVariable { name: String, span: Span },

    #[error("Infinite type detected at {0:?}")]
    InfiniteType(Span),

    #[error("Unification error: {0} at {1:?}")]
    UnificationError(String, Span),
}

/// A diagnostic message with severity.
#[derive(Debug, Clone, PartialEq)]
pub struct Diagnostic {
    pub severity: Severity,
    pub message: String,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
    Info,
}
