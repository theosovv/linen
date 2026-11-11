//! Public library facade for linen.
//!
//! At stage 0 the crate provides structured diagnostics. Compiler and audio
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

pub use diagnostic::{Diagnostic, DiagnosticLabel, ErrorCode, InvalidSpan, Severity, Span};
