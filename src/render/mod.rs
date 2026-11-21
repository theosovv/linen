mod wav;

use std::{error::Error, fmt, io};

use crate::{
    Diagnostic, DiagnosticLabel, ErrorCode, SourceSpan,
    bytecode::Bytecode,
    ir::Graph,
    sema::TypedProgram,
    vm::{Vm, VmConfig, VmCreateError, VmRuntimeError},
};

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
    pub ir: Graph,
    pub bytecode: Bytecode,
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
    VmCreate(VmCreateError),
    VmRuntime(VmRuntimeError),
    MissingDebugOrigin { instruction: usize },
}

impl fmt::Display for RenderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Diagnostic(diagnostic) => write!(formatter, "{diagnostic}"),
            Self::Io(error) => write!(formatter, "could not write rendered audio: {error}"),
            Self::VmCreate(error) => write!(formatter, "could not initialize VM: {error}"),
            Self::VmRuntime(error) => error.fmt(formatter),
            Self::MissingDebugOrigin { instruction } => write!(
                formatter,
                "missing debug origin for VM instruction {instruction}"
            ),
        }
    }
}

impl Error for RenderError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Diagnostic(diagnostic) => Some(diagnostic),
            Self::Io(error) => Some(error),
            Self::VmCreate(error) => Some(error),
            Self::VmRuntime(error) => Some(error),
            Self::MissingDebugOrigin { .. } => None,
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
        .bindings()
        .into_iter()
        .find(|binding| binding.id == program.typed.output)
        .expect("a compiled program must have its output binding")
        .value
        .span;

    let mut vm = Vm::new(
        &program.bytecode,
        VmConfig {
            sample_rate: program.config.sample_rate,
            block_size: program.config.block_size,
        },
    )
    .map_err(RenderError::VmCreate)?;
    let mut scratch = vec![0.0; program.config.block_size];

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

    let mut sample_index = 0_u64;
    while sample_index < render.render_samples {
        let remaining = render.render_samples - sample_index;
        let length = usize::try_from(remaining)
            .unwrap_or(usize::MAX)
            .min(scratch.len());
        let block_length = u64::try_from(length)
            .map_err(|_| RenderError::VmRuntime(VmRuntimeError::SampleIndexOverflow))?;

        vm.render_into(&mut scratch[..length])
            .map_err(|error| map_vm_runtime_error(program, error, output_span))?;

        for (offset, sample) in scratch[..length].iter().copied().enumerate() {
            let sample_index = sample_index + offset as u64;
            let sample = quantize_pcm16(sample).map_err(|_| {
                RenderError::Diagnostic(nonfinite_output(output_span, sample_index))
            })?;
            writer.write_all(&sample.to_le_bytes())?;
        }

        sample_index += block_length;
    }

    Ok(RenderReport {
        samples: render.render_samples,
        sample_rate: program.config.sample_rate,
        duration_seconds: render.duration_seconds,
    })
}

fn map_vm_runtime_error(
    program: &CompiledProgram,
    error: VmRuntimeError,
    output_span: SourceSpan,
) -> RenderError {
    let (instruction, sample_index) = match error {
        VmRuntimeError::NonFiniteFrequency {
            instruction,
            sample_index,
            ..
        } => (instruction, sample_index),
        VmRuntimeError::NonFiniteOutput { sample_index } => {
            return RenderError::Diagnostic(nonfinite_output(output_span, sample_index));
        }
        other => return RenderError::VmRuntime(other),
    };

    let Some(origin) = program
        .bytecode
        .debug_origins
        .as_ref()
        .and_then(|origins| origins.get(instruction))
    else {
        return RenderError::MissingDebugOrigin { instruction };
    };

    let mut diagnostic = Diagnostic::error(
        ErrorCode::NonFiniteOutput,
        "sine frequency contains NaN or infinity",
        DiagnosticLabel::new(
            origin.primary,
            "this argument produces the invalid frequency",
        ),
    )
    .with_sample_index(sample_index);

    for call in &origin.inlined_from {
        diagnostic = diagnostic.with_secondary(DiagnosticLabel::new(
            call.call_span,
            "inlined from this function call",
        ));
    }

    RenderError::Diagnostic(diagnostic)
}

fn nonfinite_output(span: SourceSpan, sample_index: u64) -> Diagnostic {
    Diagnostic::error(
        ErrorCode::NonFiniteOutput,
        "output signal contains NaN or infinity",
        DiagnosticLabel::new(span, "this out expression produces the invalid sample"),
    )
    .with_sample_index(sample_index)
}
