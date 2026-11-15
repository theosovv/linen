//! Public library facade for linen.
//!
//! The crate provides structured diagnostics and the unstable stage 1 syntax
//! scaffolding. Lexer and parser entry points are under implementation; audio
//! rendering APIs will be exposed only when their implementations exist.
//!
//! ```
//! use linen::{Diagnostic, DiagnosticLabel, ErrorCode, Span};
//!
//! let span = Span::new(4, 7)?;
//! let diagnostic = Diagnostic::error(
//!     ErrorCode::UnknownName,
//!     "unknown name",
//!     DiagnosticLabel::new(span, "this name is not defined"),
//! );
//!
//! assert_eq!(diagnostic.code(), ErrorCode::UnknownName);
//! assert_eq!(diagnostic.primary().span(), span);
//! # Ok::<(), linen::InvalidSpan>(())
//! ```

pub mod diagnostic;
pub mod syntax;

pub use diagnostic::{
    Diagnostic, DiagnosticLabel, ErrorCode, InvalidSpan, LineColumn, Severity, Span, line_column,
    render_diagnostic,
};
pub use syntax::{Program, check};
