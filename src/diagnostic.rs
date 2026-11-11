use std::error::Error;
use std::fmt;

/// A half-open range of byte offsets in UTF-8 source text.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    /// Creates a span covering `[start, end)`.
    pub const fn new(start: usize, end: usize) -> Result<Self, InvalidSpan> {
        if start <= end {
            Ok(Self { start, end })
        } else {
            Err(InvalidSpan { start, end })
        }
    }

    /// Creates an empty span at a byte offset.
    pub const fn empty(offset: usize) -> Self {
        Self {
            start: offset,
            end: offset,
        }
    }

    pub const fn start(self) -> usize {
        self.start
    }

    pub const fn end(self) -> usize {
        self.end
    }

    pub const fn len(self) -> usize {
        self.end - self.start
    }

    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

/// An error returned when a span's start follows its end.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InvalidSpan {
    start: usize,
    end: usize,
}

impl InvalidSpan {
    pub const fn start(self) -> usize {
        self.start
    }

    pub const fn end(self) -> usize {
        self.end
    }
}

impl fmt::Display for InvalidSpan {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid source span: start {} is greater than end {}",
            self.start, self.end
        )
    }
}

impl Error for InvalidSpan {}

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
    InvalidRenderConfiguration,
    NonFiniteOutput,
    RenderOverflow,
    FileIo,
}

impl ErrorCode {
    pub const ALL: [Self; 18] = [
        Self::UnknownCharacter,
        Self::InvalidNumberOrDuration,
        Self::UnexpectedToken,
        Self::MissingSemicolon,
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
        Self::InvalidRenderConfiguration,
        Self::NonFiniteOutput,
        Self::RenderOverflow,
        Self::FileIo,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownCharacter => "E1000",
            Self::InvalidNumberOrDuration => "E1001",
            Self::UnexpectedToken => "E1100",
            Self::MissingSemicolon => "E1101",
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
            Self::InvalidRenderConfiguration => "E4000",
            Self::NonFiniteOutput => "E4001",
            Self::RenderOverflow => "E4002",
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
    span: Span,
    message: String,
}

impl DiagnosticLabel {
    pub fn new(span: Span, message: impl Into<String>) -> Self {
        Self {
            span,
            message: message.into(),
        }
    }

    pub const fn span(&self) -> Span {
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
    primary: DiagnosticLabel,
    secondary: Vec<DiagnosticLabel>,
    notes: Vec<String>,
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
            primary,
            secondary: Vec::new(),
            notes: Vec::new(),
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

    pub const fn primary(&self) -> &DiagnosticLabel {
        &self.primary
    }

    pub fn secondary(&self) -> &[DiagnosticLabel] {
        &self.secondary
    }

    pub fn notes(&self) -> &[String] {
        &self.notes
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
}
