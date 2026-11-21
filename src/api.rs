use std::{error::Error, fmt};

use crate::{
    Diagnostic,
    bytecode::{self, BytecodeVerifyError},
    ir::{self, IrVerifyError},
    render::{CompiledProgram, RenderConfig, RenderConfigError},
    sema::{SymbolStorage, check_program},
    source::{SourceId, SourceMap},
    syntax::{ParserConfig, Program, parse_with_config},
    vm::{Vm, VmConfig, VmCreateError},
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CompileConfig {
    pub parser: ParserConfig,
}

/// Owns the source map used to produce the contained result.
///
/// Use `result` only with `sources` from this outcome or an identical clone.
/// IDs from other outcomes and sessions may have colliding map-local indices.
#[derive(Clone, Debug, PartialEq)]
pub struct CheckOutcome {
    pub sources: SourceMap,
    pub result: Result<Program, Diagnostic>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompileOutcome {
    pub sources: SourceMap,
    pub symbols: SymbolStorage,
    pub result: Result<CompiledProgram, CompileError>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CompileError {
    Diagnostic(Diagnostic),
    IrVerify(IrVerifyError),
    BytecodeVerify(BytecodeVerifyError),
}

impl CompileError {
    pub const fn as_diagnostic(&self) -> Option<&Diagnostic> {
        match self {
            Self::Diagnostic(diagnostic) => Some(diagnostic),
            Self::IrVerify(_) | Self::BytecodeVerify(_) => None,
        }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Diagnostic(diagnostic) => diagnostic.fmt(formatter),
            Self::IrVerify(error) => error.fmt(formatter),
            Self::BytecodeVerify(error) => error.fmt(formatter),
        }
    }
}

impl Error for CompileError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Diagnostic(diagnostic) => Some(diagnostic),
            Self::IrVerify(error) => Some(error),
            Self::BytecodeVerify(error) => Some(error),
        }
    }
}

impl From<Diagnostic> for CompileError {
    fn from(diagnostic: Diagnostic) -> Self {
        Self::Diagnostic(diagnostic)
    }
}

impl From<IrVerifyError> for CompileError {
    fn from(error: IrVerifyError) -> Self {
        Self::IrVerify(error)
    }
}

impl From<BytecodeVerifyError> for CompileError {
    fn from(error: BytecodeVerifyError) -> Self {
        Self::BytecodeVerify(error)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompileSession {
    sources: SourceMap,
    symbols: SymbolStorage,
    config: CompileConfig,
}

impl CompileSession {
    pub const fn new(config: CompileConfig) -> Self {
        Self {
            sources: SourceMap::new(),
            symbols: SymbolStorage::new(),
            config,
        }
    }

    pub const fn sources(&self) -> &SourceMap {
        &self.sources
    }

    pub fn sources_mut(&mut self) -> &mut SourceMap {
        &mut self.sources
    }

    /// Returns symbols produced by the latest semantic compilation.
    ///
    /// Storage is reset whenever `check` or `compile` reaches semantic
    /// analysis. Resolve earlier typed results before another such call.
    pub const fn symbols(&self) -> &SymbolStorage {
        &self.symbols
    }

    /// Checks a source previously added to this session.
    ///
    /// `source` must originate from `self.sources()`. Only an index absent
    /// from this session produces [`SessionError::UnknownSource`]; a foreign
    /// ID with an in-range index cannot be distinguished from a local ID.
    pub fn check(&mut self, source: SourceId) -> Result<Result<Program, Diagnostic>, SessionError> {
        let file = self
            .sources
            .get(source)
            .ok_or(SessionError::UnknownSource(source))?;
        let parsed = parse_with_config(file.text(), source, self.config.parser);
        let result = parsed.and_then(|program| {
            check_program(
                &program,
                &mut self.symbols,
                RenderConfig::default().sample_rate,
            )?;
            Ok(program)
        });
        Ok(result)
    }

    pub fn compile(
        &mut self,
        source: SourceId,
        render_config: RenderConfig,
    ) -> Result<Result<CompiledProgram, CompileError>, SessionError> {
        render_config
            .validate()
            .map_err(SessionError::InvalidRenderConfiguration)?;
        let file = self
            .sources
            .get(source)
            .ok_or(SessionError::UnknownSource(source))?;
        let parsed = parse_with_config(file.text(), source, self.config.parser);
        let result = parsed
            .map_err(CompileError::Diagnostic)
            .and_then(|program| {
                let typed = check_program(&program, &mut self.symbols, render_config.sample_rate)?;
                let output_origin = typed
                    .bindings
                    .iter()
                    .find(|binding| binding.id == typed.output)
                    .expect("typed output must refer to a binding")
                    .value
                    .span;
                let graph = ir::lower(&typed);
                ir::verify(&graph, &self.sources)?;
                let bytecode = bytecode::codegen(
                    &graph,
                    ir::NodeOrigin {
                        primary: output_origin,
                    },
                );
                bytecode::verify(&bytecode)?;
                bytecode::verify_origins(&bytecode, &self.sources)?;

                Ok(CompiledProgram {
                    typed,
                    ir: graph,
                    bytecode,
                    config: render_config,
                })
            });
        Ok(result)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionError {
    UnknownSource(SourceId),
    InvalidRenderConfiguration(RenderConfigError),
}

impl fmt::Display for SessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::UnknownSource(source) => {
                write!(formatter, "unknown source ID {}", source.index())
            }
            Self::InvalidRenderConfiguration(error) => error.fmt(formatter),
        }
    }
}

impl Error for SessionError {}

pub fn check(source: &str) -> CheckOutcome {
    check_with_config("input.ln", source, CompileConfig::default())
}

pub fn check_with_config(logical_name: &str, source: &str, config: CompileConfig) -> CheckOutcome {
    let mut session = CompileSession::new(config);
    let source = session.sources_mut().add_source(logical_name, source);
    let result = session
        .check(source)
        .expect("a freshly added source must belong to its session");

    CheckOutcome {
        sources: session.sources,
        result,
    }
}

pub fn compile(
    source: &str,
    render_config: RenderConfig,
) -> Result<CompileOutcome, RenderConfigError> {
    compile_with_config("input.ln", source, CompileConfig::default(), render_config)
}

pub fn compile_with_config(
    logical_name: &str,
    source: &str,
    config: CompileConfig,
    render_config: RenderConfig,
) -> Result<CompileOutcome, RenderConfigError> {
    render_config.validate()?;
    let mut session = CompileSession::new(config);
    let source = session.sources_mut().add_source(logical_name, source);
    let result = session
        .compile(source, render_config)
        .expect("a freshly added source must belong to its session");

    Ok(CompileOutcome {
        sources: session.sources,
        symbols: session.symbols,
        result,
    })
}

pub fn create_vm(program: &CompiledProgram) -> Result<Vm<'_>, VmCreateError> {
    Vm::new(
        &program.bytecode,
        VmConfig {
            sample_rate: program.config.sample_rate,
            block_size: program.config.block_size,
        },
    )
}
