//! Public library facade for linen.
//!
//! The crate provides structured diagnostics and the unstable stage 1A
//! source-aware syntax API. Audio rendering APIs will be exposed only when
//! their implementations exist.
//!
//! ```
//! use linen::{Diagnostic, DiagnosticLabel, ErrorCode, SourceMap, SourceSpan};
//!
//! let mut sources = SourceMap::new();
//! let source = sources.add_source("input.ln", "let out = 1; render 1s;");
//! let span = SourceSpan::new(source, 4, 7)?;
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

pub mod api;
pub mod diagnostic;
pub mod source;
pub mod syntax;

pub use api::{
    CheckOutcome, CompileConfig, CompileSession, SessionError, check, check_with_config,
};
pub use diagnostic::{
    Diagnostic, DiagnosticLabel, DiagnosticRenderError, ErrorCode, Severity, render_diagnostic,
};
pub use source::{InvalidSpan, LineColumn, SourceFile, SourceId, SourceMap, SourceSpan};
pub use syntax::Program;
