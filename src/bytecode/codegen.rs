use crate::{
    bytecode::{AudioRegister, BinaryOp, Instruction, Operand, ScalarRegister, StateId},
    ir::{Graph, NodeOp, NodeOrigin, ValueType},
};

use super::Bytecode;

struct Codegen {
    instructions: Vec<Instruction>,
    registers: Vec<Operand>,
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

fn scalar_input(operand: Operand) -> ScalarRegister {
    match operand {
        Operand::Scalar(register) => register,
        Operand::Audio(_) => panic!("verified scalar input must have a scalar register"),
    }
}

fn audio_input(operand: Operand) -> AudioRegister {
    match operand {
        Operand::Audio(register) => register,
        Operand::Scalar(_) => panic!("verified audio input must have an audio register"),
    }
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
                let instruction = Instruction::ConstScalar { dst, value: *value };
                let result = Operand::Scalar(dst);

                (result, instruction)
            }
            NodeOp::LiftFloatToSig(input) => {
                let src = scalar_input(codegen.registers[input.index()]);
                let dst = alloc_audio(&mut codegen);
                let instruction = Instruction::Lift { dst, src };
                let result = Operand::Audio(dst);

                (result, instruction)
            }
            NodeOp::Neg(input) => {
                let (instruction, result) = match node.result_type {
                    ValueType::Float => {
                        let src = scalar_input(codegen.registers[input.index()]);
                        let dst = alloc_scalar(&mut codegen);

                        (Instruction::NegScalar { dst, src }, Operand::Scalar(dst))
                    }
                    ValueType::Sig => {
                        let src = audio_input(codegen.registers[input.index()]);
                        let dst = alloc_audio(&mut codegen);

                        (Instruction::NegAudio { dst, src }, Operand::Audio(dst))
                    }
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
                            Operand::Scalar(dst),
                        )
                    }
                    ValueType::Sig => {
                        let lhs = codegen.registers[left.index()];
                        let rhs = codegen.registers[right.index()];
                        let dst = alloc_audio(&mut codegen);

                        (
                            Instruction::BinaryAudio { op, dst, lhs, rhs },
                            Operand::Audio(dst),
                        )
                    }
                };

                (result, instruction)
            }
            NodeOp::Sine(frequency) => {
                let freq = codegen.registers[frequency.index()];
                let dst = alloc_audio(&mut codegen);
                let state = alloc_state(&mut codegen);
                let instruction = Instruction::Sine { dst, freq, state };
                let result = Operand::Audio(dst);

                (result, instruction)
            }
        };

        codegen.registers.push(result);
        codegen.instructions.push(instruction);
        codegen.debug_origins.push(node.origin.clone());
    }

    let output_source = audio_input(codegen.registers[graph.output.index()]);
    let output = alloc_audio(&mut codegen);

    codegen
        .instructions
        .push(Instruction::SetOutput { src: output_source });
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
