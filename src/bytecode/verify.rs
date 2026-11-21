use std::{collections::BTreeSet, error::Error, fmt};

use crate::{
    SourceMap,
    bytecode::{Instruction, Operand},
    ir::NodeOrigin,
};

use super::{AudioRegister, Bytecode, ScalarRegister, StateId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BytecodeVerifyError {
    UnknownScalarRegister {
        instruction: usize,
        register: ScalarRegister,
    },
    UnknownAudioRegister {
        instruction: usize,
        register: AudioRegister,
    },
    UninitializedScalarRegister {
        instruction: usize,
        register: ScalarRegister,
    },
    UninitializedAudioRegister {
        instruction: usize,
        register: AudioRegister,
    },
    NonFiniteConstant {
        instruction: usize,
    },
    UnknownState {
        instruction: usize,
        state: StateId,
    },
    DuplicateState {
        instruction: usize,
        state: StateId,
    },
    MissingState(StateId),
    OutputOutOfRange(AudioRegister),
    OutputWritten {
        instruction: usize,
    },
    MissingSetOutput,
    NonFinalSetOutput {
        instruction: usize,
    },
    DebugOriginCount {
        instructions: usize,
        origins: usize,
    },
    InvalidOrigin {
        instruction: usize,
    },
}

impl fmt::Display for BytecodeVerifyError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid bytecode: {self:?}")
    }
}

impl Error for BytecodeVerifyError {}

fn check_scalar(
    bytecode: &Bytecode,
    initialized: &BTreeSet<usize>,
    instruction: usize,
    register: ScalarRegister,
) -> Result<(), BytecodeVerifyError> {
    if register.index() >= bytecode.scalar_register_count {
        return Err(BytecodeVerifyError::UnknownScalarRegister {
            instruction,
            register,
        });
    }

    if !initialized.contains(&register.index()) {
        return Err(BytecodeVerifyError::UninitializedScalarRegister {
            instruction,
            register,
        });
    }

    Ok(())
}

fn check_scalar_dst(
    bytecode: &Bytecode,
    instruction: usize,
    dst: ScalarRegister,
) -> Result<(), BytecodeVerifyError> {
    if dst.index() >= bytecode.scalar_register_count {
        return Err(BytecodeVerifyError::UnknownScalarRegister {
            instruction,
            register: dst,
        });
    }

    Ok(())
}

fn check_audio_dst(
    bytecode: &Bytecode,
    instruction: usize,
    dst: AudioRegister,
) -> Result<(), BytecodeVerifyError> {
    if dst.index() >= bytecode.audio_register_count {
        return Err(BytecodeVerifyError::UnknownAudioRegister {
            instruction,
            register: dst,
        });
    }

    if dst == bytecode.output {
        return Err(BytecodeVerifyError::OutputWritten { instruction });
    }

    Ok(())
}

fn check_audio(
    bytecode: &Bytecode,
    initialized: &BTreeSet<usize>,
    instruction: usize,
    register: AudioRegister,
) -> Result<(), BytecodeVerifyError> {
    if register.index() >= bytecode.audio_register_count {
        return Err(BytecodeVerifyError::UnknownAudioRegister {
            instruction,
            register,
        });
    }

    if !initialized.contains(&register.index()) {
        return Err(BytecodeVerifyError::UninitializedAudioRegister {
            instruction,
            register,
        });
    }

    Ok(())
}

fn check_operand(
    bytecode: &Bytecode,
    scalar_initialized: &BTreeSet<usize>,
    audio_initialized: &BTreeSet<usize>,
    instruction: usize,
    operand: Operand,
) -> Result<(), BytecodeVerifyError> {
    match operand {
        Operand::Scalar(register) => {
            check_scalar(bytecode, scalar_initialized, instruction, register)
        }
        Operand::Audio(register) => check_audio(bytecode, audio_initialized, instruction, register),
    }
}

pub fn verify(bytecode: &Bytecode) -> Result<(), BytecodeVerifyError> {
    let mut scalar_initialized = BTreeSet::new();
    let mut audio_initialized = BTreeSet::new();
    let mut state_used = BTreeSet::new();

    if bytecode.output.index() >= bytecode.audio_register_count {
        return Err(BytecodeVerifyError::OutputOutOfRange(bytecode.output));
    }

    if !matches!(
        bytecode.instructions.last(),
        Some(Instruction::SetOutput { .. })
    ) {
        return Err(BytecodeVerifyError::MissingSetOutput);
    }

    for (instruction_index, instruction) in bytecode.instructions.iter().enumerate() {
        match instruction {
            Instruction::ConstScalar { dst, value } => {
                check_scalar_dst(bytecode, instruction_index, *dst)?;

                if !value.is_finite() {
                    return Err(BytecodeVerifyError::NonFiniteConstant {
                        instruction: instruction_index,
                    });
                }

                scalar_initialized.insert(dst.index());
            }
            Instruction::Lift { dst, src } => {
                check_audio_dst(bytecode, instruction_index, *dst)?;
                check_scalar(bytecode, &scalar_initialized, instruction_index, *src)?;

                audio_initialized.insert(dst.index());
            }
            Instruction::NegScalar { dst, src } => {
                check_scalar_dst(bytecode, instruction_index, *dst)?;
                check_scalar(bytecode, &scalar_initialized, instruction_index, *src)?;

                scalar_initialized.insert(dst.index());
            }
            Instruction::NegAudio { dst, src } => {
                check_audio_dst(bytecode, instruction_index, *dst)?;
                check_audio(bytecode, &audio_initialized, instruction_index, *src)?;

                audio_initialized.insert(dst.index());
            }
            Instruction::BinaryScalar {
                op: _,
                dst,
                lhs,
                rhs,
            } => {
                check_scalar_dst(bytecode, instruction_index, *dst)?;
                check_scalar(bytecode, &scalar_initialized, instruction_index, *lhs)?;
                check_scalar(bytecode, &scalar_initialized, instruction_index, *rhs)?;

                scalar_initialized.insert(dst.index());
            }
            Instruction::BinaryAudio {
                op: _,
                dst,
                lhs,
                rhs,
            } => {
                check_audio_dst(bytecode, instruction_index, *dst)?;
                check_operand(
                    bytecode,
                    &scalar_initialized,
                    &audio_initialized,
                    instruction_index,
                    *lhs,
                )?;
                check_operand(
                    bytecode,
                    &scalar_initialized,
                    &audio_initialized,
                    instruction_index,
                    *rhs,
                )?;

                audio_initialized.insert(dst.index());
            }
            Instruction::Sine { dst, freq, state } => {
                check_audio_dst(bytecode, instruction_index, *dst)?;
                check_operand(
                    bytecode,
                    &scalar_initialized,
                    &audio_initialized,
                    instruction_index,
                    *freq,
                )?;

                if state.index() >= bytecode.state_count {
                    return Err(BytecodeVerifyError::UnknownState {
                        instruction: instruction_index,
                        state: *state,
                    });
                }

                if state_used.contains(&state.index()) {
                    return Err(BytecodeVerifyError::DuplicateState {
                        instruction: instruction_index,
                        state: *state,
                    });
                }

                state_used.insert(state.index());
                audio_initialized.insert(dst.index());
            }
            Instruction::SetOutput { src } => {
                if instruction_index + 1 != bytecode.instructions.len() {
                    return Err(BytecodeVerifyError::NonFinalSetOutput {
                        instruction: instruction_index,
                    });
                }

                check_audio(bytecode, &audio_initialized, instruction_index, *src)?;

                audio_initialized.insert(bytecode.output.index());
            }
        }
    }

    if state_used.len() != bytecode.state_count {
        for index in 0..bytecode.state_count {
            if !state_used.contains(&index) {
                return Err(BytecodeVerifyError::MissingState(StateId::new(index)));
            }
        }
    }

    Ok(())
}

fn origin_is_valid(sources: &SourceMap, origin: NodeOrigin) -> bool {
    let span = origin.primary;
    let Some(source) = sources.get(span.source()) else {
        return false;
    };

    let text = source.text();

    span.start() <= span.end()
        && span.end() <= text.len()
        && text.is_char_boundary(span.start())
        && text.is_char_boundary(span.end())
}

pub fn verify_origins(bytecode: &Bytecode, sources: &SourceMap) -> Result<(), BytecodeVerifyError> {
    let Some(origins) = &bytecode.debug_origins else {
        return Ok(());
    };

    if origins.len() != bytecode.instructions.len() {
        return Err(BytecodeVerifyError::DebugOriginCount {
            instructions: bytecode.instructions.len(),
            origins: origins.len(),
        });
    }

    for (instruction, &origin) in origins.iter().enumerate() {
        if !origin_is_valid(sources, origin) {
            return Err(BytecodeVerifyError::InvalidOrigin { instruction });
        }
    }

    Ok(())
}
