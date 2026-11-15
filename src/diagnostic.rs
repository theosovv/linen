use std::error::Error;
use std::fmt;
use std::fmt::Write;

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
    InvalidRenderConfiguration,
    NonFiniteOutput,
    RenderOverflow,
    FileIo,
}

impl ErrorCode {
    pub const ALL: [Self; 20] = [
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LineColumn {
    pub line: usize,
    pub column: usize,
}

impl LineColumn {
    pub const fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
}

pub fn line_column(source: &str, offset: usize) -> Option<LineColumn> {
    let mut chars = source.get(..offset)?.chars().peekable();
    let mut line = 1;
    let mut column = 1;

    while let Some(ch) = chars.next() {
        match ch {
            '\r' if chars.peek() == Some(&'\n') => {
                chars.next();
                line += 1;
                column = 1;
            }
            '\n' => {
                line += 1;
                column = 1;
            }
            '\r' => {
                column += 1;
            }
            _ => {
                column += 1;
            }
        }
    }

    Some(LineColumn::new(line, column))
}

pub fn render_diagnostic(source_name: &str, source: &str, diagnostic: &Diagnostic) -> String {
    let severity = diagnostic.severity();
    let code = diagnostic.code();
    let message = diagnostic.message();
    let primary = diagnostic.primary();
    let primary_message = primary.message();
    let span = diagnostic.primary().span();
    let start = span.start();

    let (column, line) = match line_column(source, span.start()) {
        Some(location) => (location.column, location.line),
        None => (0, 0),
    };

    let line_start = source[..start].rfind('\n').map_or(0, |index| index + 1);

    let line_end = source[start..]
        .find('\n')
        .map_or(source.len(), |relative| start + relative);

    let position = if column > 0 && line > 0 {
        format!("{line}:{column}")
    } else {
        "".to_string()
    };

    let raw_line = &source[line_start..line_end];
    let source_line = raw_line.strip_suffix('\r').unwrap_or(raw_line);

    let mut output = String::new();

    let _ = writeln!(output, "{severity}[{code}]: {message}");
    let _ = writeln!(output, "  --> {source_name}:{position}");
    let _ = writeln!(output, "    |");
    let _ = writeln!(output, "  {line} | {source_line}",);
    let _ = writeln!(output, "    |{}^ {}", " ".repeat(column), primary_message);
    output
}
