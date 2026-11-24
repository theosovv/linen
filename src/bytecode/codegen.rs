use crate::{
    bytecode::{
        AudioRegister, BinaryOp, Instruction, Operand, OutputLayout, ScalarRegister, StateId,
    },
    ir::{Graph, NodeOp, NodeOrigin, ValueType},
};

use super::Bytecode;

#[derive(Clone, Copy)]
enum ValueRegisters {
    Scalar(ScalarRegister),
    Audio(AudioRegister),
    Stereo {
        left: AudioRegister,
        right: AudioRegister,
    },
}

struct Codegen {
    instructions: Vec<Instruction>,
    registers: Vec<ValueRegisters>,
    debug_origins: Vec<NodeOrigin>,
    scalar_register_count: usize,
    audio_register_count: usize,
    state_count: usize,
}

fn alloc_scalar(codegen: &mut Codegen) -> ScalarRegister {
    let register = ScalarRegister::new(codegen.scalar_register_count);
    codegen.scalar_register_count += 1;
    register
}

fn alloc_audio(codegen: &mut Codegen) -> AudioRegister {
    let register = AudioRegister::new(codegen.audio_register_count);
    codegen.audio_register_count += 1;
    register
}

fn alloc_state(codegen: &mut Codegen) -> StateId {
    let state = StateId::new(codegen.state_count);
    codegen.state_count += 1;
    state
}

fn scalar_input(registers: ValueRegisters) -> ScalarRegister {
    match registers {
        ValueRegisters::Scalar(register) => register,
        _ => panic!("verified scalar input must have a scalar register"),
    }
}

fn audio_input(registers: ValueRegisters) -> AudioRegister {
    match registers {
        ValueRegisters::Audio(register) => register,
        _ => panic!("verified audio input must have an audio register"),
    }
}

fn audio_operand(registers: ValueRegisters) -> Operand {
    Operand::Audio(audio_input(registers))
}

pub fn codegen(graph: &Graph, output_origin: NodeOrigin) -> Bytecode {
    let mut codegen = Codegen {
        instructions: Vec::new(),
        registers: Vec::new(),
        debug_origins: Vec::new(),
        scalar_register_count: 0,
        audio_register_count: 0,
        state_count: 0,
    };

    for node in &graph.nodes {
        let (result, instruction) = match &node.op {
            NodeOp::ConstFloat(value) => {
                let dst = alloc_scalar(&mut codegen);
                (
                    ValueRegisters::Scalar(dst),
                    Instruction::ConstScalar { dst, value: *value },
                )
            }
            NodeOp::LiftFloatToSig(input) => {
                let src = scalar_input(codegen.registers[input.index()]);
                let dst = alloc_audio(&mut codegen);
                (ValueRegisters::Audio(dst), Instruction::Lift { dst, src })
            }
            NodeOp::Neg(input) => {
                let (instruction, result) = match node.result_type {
                    ValueType::Float => {
                        let src = scalar_input(codegen.registers[input.index()]);
                        let dst = alloc_scalar(&mut codegen);
                        (
                            Instruction::NegScalar { dst, src },
                            ValueRegisters::Scalar(dst),
                        )
                    }
                    ValueType::Sig => {
                        let src = audio_input(codegen.registers[input.index()]);
                        let dst = alloc_audio(&mut codegen);
                        (
                            Instruction::NegAudio { dst, src },
                            ValueRegisters::Audio(dst),
                        )
                    }
                    ValueType::Stereo => panic!("verified unary nodes cannot produce Stereo"),
                };
                (result, instruction)
            }
            NodeOp::Add(left, right)
            | NodeOp::Sub(left, right)
            | NodeOp::Mul(left, right)
            | NodeOp::Div(left, right) => {
                let op = match &node.op {
                    NodeOp::Add(_, _) => BinaryOp::Add,
                    NodeOp::Sub(_, _) => BinaryOp::Subtract,
                    NodeOp::Mul(_, _) => BinaryOp::Multiply,
                    NodeOp::Div(_, _) => BinaryOp::Divide,
                    _ => panic!("invalid operator"),
                };
                let (instruction, result) = match node.result_type {
                    ValueType::Float => {
                        let lhs = scalar_input(codegen.registers[left.index()]);
                        let rhs = scalar_input(codegen.registers[right.index()]);
                        let dst = alloc_scalar(&mut codegen);
                        (
                            Instruction::BinaryScalar { op, dst, lhs, rhs },
                            ValueRegisters::Scalar(dst),
                        )
                    }
                    ValueType::Sig => {
                        let lhs = codegen.registers[left.index()];
                        let rhs = codegen.registers[right.index()];
                        let lhs = audio_operand(lhs);
                        let rhs = audio_operand(rhs);
                        let dst = alloc_audio(&mut codegen);
                        (
                            Instruction::BinaryAudio { op, dst, lhs, rhs },
                            ValueRegisters::Audio(dst),
                        )
                    }
                    ValueType::Stereo => panic!("verified binary nodes cannot produce Stereo"),
                };
                (result, instruction)
            }
            NodeOp::Sine(_)
            | NodeOp::Phasor(_)
            | NodeOp::Saw(_)
            | NodeOp::Square(_)
            | NodeOp::SawPolyblep(_)
            | NodeOp::SquarePolyblep(_)
            | NodeOp::Triangle(_) => {
                let frequency = match &node.op {
                    NodeOp::Sine(frequency)
                    | NodeOp::Phasor(frequency)
                    | NodeOp::Saw(frequency)
                    | NodeOp::Square(frequency)
                    | NodeOp::SawPolyblep(frequency)
                    | NodeOp::SquarePolyblep(frequency)
                    | NodeOp::Triangle(frequency) => *frequency,
                    _ => panic!("matched phase operation"),
                };
                let freq = audio_operand(codegen.registers[frequency.index()]);
                let dst = alloc_audio(&mut codegen);
                let state = alloc_state(&mut codegen);
                let instruction = match &node.op {
                    NodeOp::Sine(_) => Instruction::Sine { dst, freq, state },
                    NodeOp::Phasor(_) => Instruction::Phasor { dst, freq, state },
                    NodeOp::Saw(_) => Instruction::Saw { dst, freq, state },
                    NodeOp::Square(_) => Instruction::Square { dst, freq, state },
                    NodeOp::SawPolyblep(_) => Instruction::SawPolyblep { dst, freq, state },
                    NodeOp::SquarePolyblep(_) => Instruction::SquarePolyblep { dst, freq, state },
                    NodeOp::Triangle(_) => Instruction::Triangle { dst, freq, state },
                    _ => panic!("matched phase operation"),
                };
                (ValueRegisters::Audio(dst), instruction)
            }
            NodeOp::Stereo(left, right) => {
                let src_left = audio_input(codegen.registers[left.index()]);
                let src_right = audio_input(codegen.registers[right.index()]);
                let dst_left = alloc_audio(&mut codegen);
                let dst_right = alloc_audio(&mut codegen);
                (
                    ValueRegisters::Stereo {
                        left: dst_left,
                        right: dst_right,
                    },
                    Instruction::MakeStereo {
                        dst_left,
                        dst_right,
                        src_left,
                        src_right,
                    },
                )
            }
            NodeOp::Pan(input, position) => {
                let input = audio_operand(codegen.registers[input.index()]);
                let position = audio_operand(codegen.registers[position.index()]);
                let dst_left = alloc_audio(&mut codegen);
                let dst_right = alloc_audio(&mut codegen);
                (
                    ValueRegisters::Stereo {
                        left: dst_left,
                        right: dst_right,
                    },
                    Instruction::Pan {
                        dst_left,
                        dst_right,
                        input,
                        position,
                    },
                )
            }
            NodeOp::Noise { stream, .. } => {
                let dst = alloc_audio(&mut codegen);
                let state = alloc_state(&mut codegen);
                (
                    ValueRegisters::Audio(dst),
                    Instruction::Noise {
                        dst,
                        stream: *stream,
                        state,
                    },
                )
            }
        };

        codegen.registers.push(result);
        codegen.instructions.push(instruction);
        codegen.debug_origins.push(node.origin.clone());
    }

    let (output, output_instruction) = match codegen.registers[graph.output.index()] {
        ValueRegisters::Audio(src) => {
            let output = alloc_audio(&mut codegen);
            (OutputLayout::Mono(output), Instruction::SetOutput { src })
        }
        ValueRegisters::Stereo { left, right } => {
            let output_left = alloc_audio(&mut codegen);
            let output_right = alloc_audio(&mut codegen);
            (
                OutputLayout::Stereo {
                    left: output_left,
                    right: output_right,
                },
                Instruction::SetStereoOutput {
                    src_left: left,
                    src_right: right,
                },
            )
        }
        ValueRegisters::Scalar(_) => panic!("verified output must be audio"),
    };

    codegen.instructions.push(output_instruction);
    codegen.debug_origins.push(output_origin);

    Bytecode {
        instructions: codegen.instructions,
        scalar_register_count: codegen.scalar_register_count,
        audio_register_count: codegen.audio_register_count,
        state_count: codegen.state_count,
        output,
        debug_origins: Some(codegen.debug_origins),
    }
}
