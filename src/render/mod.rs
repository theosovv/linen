mod interpreter;
mod wav;

use std::{error::Error, fmt, io};

use crate::{Diagnostic, DiagnosticLabel, ErrorCode, SourceSpan, sema::TypedProgram};

pub use interpreter::NaiveInterpreter;
pub use wav::{NonFiniteSample, WavError, quantize_pcm16, write_wav_header};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderConfig {
    pub sample_rate: u32,
    pub block_size: usize,
    pub max_output_seconds: u64,
}

impl RenderConfig {
    pub fn validate(self) -> Result<(), RenderConfigError> {
        if !(8_000..=384_000).contains(&self.sample_rate) {
            return Err(RenderConfigError::InvalidSampleRate(self.sample_rate));
        }
        if self.block_size == 0 {
            return Err(RenderConfigError::ZeroBlockSize);
        }
        Ok(())
    }
}

impl Default for RenderConfig {
    fn default() -> Self {
        Self {
            sample_rate: 48_000,
            block_size: 128,
            max_output_seconds: 600,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderConfigError {
    InvalidSampleRate(u32),
    ZeroBlockSize,
}

impl fmt::Display for RenderConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::InvalidSampleRate(rate) => write!(
                formatter,
                "invalid sample rate {rate}; expected 8000..=384000"
            ),
            Self::ZeroBlockSize => formatter.write_str("block size must be greater than zero"),
        }
    }
}

impl Error for RenderConfigError {}

#[derive(Clone, Debug, PartialEq)]
pub struct CompiledProgram {
    pub typed: TypedProgram,
    pub config: RenderConfig,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RenderReport {
    pub samples: u64,
    pub sample_rate: u32,
    pub duration_seconds: f64,
}

#[derive(Debug)]
pub enum RenderError {
    Diagnostic(Diagnostic),
    Io(io::Error),
}

impl fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Diagnostic(diagnostic) => write!(formatter, "{diagnostic}"),
            Self::Io(error) => write!(formatter, "could not write rendered audio: {error}"),
        }
    }
}

impl Error for RenderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Diagnostic(diagnostic) => Some(diagnostic),
            Self::Io(error) => Some(error),
        }
    }
}

impl From<io::Error> for RenderError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub fn render<W: io::Write>(
    program: &CompiledProgram,
    writer: &mut W,
) -> Result<RenderReport, RenderError> {
    let render = &program.typed.render;
    if render.duration_seconds > program.config.max_output_seconds as f64 {
        return Err(RenderError::Diagnostic(Diagnostic::error(
            ErrorCode::RenderLimitExceeded,
            "render exceeds the configured duration limit",
            DiagnosticLabel::new(
                render.duration.span,
                format!(
                    "requested {} seconds, but max_output_seconds is {}",
                    render.duration_seconds, program.config.max_output_seconds
                ),
            ),
        )));
    }

    let output_span = program
        .typed
        .bindings
        .iter()
        .find(|binding| binding.id == program.typed.output)
        .expect("a compiled program must have its output binding")
        .value
        .span;

    write_wav_header(writer, program.config.sample_rate, render.render_samples).map_err(
        |error| match error {
            WavError::Io(error) => RenderError::Io(error),
            WavError::SizeOverflow => RenderError::Diagnostic(Diagnostic::error(
                ErrorCode::RenderOverflow,
                "render duration exceeds WAV size limits",
                DiagnosticLabel::new(
                    render.duration.span,
                    "sample count or RIFF/PCM size overflows its supported integer range",
                ),
            )),
        },
    )?;

    let mut interpreter = NaiveInterpreter::new(&program.typed);
    for sample_index in 0..render.render_samples {
        let sample = interpreter
            .next_sample(sample_index, program.config.sample_rate)
            .map_err(|diagnostic| {
                RenderError::Diagnostic(ensure_sample_index(diagnostic, sample_index))
            })?;

        if !sample.is_finite() {
            return Err(RenderError::Diagnostic(nonfinite_output(
                output_span,
                sample_index,
            )));
        }

        let sample = quantize_pcm16(sample)
            .map_err(|_| RenderError::Diagnostic(nonfinite_output(output_span, sample_index)))?;
        writer.write_all(&sample.to_le_bytes())?;
    }

    Ok(RenderReport {
        samples: render.render_samples,
        sample_rate: program.config.sample_rate,
        duration_seconds: render.duration_seconds,
    })
}

fn ensure_sample_index(mut diagnostic: Diagnostic, sample_index: u64) -> Diagnostic {
    if diagnostic.code() == ErrorCode::NonFiniteOutput && diagnostic.sample_index().is_none() {
        diagnostic = diagnostic.with_sample_index(sample_index);
    }
    diagnostic
}

fn nonfinite_output(span: SourceSpan, sample_index: u64) -> Diagnostic {
    Diagnostic::error(
        ErrorCode::NonFiniteOutput,
        "output signal contains NaN or infinity",
        DiagnosticLabel::new(span, "this out expression produces the invalid sample"),
    )
    .with_sample_index(sample_index)
}
