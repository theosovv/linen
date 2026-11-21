use std::{error::Error, fmt};

use crate::bytecode::{
    self, BinaryOp, Bytecode, BytecodeVerifyError, Instruction, Operand, StateId,
};

use super::{VmConfig, VmConfigError, state::NodeState};

#[derive(Debug)]
pub struct Vm<'program> {
    program: &'program Bytecode,
    config: VmConfig,
    scalar_registers: Vec<f64>,
    audio_registers: Vec<Vec<f32>>,
    states: Vec<NodeState>,
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
    NonFiniteFrequency {
        instruction: usize,
        state: StateId,
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
        checked_allocation_bytes(program.state_count, size_of::<NodeState>())?;
        checked_allocation_bytes(program.audio_register_count, size_of::<Vec<f32>>())?;

        bytecode::verify(program)?;

        let scalar_registers = vec![0.0; program.scalar_register_count];
        let mut audio_registers = Vec::with_capacity(program.audio_register_count);

        for _ in 0..program.audio_register_count {
            audio_registers.push(vec![0.0; config.block_size]);
        }

        let states = vec![NodeState::new(); program.state_count];

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
        for state in &mut self.states {
            state.reset();
        }
        self.sample_index = 0;
    }

    pub fn render_into(&mut self, output: &mut [f32]) -> Result<(), VmRuntimeError> {
        if output.is_empty() {
            return Ok(());
        }

        if output.len() > self.config.block_size {
            return Err(VmRuntimeError::BlockTooLarge {
                requested: output.len(),
                maximum: self.config.block_size,
            });
        }

        let block_start = self.sample_index;
        let block_length =
            u64::try_from(output.len()).map_err(|_| VmRuntimeError::SampleIndexOverflow)?;
        let next_sample_index = block_start
            .checked_add(block_length)
            .ok_or(VmRuntimeError::SampleIndexOverflow)?;

        for sample_offset in 0..output.len() {
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
                    Instruction::Sine { dst, freq, state } => {
                        let frequency = operand_at(
                            *freq,
                            sample_offset,
                            &self.scalar_registers,
                            &self.audio_registers,
                        );
                        let sample = self.states[state.index()]
                            .next_sample(f64::from(frequency), self.config.sample_rate)
                            .map_err(|_| VmRuntimeError::NonFiniteFrequency {
                                instruction: instruction_index,
                                state: *state,
                                sample_index,
                            })?;

                        self.audio_registers[dst.index()][sample_offset] = sample;
                    }
                    Instruction::SetOutput { src } => {
                        let output_index = self.program.output.index();
                        let value = self.audio_registers[src.index()][sample_offset];

                        self.audio_registers[output_index][sample_offset] = value;
                        if !value.is_finite() {
                            return Err(VmRuntimeError::NonFiniteOutput { sample_index });
                        }
                    }
                }
            }
        }

        output.copy_from_slice(&self.audio_registers[self.program.output.index()][..output.len()]);

        self.sample_index = next_sample_index;

        Ok(())
    }
}
