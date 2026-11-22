use std::error::Error;
use std::fmt;
use std::fmt::Write;

use crate::source::{SourceFile, SourceId, SourceMap, SourceSpan};

pub use crate::source::{InvalidSpan, LineColumn};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Severity {
    Error,
    Warning,
    Note,
}

impl Severity {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Note => "note",
        }
    }
}

impl fmt::Display for Severity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A stable diagnostic code defined by the language specification.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ErrorCode {
    UnknownCharacter,
    InvalidNumberOrDuration,
    UnexpectedToken,
    MissingSemicolon,
    ExpressionDepthLimit,
    AstNodeLimit,
    DuplicateBinding,
    UnknownName,
    MissingOutput,
    MultipleOutputs,
    MissingOrMisplacedRender,
    TypeMismatch,
    InvalidArity,
    UnknownFunction,
    InvalidDuration,
    ConstantDivisionByZero,
    InvalidFloat,
    FunctionExpansionLimit,
    InvalidNoiseSeed,
    InvalidRenderConfiguration,
    NonFiniteOutput,
    RenderOverflow,
    RenderLimitExceeded,
    FileIo,
}

impl ErrorCode {
    pub const ALL: [Self; 24] = [
        Self::UnknownCharacter,
        Self::InvalidNumberOrDuration,
        Self::UnexpectedToken,
        Self::MissingSemicolon,
        Self::ExpressionDepthLimit,
        Self::AstNodeLimit,
        Self::DuplicateBinding,
        Self::UnknownName,
        Self::MissingOutput,
        Self::MultipleOutputs,
        Self::MissingOrMisplacedRender,
        Self::TypeMismatch,
        Self::InvalidArity,
        Self::UnknownFunction,
        Self::InvalidDuration,
        Self::ConstantDivisionByZero,
        Self::InvalidFloat,
        Self::FunctionExpansionLimit,
        Self::InvalidNoiseSeed,
        Self::InvalidRenderConfiguration,
        Self::NonFiniteOutput,
        Self::RenderOverflow,
        Self::RenderLimitExceeded,
        Self::FileIo,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownCharacter => "E1000",
            Self::InvalidNumberOrDuration => "E1001",
            Self::UnexpectedToken => "E1100",
            Self::MissingSemicolon => "E1101",
            Self::ExpressionDepthLimit => "E1102",
            Self::AstNodeLimit => "E1103",
            Self::DuplicateBinding => "E2000",
            Self::UnknownName => "E2001",
            Self::MissingOutput => "E2002",
            Self::MultipleOutputs => "E2003",
            Self::MissingOrMisplacedRender => "E2004",
            Self::TypeMismatch => "E2100",
            Self::InvalidArity => "E2101",
            Self::UnknownFunction => "E2102",
            Self::InvalidDuration => "E2103",
            Self::ConstantDivisionByZero => "E2104",
            Self::InvalidFloat => "E2105",
            Self::FunctionExpansionLimit => "E2106",
            Self::InvalidNoiseSeed => "E2107",
            Self::InvalidRenderConfiguration => "E4000",
            Self::NonFiniteOutput => "E4001",
            Self::RenderOverflow => "E4002",
            Self::RenderLimitExceeded => "E4003",
            Self::FileIo => "E5000",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

/// A source span with a human-readable explanation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticLabel {
    span: SourceSpan,
    message: String,
}

impl DiagnosticLabel {
    pub fn new(span: SourceSpan, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
        }
    }

    pub const fn span(&self) -> SourceSpan {
        self.span
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

/// A structured diagnostic independent of source rendering and CLI output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    severity: Severity,
    code: ErrorCode,
    message: String,
    primary: Box<DiagnosticLabel>,
    secondary: Vec<DiagnosticLabel>,
    notes: Vec<String>,
    sample_index: Option<u64>,
}

impl Diagnostic {
    pub fn new(
        severity: Severity,
        code: ErrorCode,
        message: impl Into<String>,
        primary: DiagnosticLabel,
    ) -> Self {
        Self {
            severity,
            code,
            message: message.into(),
            primary: Box::new(primary),
            secondary: Vec::new(),
            notes: Vec::new(),
            sample_index: None,
        }
    }

    pub fn error(code: ErrorCode, message: impl Into<String>, primary: DiagnosticLabel) -> Self {
        Self::new(Severity::Error, code, message, primary)
    }

    pub const fn severity(&self) -> Severity {
        self.severity
    }

    pub const fn code(&self) -> ErrorCode {
        self.code
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn primary(&self) -> &DiagnosticLabel {
        self.primary.as_ref()
    }

    pub fn secondary(&self) -> &[DiagnosticLabel] {
        &self.secondary
    }

    pub fn notes(&self) -> &[String] {
        &self.notes
    }

    pub const fn sample_index(&self) -> Option<u64> {
        self.sample_index
    }

    #[must_use]
    pub fn with_secondary(mut self, label: DiagnosticLabel) -> Self {
        self.secondary.push(label);
        self
    }

    #[must_use]
    pub fn with_note(mut self, note: impl Into<String>) -> Self {
        self.notes.push(note.into());
        self
    }

    #[must_use]
    pub fn with_sample_index(mut self, sample_index: u64) -> Self {
        self.sample_index = Some(sample_index);
        self
    }
}

impl fmt::Display for Diagnostic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.code(), self.message())
    }
}

impl Error for Diagnostic {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticRenderError {
    UnknownSource(SourceId),
    ReversedSpan(SourceSpan),
    OffsetOutOfBounds {
        source: SourceId,
        offset: usize,
        source_len: usize,
    },
    NonUtf8Boundary {
        source: SourceId,
        offset: usize,
    },
}

impl fmt::Display for DiagnosticRenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UnknownSource(source) => {
                write!(formatter, "unknown source ID {}", source.index())
            }
            Self::ReversedSpan(span) => write!(
                formatter,
                "span start {} is greater than end {} in source ID {}",
                span.start(),
                span.end(),
                span.source().index()
            ),
            Self::OffsetOutOfBounds {
                source,
                offset,
                source_len,
            } => write!(
                formatter,
                "offset {offset} is outside source ID {} of length {source_len}",
                source.index()
            ),
            Self::NonUtf8Boundary { source, offset } => write!(
                formatter,
                "offset {offset} is not a UTF-8 boundary in source ID {}",
                source.index()
            ),
        }
    }
}

impl Error for DiagnosticRenderError {}

fn source_for_span(
    sources: &SourceMap,
    span: SourceSpan,
) -> Result<&SourceFile, DiagnosticRenderError> {
    let source = sources
        .get(span.source())
        .ok_or(DiagnosticRenderError::UnknownSource(span.source()))?;
    let start = span.start();
    let end = span.end();

    if start > end {
        return Err(DiagnosticRenderError::ReversedSpan(span));
    }

    for offset in [start, end] {
        if offset > source.text().len() {
            return Err(DiagnosticRenderError::OffsetOutOfBounds {
                source: span.source(),
                offset,
                source_len: source.text().len(),
            });
        }

        if !source.text().is_char_boundary(offset) {
            return Err(DiagnosticRenderError::NonUtf8Boundary {
                source: span.source(),
                offset,
            });
        }
    }

    Ok(source)
}

/// Renders a diagnostic using the map that owns all of its label source IDs.
///
/// [`DiagnosticRenderError::UnknownSource`] reports an index absent from
/// `sources`. IDs from independently created maps may collide; passing a
/// foreign ID with an in-range index is outside this function's contract.
pub fn render_diagnostic(
    sources: &SourceMap,
    diagnostic: &Diagnostic,
) -> Result<String, DiagnosticRenderError> {
    let primary = diagnostic.primary();
    let span = primary.span();
    let source = source_for_span(sources, span)?;

    let start = span.start();
    let location = source
        .line_column(start)
        .ok_or(DiagnosticRenderError::NonUtf8Boundary {
            source: span.source(),
            offset: start,
        })?;
    let line = location.line;
    let column = location.column;
    let text = source.text();
    let line_start = text[..start].rfind('\n').map_or(0, |index| index + 1);
    let line_end = text[start..]
        .find('\n')
        .map_or(text.len(), |relative| start + relative);
    let raw_line = &text[line_start..line_end];
    let source_line = raw_line.strip_suffix('\r').unwrap_or(raw_line);
    let mut output = String::new();

    let _ = writeln!(
        output,
        "{}[{}]: {}",
        diagnostic.severity(),
        diagnostic.code(),
        diagnostic.message()
    );
    let _ = writeln!(output, "  --> {}:{line}:{column}", source.logical_name());
    let _ = writeln!(output, "    |");
    let _ = writeln!(output, "  {line} | {source_line}");
    let _ = writeln!(output, "    |{}^ {}", " ".repeat(column), primary.message());

    for label in diagnostic.secondary() {
        let span = label.span();
        let source = source_for_span(sources, span)?;
        let location =
            source
                .line_column(span.start())
                .ok_or(DiagnosticRenderError::NonUtf8Boundary {
                    source: span.source(),
                    offset: span.start(),
                })?;
        let _ = writeln!(
            output,
            "  = {}:{}:{}: {}",
            source.logical_name(),
            location.line,
            location.column,
            label.message()
        );
    }

    for note in diagnostic.notes() {
        let _ = writeln!(output, "  = {note}");
    }

    if let Some(sample_index) = diagnostic.sample_index() {
        let _ = writeln!(output, "  = sample index: {sample_index}");
    }

    Ok(output)
}
