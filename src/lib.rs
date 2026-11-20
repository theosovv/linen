//! Public library facade for linen.
//!
//! The crate provides structured diagnostics, the source-aware syntax API and
//! the unstable stage 2 representations used by the current implementation.
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
pub mod dsp;
pub mod render;
pub mod sema;
pub mod source;
pub mod syntax;

pub use api::{
    CheckOutcome, CompileConfig, CompileOutcome, CompileSession, SessionError, check,
    check_with_config, compile, compile_with_config,
};
pub use diagnostic::{
    Diagnostic, DiagnosticLabel, DiagnosticRenderError, ErrorCode, Severity, render_diagnostic,
};
pub use render::{
    CompiledProgram, RenderConfig, RenderConfigError, RenderError, RenderReport, render,
};
pub use source::{InvalidSpan, LineColumn, SourceFile, SourceId, SourceMap, SourceSpan};
pub use syntax::Program;
