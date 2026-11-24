use std::{error::Error, fmt};

use crate::{
    bytecode::{
        self, BinaryOp, Bytecode, BytecodeVerifyError, Instruction, Operand, OutputLayout, StateId,
    },
    dsp::{self, Noise, Phasor, Saw, SawPolyblep, Sine, Square, SquarePolyblep, Triangle},
};

use super::{VmConfig, VmConfigError, state::NodeState};

#[derive(Debug)]
pub struct Vm<'program> {
    program: &'program Bytecode,
    config: VmConfig,
    scalar_registers: Vec<f64>,
    audio_registers: Vec<Vec<f32>>,
    states: Vec<Option<NodeState>>,
    sample_index: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum VmCreateError {
    InvalidConfiguration(VmConfigError),
    InvalidBytecode(BytecodeVerifyError),
    SizeOverflow,
}

impl fmt::Display for VmCreateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfiguration(error) => error.fmt(formatter),
            Self::InvalidBytecode(error) => error.fmt(formatter),
            Self::SizeOverflow => formatter.write_str("VM buffer sizes overflow usize"),
        }
    }
}

impl Error for VmCreateError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidConfiguration(error) => Some(error),
            Self::InvalidBytecode(error) => Some(error),
            Self::SizeOverflow => None,
        }
    }
}

impl From<VmConfigError> for VmCreateError {
    fn from(error: VmConfigError) -> Self {
        Self::InvalidConfiguration(error)
    }
}

impl From<BytecodeVerifyError> for VmCreateError {
    fn from(error: BytecodeVerifyError) -> Self {
        Self::InvalidBytecode(error)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VmRuntimeError {
    BlockTooLarge {
        requested: usize,
        maximum: usize,
    },
    InvalidInterleavedBlock {
        length: usize,
    },
    NonFiniteFrequency {
        instruction: usize,
        state: StateId,
        sample_index: u64,
    },
    NonFinitePosition {
        instruction: usize,
        sample_index: u64,
    },
    NonFiniteOutput {
        sample_index: u64,
    },
    SampleIndexOverflow,
}

impl fmt::Display for VmRuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "VM execution failed: {self:?}")
    }
}

impl Error for VmRuntimeError {}

fn operand_at(
    operand: Operand,
    sample_offset: usize,
    scalar_registers: &[f64],
    audio_registers: &[Vec<f32>],
) -> f32 {
    match operand {
        Operand::Scalar(register) => scalar_registers[register.index()] as f32,
        Operand::Audio(register) => audio_registers[register.index()][sample_offset],
    }
}

fn checked_allocation_bytes(count: usize, element_size: usize) -> Result<usize, VmCreateError> {
    let bytes = count
        .checked_mul(element_size)
        .ok_or(VmCreateError::SizeOverflow)?;

    if bytes > isize::MAX as usize {
        return Err(VmCreateError::SizeOverflow);
    }

    Ok(bytes)
}

impl<'program> Vm<'program> {
    pub fn new(program: &'program Bytecode, config: VmConfig) -> Result<Self, VmCreateError> {
        config.validate()?;

        let audio_buffer_bytes = checked_allocation_bytes(config.block_size, size_of::<f32>())?;
        program
            .audio_register_count
            .checked_mul(audio_buffer_bytes)
            .ok_or(VmCreateError::SizeOverflow)?;
        checked_allocation_bytes(program.scalar_register_count, size_of::<f64>())?;
        checked_allocation_bytes(program.state_count, size_of::<Option<NodeState>>())?;
        checked_allocation_bytes(program.audio_register_count, size_of::<Vec<f32>>())?;
        config
            .block_size
            .checked_mul(program.output.channels())
            .ok_or(VmCreateError::SizeOverflow)?;

        bytecode::verify(program)?;

        let scalar_registers = vec![0.0; program.scalar_register_count];
        let mut audio_registers = Vec::with_capacity(program.audio_register_count);
        for _ in 0..program.audio_register_count {
            audio_registers.push(vec![0.0; config.block_size]);
        }

        let mut states = Vec::with_capacity(program.state_count);
        states.resize_with(program.state_count, || None);
        for instruction in &program.instructions {
            match instruction {
                Instruction::Sine { state, .. } => {
                    states[state.index()] = Some(NodeState::Sine(Sine::new()));
                }
                Instruction::Phasor { state, .. } => {
                    states[state.index()] = Some(NodeState::Phasor(Phasor::new()));
                }
                Instruction::Saw { state, .. } => {
                    states[state.index()] = Some(NodeState::Saw(Saw::new()));
                }
                Instruction::Square { state, .. } => {
                    states[state.index()] = Some(NodeState::Square(Square::new()));
                }
                Instruction::SawPolyblep { state, .. } => {
                    states[state.index()] = Some(NodeState::SawPolyblep(SawPolyblep::new()));
                }
                Instruction::SquarePolyblep { state, .. } => {
                    states[state.index()] = Some(NodeState::SquarePolyblep(SquarePolyblep::new()));
                }
                Instruction::Triangle { state, .. } => {
                    states[state.index()] = Some(NodeState::Triangle(Triangle::new()));
                }
                Instruction::Noise { stream, state, .. } => {
                    states[state.index()] = Some(NodeState::Noise(Noise::new(stream.seed())));
                }
                _ => {}
            }
        }

        Ok(Self {
            program,
            config,
            scalar_registers,
            audio_registers,
            states,
            sample_index: 0,
        })
    }

    pub fn bytecode(&self) -> &Bytecode {
        self.program
    }

    pub const fn config(&self) -> VmConfig {
        self.config
    }

    pub const fn sample_index(&self) -> u64 {
        self.sample_index
    }

    pub fn reset(&mut self) {
        self.scalar_registers.fill(0.0);
        for buffer in &mut self.audio_registers {
            buffer.fill(0.0);
        }
        for state in self.states.iter_mut().flatten() {
            state.reset();
        }
        self.sample_index = 0;
    }

    pub fn render_into(&mut self, output: &mut [f32]) -> Result<(), VmRuntimeError> {
        if output.is_empty() {
            return Ok(());
        }

        let channels = self.program.output.channels();
        if channels == 2 && !output.len().is_multiple_of(2) {
            return Err(VmRuntimeError::InvalidInterleavedBlock {
                length: output.len(),
            });
        }
        let frames = output.len() / channels;
        if frames > self.config.block_size {
            return Err(VmRuntimeError::BlockTooLarge {
                requested: frames,
                maximum: self.config.block_size,
            });
        }

        let block_start = self.sample_index;
        let block_length =
            u64::try_from(frames).map_err(|_| VmRuntimeError::SampleIndexOverflow)?;
        let next_sample_index = block_start
            .checked_add(block_length)
            .ok_or(VmRuntimeError::SampleIndexOverflow)?;

        for sample_offset in 0..frames {
            let sample_index = block_start + sample_offset as u64;

            for (instruction_index, instruction) in self.program.instructions.iter().enumerate() {
                match instruction {
                    Instruction::ConstScalar { dst, value } => {
                        self.scalar_registers[dst.index()] = *value;
                    }
                    Instruction::NegScalar { dst, src } => {
                        self.scalar_registers[dst.index()] = -self.scalar_registers[src.index()];
                    }
                    Instruction::BinaryScalar { op, dst, lhs, rhs } => {
                        let lhs = self.scalar_registers[lhs.index()];
                        let rhs = self.scalar_registers[rhs.index()];
                        self.scalar_registers[dst.index()] = match op {
                            BinaryOp::Add => lhs + rhs,
                            BinaryOp::Subtract => lhs - rhs,
                            BinaryOp::Multiply => lhs * rhs,
                            BinaryOp::Divide => lhs / rhs,
                        };
                    }
                    Instruction::Lift { dst, src } => {
                        self.audio_registers[dst.index()][sample_offset] =
                            self.scalar_registers[src.index()] as f32;
                    }
                    Instruction::NegAudio { dst, src } => {
                        self.audio_registers[dst.index()][sample_offset] =
                            -self.audio_registers[src.index()][sample_offset];
                    }
                    Instruction::BinaryAudio { op, dst, lhs, rhs } => {
                        let lhs = operand_at(
                            *lhs,
                            sample_offset,
                            &self.scalar_registers,
                            &self.audio_registers,
                        );
                        let rhs = operand_at(
                            *rhs,
                            sample_offset,
                            &self.scalar_registers,
                            &self.audio_registers,
                        );
                        self.audio_registers[dst.index()][sample_offset] = match op {
                            BinaryOp::Add => lhs + rhs,
                            BinaryOp::Subtract => lhs - rhs,
                            BinaryOp::Multiply => lhs * rhs,
                            BinaryOp::Divide => lhs / rhs,
                        };
                    }
                    Instruction::Sine { dst, freq, state }
                    | Instruction::Phasor { dst, freq, state }
                    | Instruction::Saw { dst, freq, state }
                    | Instruction::Square { dst, freq, state }
                    | Instruction::SawPolyblep { dst, freq, state }
                    | Instruction::SquarePolyblep { dst, freq, state }
                    | Instruction::Triangle { dst, freq, state } => {
                        let frequency = operand_at(
                            *freq,
                            sample_offset,
                            &self.scalar_registers,
                            &self.audio_registers,
                        );
                        let node_state = self.states[state.index()]
                            .as_mut()
                            .expect("verified state slot must be initialized");
                        let result = match (instruction, node_state) {
                            (Instruction::Sine { .. }, NodeState::Sine(state)) => {
                                state.next_sample(f64::from(frequency), self.config.sample_rate)
                            }
                            (Instruction::Phasor { .. }, NodeState::Phasor(state)) => {
                                state.next_sample(f64::from(frequency), self.config.sample_rate)
                            }
                            (Instruction::Saw { .. }, NodeState::Saw(state)) => {
                                state.next_sample(f64::from(frequency), self.config.sample_rate)
                            }
                            (Instruction::Square { .. }, NodeState::Square(state)) => {
                                state.next_sample(f64::from(frequency), self.config.sample_rate)
                            }
                            (Instruction::SawPolyblep { .. }, NodeState::SawPolyblep(state)) => {
                                state.next_sample(f64::from(frequency), self.config.sample_rate)
                            }
                            (
                                Instruction::SquarePolyblep { .. },
                                NodeState::SquarePolyblep(state),
                            ) => state.next_sample(f64::from(frequency), self.config.sample_rate),
                            (Instruction::Triangle { .. }, NodeState::Triangle(state)) => {
                                state.next_sample(f64::from(frequency), self.config.sample_rate)
                            }
                            _ => panic!("verified oscillator instruction must match its state"),
                        };
                        let sample = result.map_err(|_| VmRuntimeError::NonFiniteFrequency {
                            instruction: instruction_index,
                            state: *state,
                            sample_index,
                        })?;
                        self.audio_registers[dst.index()][sample_offset] = sample;
                    }
                    Instruction::MakeStereo {
                        dst_left,
                        dst_right,
                        src_left,
                        src_right,
                    } => {
                        let left = self.audio_registers[src_left.index()][sample_offset];
                        let right = self.audio_registers[src_right.index()][sample_offset];
                        self.audio_registers[dst_left.index()][sample_offset] = left;
                        self.audio_registers[dst_right.index()][sample_offset] = right;
                    }
                    Instruction::Pan {
                        dst_left,
                        dst_right,
                        input,
                        position,
                    } => {
                        let input = operand_at(
                            *input,
                            sample_offset,
                            &self.scalar_registers,
                            &self.audio_registers,
                        );
                        let position = operand_at(
                            *position,
                            sample_offset,
                            &self.scalar_registers,
                            &self.audio_registers,
                        );
                        let [left, right] = dsp::pan(input, position).map_err(|_| {
                            VmRuntimeError::NonFinitePosition {
                                instruction: instruction_index,
                                sample_index,
                            }
                        })?;
                        self.audio_registers[dst_left.index()][sample_offset] = left;
                        self.audio_registers[dst_right.index()][sample_offset] = right;
                    }
                    Instruction::Noise { dst, state, .. } => {
                        let node_state = self.states[state.index()]
                            .as_mut()
                            .expect("verified state slot must be initialized");
                        let NodeState::Noise(noise) = node_state else {
                            panic!("verified noise instruction must match its state");
                        };
                        self.audio_registers[dst.index()][sample_offset] = noise.next_sample();
                    }
                    Instruction::SetOutput { src } => {
                        let OutputLayout::Mono(output) = self.program.output else {
                            panic!("verified mono output must have mono layout");
                        };
                        let value = self.audio_registers[src.index()][sample_offset];
                        if !value.is_finite() {
                            return Err(VmRuntimeError::NonFiniteOutput { sample_index });
                        }
                        self.audio_registers[output.index()][sample_offset] = value;
                    }
                    Instruction::SetStereoOutput {
                        src_left,
                        src_right,
                    } => {
                        let OutputLayout::Stereo { left, right } = self.program.output else {
                            panic!("verified stereo output must have stereo layout");
                        };
                        let left_value = self.audio_registers[src_left.index()][sample_offset];
                        let right_value = self.audio_registers[src_right.index()][sample_offset];
                        if !left_value.is_finite() || !right_value.is_finite() {
                            return Err(VmRuntimeError::NonFiniteOutput { sample_index });
                        }
                        self.audio_registers[left.index()][sample_offset] = left_value;
                        self.audio_registers[right.index()][sample_offset] = right_value;
                    }
                }
            }
        }

        match self.program.output {
            OutputLayout::Mono(register) => {
                output.copy_from_slice(&self.audio_registers[register.index()][..frames]);
            }
            OutputLayout::Stereo { left, right } => {
                for frame in 0..frames {
                    output[frame * 2] = self.audio_registers[left.index()][frame];
                    output[frame * 2 + 1] = self.audio_registers[right.index()][frame];
                }
            }
        }

        self.sample_index = next_sample_index;
        Ok(())
    }
}
