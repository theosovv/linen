use std::{collections::BTreeSet, error::Error, fmt};

use crate::{
    SourceMap,
    bytecode::{Instruction, Operand, OutputLayout},
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
    InvalidNoiseStream {
        instruction: usize,
    },
    OutputOutOfRange(AudioRegister),
    OutputWritten {
        instruction: usize,
    },
    MissingSetOutput,
    NonFinalSetOutput {
        instruction: usize,
    },
    OutputLayoutMismatch,
    DuplicateStereoDestination {
        instruction: usize,
        register: AudioRegister,
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
    if bytecode.output.contains(dst) {
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

fn check_state(
    bytecode: &Bytecode,
    state_used: &mut BTreeSet<usize>,
    instruction: usize,
    state: StateId,
) -> Result<(), BytecodeVerifyError> {
    if state.index() >= bytecode.state_count {
        return Err(BytecodeVerifyError::UnknownState { instruction, state });
    }
    if state_used.contains(&state.index()) {
        return Err(BytecodeVerifyError::DuplicateState { instruction, state });
    }
    state_used.insert(state.index());
    Ok(())
}

fn check_output_registers(bytecode: &Bytecode) -> Result<(), BytecodeVerifyError> {
    match bytecode.output {
        OutputLayout::Mono(output) => {
            if output.index() >= bytecode.audio_register_count {
                return Err(BytecodeVerifyError::OutputOutOfRange(output));
            }
        }
        OutputLayout::Stereo { left, right } => {
            if left.index() >= bytecode.audio_register_count {
                return Err(BytecodeVerifyError::OutputOutOfRange(left));
            }
            if right.index() >= bytecode.audio_register_count {
                return Err(BytecodeVerifyError::OutputOutOfRange(right));
            }
            if left == right {
                return Err(BytecodeVerifyError::OutputLayoutMismatch);
            }
        }
    }
    Ok(())
}

fn check_final_output(bytecode: &Bytecode) -> Result<(), BytecodeVerifyError> {
    match (bytecode.output, bytecode.instructions.last()) {
        (OutputLayout::Mono(_), Some(Instruction::SetOutput { .. }))
        | (OutputLayout::Stereo { .. }, Some(Instruction::SetStereoOutput { .. })) => Ok(()),
        (OutputLayout::Mono(_), Some(Instruction::SetStereoOutput { .. }))
        | (OutputLayout::Stereo { .. }, Some(Instruction::SetOutput { .. })) => {
            Err(BytecodeVerifyError::OutputLayoutMismatch)
        }
        _ => Err(BytecodeVerifyError::MissingSetOutput),
    }
}

pub fn verify(bytecode: &Bytecode) -> Result<(), BytecodeVerifyError> {
    let mut scalar_initialized = BTreeSet::new();
    let mut audio_initialized = BTreeSet::new();
    let mut state_used = BTreeSet::new();

    check_output_registers(bytecode)?;
    check_final_output(bytecode)?;

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
            Instruction::BinaryScalar { dst, lhs, rhs, .. } => {
                check_scalar_dst(bytecode, instruction_index, *dst)?;
                check_scalar(bytecode, &scalar_initialized, instruction_index, *lhs)?;
                check_scalar(bytecode, &scalar_initialized, instruction_index, *rhs)?;
                scalar_initialized.insert(dst.index());
            }
            Instruction::BinaryAudio { dst, lhs, rhs, .. } => {
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
            Instruction::Sine { dst, freq, state }
            | Instruction::Phasor { dst, freq, state }
            | Instruction::Saw { dst, freq, state }
            | Instruction::Square { dst, freq, state }
            | Instruction::SawPolyblep { dst, freq, state }
            | Instruction::SquarePolyblep { dst, freq, state }
            | Instruction::Triangle { dst, freq, state } => {
                check_audio_dst(bytecode, instruction_index, *dst)?;
                check_operand(
                    bytecode,
                    &scalar_initialized,
                    &audio_initialized,
                    instruction_index,
                    *freq,
                )?;
                check_state(bytecode, &mut state_used, instruction_index, *state)?;
                audio_initialized.insert(dst.index());
            }
            Instruction::MakeStereo {
                dst_left,
                dst_right,
                src_left,
                src_right,
            } => {
                check_audio_dst(bytecode, instruction_index, *dst_left)?;
                check_audio_dst(bytecode, instruction_index, *dst_right)?;
                if dst_left == dst_right {
                    return Err(BytecodeVerifyError::DuplicateStereoDestination {
                        instruction: instruction_index,
                        register: *dst_left,
                    });
                }
                check_audio(bytecode, &audio_initialized, instruction_index, *src_left)?;
                check_audio(bytecode, &audio_initialized, instruction_index, *src_right)?;
                audio_initialized.insert(dst_left.index());
                audio_initialized.insert(dst_right.index());
            }
            Instruction::Pan {
                dst_left,
                dst_right,
                input,
                position,
            } => {
                check_audio_dst(bytecode, instruction_index, *dst_left)?;
                check_audio_dst(bytecode, instruction_index, *dst_right)?;
                if dst_left == dst_right {
                    return Err(BytecodeVerifyError::DuplicateStereoDestination {
                        instruction: instruction_index,
                        register: *dst_left,
                    });
                }
                check_operand(
                    bytecode,
                    &scalar_initialized,
                    &audio_initialized,
                    instruction_index,
                    *input,
                )?;
                check_operand(
                    bytecode,
                    &scalar_initialized,
                    &audio_initialized,
                    instruction_index,
                    *position,
                )?;
                audio_initialized.insert(dst_left.index());
                audio_initialized.insert(dst_right.index());
            }
            Instruction::Noise { dst, stream, state } => {
                check_audio_dst(bytecode, instruction_index, *dst)?;
                if stream.seed() > 9_007_199_254_740_991 {
                    return Err(BytecodeVerifyError::InvalidNoiseStream {
                        instruction: instruction_index,
                    });
                }
                check_state(bytecode, &mut state_used, instruction_index, *state)?;
                audio_initialized.insert(dst.index());
            }
            Instruction::SetOutput { src } => {
                if instruction_index + 1 != bytecode.instructions.len() {
                    return Err(BytecodeVerifyError::NonFinalSetOutput {
                        instruction: instruction_index,
                    });
                }
                let OutputLayout::Mono(output) = bytecode.output else {
                    return Err(BytecodeVerifyError::OutputLayoutMismatch);
                };
                check_audio(bytecode, &audio_initialized, instruction_index, *src)?;
                audio_initialized.insert(output.index());
            }
            Instruction::SetStereoOutput {
                src_left,
                src_right,
            } => {
                if instruction_index + 1 != bytecode.instructions.len() {
                    return Err(BytecodeVerifyError::NonFinalSetOutput {
                        instruction: instruction_index,
                    });
                }
                let OutputLayout::Stereo { left, right } = bytecode.output else {
                    return Err(BytecodeVerifyError::OutputLayoutMismatch);
                };
                check_audio(bytecode, &audio_initialized, instruction_index, *src_left)?;
                check_audio(bytecode, &audio_initialized, instruction_index, *src_right)?;
                audio_initialized.insert(left.index());
                audio_initialized.insert(right.index());
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

fn span_is_valid(sources: &SourceMap, span: crate::SourceSpan) -> bool {
    let Some(source) = sources.get(span.source()) else {
        return false;
    };
    let text = source.text();

    span.start() <= span.end()
        && span.end() <= text.len()
        && text.is_char_boundary(span.start())
        && text.is_char_boundary(span.end())
}

fn origin_is_valid(sources: &SourceMap, origin: &NodeOrigin) -> bool {
    span_is_valid(sources, origin.primary)
        && origin
            .inlined_from
            .iter()
            .all(|call| span_is_valid(sources, call.call_span))
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

    for (instruction, origin) in origins.iter().enumerate() {
        if !origin_is_valid(sources, origin) {
            return Err(BytecodeVerifyError::InvalidOrigin { instruction });
        }
    }

    Ok(())
}
