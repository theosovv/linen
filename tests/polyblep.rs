mod common;
#[path = "../test-support/reference/mod.rs"]
mod reference;

use common::{assert_close, assert_primary, builtin, compile_program, lift, number, typed_program};
use linen::bytecode::{
    BytecodeVerifyError, Instruction, OutputLayout, StateId, codegen, verify as verify_bytecode,
    verify_origins,
};
use linen::dsp::{SawPolyblep, SquarePolyblep};
use linen::ir::{
    Graph, IrVerifyError, Node, NodeId, NodeOp, NodeOrigin, ValueType, verify as verify_ir,
};
use linen::sema::{
    BuiltinId, ResourceModel, RuntimeCapability, Statefulness, Type, builtin_by_name,
    builtin_descriptor,
};
use linen::{ErrorCode, RenderConfig, SourceMap, SourceSpan, check, compile, render};
use reference::ReferenceInterpreter;

const ALIAS_BINS: [usize; 12] = [
    48, 96, 144, 857, 905, 953, 1049, 1097, 1145, 1858, 1906, 1954,
];

fn oscillator_program(
    builtin_id: BuiltinId,
    frequency: f64,
    frames: u64,
) -> linen::sema::TypedProgram {
    typed_program(
        builtin(builtin_id, vec![lift(number(frequency))], Type::Sig),
        frames,
    )
}

fn saw_dt_03_vector() -> [f32; 10] {
    [
        0.0,
        -0.4,
        0.2,
        16.0 / 45.0,
        -22.0 / 45.0,
        0.0,
        22.0 / 45.0,
        -16.0 / 45.0,
        -0.2,
        0.4,
    ]
}

fn square_dt_03_vector() -> [f32; 10] {
    [
        0.0,
        8.0 / 9.0,
        -5.0 / 9.0,
        -5.0 / 9.0,
        8.0 / 9.0,
        0.0,
        -8.0 / 9.0,
        5.0 / 9.0,
        5.0 / 9.0,
        -8.0 / 9.0,
    ]
}

fn dt_03_vector(builtin_id: BuiltinId) -> [f32; 10] {
    match builtin_id {
        BuiltinId::SAW_POLYBLEP => saw_dt_03_vector(),
        BuiltinId::SQUARE_POLYBLEP => square_dt_03_vector(),
        _ => panic!("expected a PolyBLEP oscillator"),
    }
}

fn assert_reference_vector(
    builtin_id: BuiltinId,
    frequency: f64,
    sample_rate: u32,
    expected: &[f32],
) {
    let program = oscillator_program(builtin_id, frequency, expected.len() as u64);
    let mut reference = ReferenceInterpreter::new(&program);
    for (index, expected) in expected.iter().enumerate() {
        let actual = reference.next_sample(index as u64, sample_rate).unwrap();
        assert_close(actual, *expected, index);
    }
}

fn compile_at_sample_rate(
    source: &str,
    sample_rate: u32,
    block_size: usize,
) -> linen::CompiledProgram {
    compile(
        source,
        RenderConfig {
            sample_rate,
            block_size,
            ..RenderConfig::default()
        },
    )
    .unwrap()
    .result
    .expect("the PolyBLEP source must compile")
}

fn manual_graph(op: NodeOp) -> (SourceMap, Graph) {
    let mut sources = SourceMap::new();
    let source = sources.add_source("polyblep.ln", "0");
    let span = SourceSpan::new(source, 0, 1).unwrap();
    let node = |index, op, result_type| Node {
        id: NodeId::new(index),
        op,
        result_type,
        origin: NodeOrigin::new(span),
    };
    let graph = Graph {
        nodes: vec![
            node(0, NodeOp::ConstFloat(1_000.0), ValueType::Float),
            node(1, NodeOp::LiftFloatToSig(NodeId::new(0)), ValueType::Sig),
            node(2, op, ValueType::Sig),
        ],
        roots: vec![NodeId::new(2)],
        output: NodeId::new(2),
        render_samples: 8,
    };
    (sources, graph)
}

fn alias_energy(samples: &[f32]) -> f64 {
    let frame_count = samples.len() as f64;
    let mut energy = 0.0;
    for bin in ALIAS_BINS {
        let mut real = 0.0;
        let mut imaginary = 0.0;
        for (index, sample) in samples.iter().enumerate() {
            let angle = -std::f64::consts::TAU * bin as f64 * index as f64 / frame_count;
            let (sin, cos) = angle.sin_cos();
            let sample = f64::from(*sample);
            real += sample * cos;
            imaginary += sample * sin;
        }
        energy += (real * real + imaginary * imaginary) / (frame_count * frame_count);
    }
    energy
}

#[test]
fn spec_4_8_4_polyblep_descriptors_typing_and_name_compatibility() {
    for (id, name) in [
        (BuiltinId::SAW_POLYBLEP, "saw_polyblep"),
        (BuiltinId::SQUARE_POLYBLEP, "square_polyblep"),
    ] {
        let descriptor = builtin_descriptor(id).unwrap();
        assert_eq!(descriptor.id, id);
        assert_eq!(descriptor.name, name);
        assert_eq!(builtin_by_name(name), Some(descriptor));
        assert_eq!(descriptor.signature.parameters, &[Type::Sig]);
        assert_eq!(descriptor.signature.result, Type::Sig);
        assert_eq!(descriptor.statefulness, Statefulness::Stateful);
        assert_eq!(descriptor.capability, RuntimeCapability::CoreAudio);
        assert_eq!(
            descriptor.resources,
            ResourceModel {
                scalar_registers: 0,
                audio_registers: 1,
                state_slots: 1,
                state_bytes: size_of::<f64>(),
            }
        );
    }

    let wrong_arity = "let out = saw_polyblep(); render 1ms;";
    let diagnostic = check(wrong_arity).result.unwrap_err();
    assert_eq!(diagnostic.code(), ErrorCode::InvalidArity);
    assert_primary(wrong_arity, &diagnostic, "saw_polyblep");

    let wrong_type = "let out = square_polyblep(stereo(1, 1)); render 1ms;";
    let diagnostic = check(wrong_type).result.unwrap_err();
    assert_eq!(diagnostic.code(), ErrorCode::TypeMismatch);
    assert_primary(wrong_type, &diagnostic, "stereo(1, 1)");

    let value_binding = "let saw_polyblep = 480; let out = sine(saw_polyblep); render 1ms;";
    assert!(check(value_binding).result.is_ok());

    let conflict = include_str!("../fixtures/invalid/stage-05d-builtin-function-conflict.ln");
    let diagnostic = check(conflict).result.unwrap_err();
    assert_eq!(diagnostic.code(), ErrorCode::DuplicateBinding);
    assert_primary(conflict, &diagnostic, "square_polyblep");
}

#[test]
fn adr_0016_reference_matches_fixed_saw_and_square_vectors() {
    assert_reference_vector(
        BuiltinId::SAW_POLYBLEP,
        3_000.0,
        10_000,
        &saw_dt_03_vector(),
    );
    assert_reference_vector(
        BuiltinId::SQUARE_POLYBLEP,
        3_000.0,
        10_000,
        &square_dt_03_vector(),
    );
}

#[test]
fn adr_0016_reference_covers_strict_boundaries_and_frequency_branches() {
    for (id, expected) in [
        (
            BuiltinId::SAW_POLYBLEP,
            [0.0, -0.5, 0.0, 0.5, 0.0, -0.5, 0.0, 0.5],
        ),
        (
            BuiltinId::SQUARE_POLYBLEP,
            [0.0, 1.0, 0.0, -1.0, 0.0, 1.0, 0.0, -1.0],
        ),
    ] {
        assert_reference_vector(id, 2_000.0, 8_000, &expected);
    }

    for (id, initial) in [
        (BuiltinId::SAW_POLYBLEP, -1.0),
        (BuiltinId::SQUARE_POLYBLEP, 1.0),
    ] {
        assert_reference_vector(id, 0.0, 8_000, &[initial; 8]);
        assert_reference_vector(id, 4_000.0, 8_000, &[0.0; 8]);
    }

    assert_reference_vector(
        BuiltinId::SAW_POLYBLEP,
        -3_000.0,
        10_000,
        &[
            0.0,
            0.4,
            -0.2,
            -16.0 / 45.0,
            22.0 / 45.0,
            0.0,
            -22.0 / 45.0,
            16.0 / 45.0,
        ],
    );
    assert_reference_vector(
        BuiltinId::SQUARE_POLYBLEP,
        -3_000.0,
        10_000,
        &[
            0.0,
            -8.0 / 9.0,
            5.0 / 9.0,
            5.0 / 9.0,
            -8.0 / 9.0,
            0.0,
            8.0 / 9.0,
            -5.0 / 9.0,
        ],
    );

    for (polyblep, naive) in [
        (BuiltinId::SAW_POLYBLEP, BuiltinId::SAW),
        (BuiltinId::SQUARE_POLYBLEP, BuiltinId::SQUARE),
    ] {
        let polyblep_program = oscillator_program(polyblep, 25_000.0, 48);
        let naive_program = oscillator_program(naive, 25_000.0, 48);
        let mut polyblep_reference = ReferenceInterpreter::new(&polyblep_program);
        let mut naive_reference = ReferenceInterpreter::new(&naive_program);
        for index in 0..48 {
            let actual = polyblep_reference.next_sample(index, 48_000).unwrap();
            let expected = naive_reference.next_sample(index, 48_000).unwrap();
            assert_close(actual, expected, index as usize);
        }
    }
}

#[test]
fn adr_0016_square_shifted_phase_rounding_uses_unwrapped_right_branch() {
    let phase = f64::from_bits(0.5_f64.to_bits() - 1);
    let sample_rate = 1_000_000;
    let frequency = phase * f64::from(sample_rate);
    assert!(phase < 0.5);
    assert_eq!(phase + 0.5, 1.0);
    assert_eq!(frequency / f64::from(sample_rate), phase);

    let mut square = SquarePolyblep::new();
    assert_eq!(square.next_sample(frequency, sample_rate).unwrap(), 0.0);
    assert_eq!(square.phase(), phase);
    assert_eq!(square.next_sample(frequency, sample_rate).unwrap(), 0.0);
}

#[test]
fn adr_0016_production_matches_fixed_vectors_at_required_block_sizes() {
    for id in [BuiltinId::SAW_POLYBLEP, BuiltinId::SQUARE_POLYBLEP] {
        let name = builtin_descriptor(id).unwrap().name;
        let source = format!("let out = {name}(3000); render 1ms;");
        let expected = dt_03_vector(id);
        for block_size in [1, 3, 64, 128, 256] {
            let program = compile_at_sample_rate(&source, 10_000, block_size);
            let actual = common::vm_values(&program);
            assert_eq!(actual.len(), expected.len());
            for (index, (actual, expected)) in actual.iter().zip(&expected).enumerate() {
                assert_close(*actual, *expected, index);
            }
            common::assert_reset_repeats(&program, &actual);
        }
    }
}

#[test]
fn adr_0016_production_frequency_edges_match_independent_reference() {
    for id in [BuiltinId::SAW_POLYBLEP, BuiltinId::SQUARE_POLYBLEP] {
        let name = builtin_descriptor(id).unwrap().name;
        for frequency in [0.0, -3_000.0, 4_000.0, 9_000.0] {
            let source = format!("let out = {name}({frequency}); render 1ms;");
            let program = compile_at_sample_rate(&source, 8_000, 3);
            let actual = common::vm_values(&program);
            let mut reference = ReferenceInterpreter::new(&program.typed);
            for (index, actual) in actual.into_iter().enumerate() {
                let expected = reference.next_sample(index as u64, 8_000).unwrap();
                assert_close(actual, expected, index);
            }
        }
    }
}

#[test]
fn adr_0016_nonfinite_frequency_preserves_phase_and_reset_restores_state() {
    let mut saw = SawPolyblep::new();
    let mut square = SquarePolyblep::new();
    assert!(saw.next_sample(f64::NAN, 48_000).is_err());
    assert!(square.next_sample(f64::INFINITY, 48_000).is_err());
    assert_eq!(saw.phase(), 0.0);
    assert_eq!(square.phase(), 0.0);

    saw.next_sample(1_000.0, 8_000).unwrap();
    square.next_sample(-1_000.0, 8_000).unwrap();
    let saw_phase = saw.phase();
    let square_phase = square.phase();
    assert_ne!(saw_phase, 0.0);
    assert_ne!(square_phase, 0.0);
    assert!(saw.next_sample(f64::NEG_INFINITY, 8_000).is_err());
    assert!(square.next_sample(f64::NAN, 8_000).is_err());
    assert_eq!(saw.phase(), saw_phase);
    assert_eq!(square.phase(), square_phase);

    for _ in 0..100_000 {
        saw.next_sample(1_000.0, 8_000).unwrap();
        square.next_sample(-1_000.0, 8_000).unwrap();
    }
    for phase in [saw.phase(), square.phase()] {
        assert!(phase.is_finite() && (0.0..1.0).contains(&phase));
    }

    saw.reset();
    square.reset();
    assert_eq!(saw.phase(), 0.0);
    assert_eq!(square.phase(), 0.0);
}

#[test]
fn adr_0016_runtime_frequency_diagnostic_keeps_code_span_and_frame() {
    let source = include_str!("../fixtures/invalid/stage-05d-polyblep-frequency.ln");
    let program = compile_program(source, 3);
    let error = render(&program, &mut Vec::new()).unwrap_err();
    let linen::RenderError::Diagnostic(diagnostic) = error else {
        panic!("frequency failure must be a source diagnostic")
    };
    assert_eq!(diagnostic.code(), ErrorCode::NonFiniteOutput);
    assert_eq!(diagnostic.sample_index(), Some(0));
    assert_primary(source, &diagnostic, "zero / zero");
}

#[test]
fn adr_0016_ir_bytecode_resources_and_verifier_corruption_are_explicit() {
    for (id, op) in [
        (BuiltinId::SAW_POLYBLEP, NodeOp::SawPolyblep(NodeId::new(1))),
        (
            BuiltinId::SQUARE_POLYBLEP,
            NodeOp::SquarePolyblep(NodeId::new(1)),
        ),
    ] {
        let (sources, graph) = manual_graph(op.clone());
        verify_ir(&graph, &sources).unwrap();
        assert_eq!(
            op.descriptor(ValueType::Sig).resources,
            builtin_descriptor(id).unwrap().resources
        );
        assert_eq!(
            op.descriptor(ValueType::Sig).capability,
            RuntimeCapability::CoreAudio
        );
        let bytecode = codegen(&graph, NodeOrigin::new(graph.nodes[2].origin.primary));
        verify_bytecode(&bytecode).unwrap();
        verify_origins(&bytecode, &sources).unwrap();
        assert_eq!(bytecode.state_count, 1);
        assert!(matches!(bytecode.output, OutputLayout::Mono(_)));
        match (id, &bytecode.instructions[2]) {
            (BuiltinId::SAW_POLYBLEP, Instruction::SawPolyblep { .. })
            | (BuiltinId::SQUARE_POLYBLEP, Instruction::SquarePolyblep { .. }) => {}
            _ => panic!("PolyBLEP IR must produce its own bytecode instruction"),
        }

        let mut wrong_type = graph.clone();
        wrong_type.nodes[2].result_type = ValueType::Float;
        assert!(matches!(
            verify_ir(&wrong_type, &sources),
            Err(IrVerifyError::ResultTypeMismatch { .. })
        ));

        let mut unknown_state = bytecode;
        match (id, &mut unknown_state.instructions[2]) {
            (BuiltinId::SAW_POLYBLEP, Instruction::SawPolyblep { state, .. })
            | (BuiltinId::SQUARE_POLYBLEP, Instruction::SquarePolyblep { state, .. }) => {
                *state = StateId::new(1);
            }
            _ => panic!("matched PolyBLEP instruction"),
        }
        assert!(matches!(
            verify_bytecode(&unknown_state),
            Err(BytecodeVerifyError::UnknownState {
                instruction: 2,
                state: StateId { .. },
            })
        ));
    }
}

#[test]
fn adr_0016_function_state_origins_partitions_and_debug_stripping() {
    let source = include_str!("../fixtures/valid/stage-05d-polyblep.ln");
    let program = compile_program(source, 7);
    assert_eq!(program.bytecode.state_count, 4);
    assert_eq!(
        program
            .ir
            .nodes
            .iter()
            .filter(|node| matches!(node.op, NodeOp::SawPolyblep(_)))
            .count(),
        2
    );
    assert_eq!(
        program
            .ir
            .nodes
            .iter()
            .filter(|node| matches!(node.op, NodeOp::SquarePolyblep(_)))
            .count(),
        2
    );
    for node in program
        .ir
        .nodes
        .iter()
        .filter(|node| matches!(node.op, NodeOp::SawPolyblep(_) | NodeOp::SquarePolyblep(_)))
    {
        let primary = node.origin.primary;
        let primary_text = &source[primary.start()..primary.end()];
        let expected_primary = match node.op {
            NodeOp::SawPolyblep(_) => "frequency",
            NodeOp::SquarePolyblep(_) => "frequency * 0.5",
            _ => unreachable!(),
        };
        assert_eq!(primary_text, expected_primary);
        assert_eq!(node.origin.inlined_from.len(), 1);
        let call = node.origin.inlined_from[0].call_span;
        assert!(matches!(
            &source[call.start()..call.end()],
            "voice(3000)" | "voice(1000)"
        ));
    }

    let expected = common::vm_values(&program);
    let mut reference = ReferenceInterpreter::new(&program.typed);
    for (index, actual) in expected.iter().enumerate() {
        assert_close(
            *actual,
            reference.next_sample(index as u64, 48_000).unwrap(),
            index,
        );
    }
    common::assert_reset_repeats(&program, &expected);

    let mut stripped = compile_program(source, 3);
    stripped.bytecode.strip_debug_info();
    let actual = common::vm_values(&stripped);
    for (index, (actual, expected)) in actual.iter().zip(&expected).enumerate() {
        assert_close(*actual, *expected, index);
    }
}

#[test]
fn adr_0016_polyblep_reduces_fixed_alias_energy_by_at_least_20_db() {
    for (naive, polyblep) in [("saw", "saw_polyblep"), ("square", "square_polyblep")] {
        let naive_source = format!("let out = {naive}(1001); render 1s;");
        let polyblep_source = format!("let out = {polyblep}(1001); render 1s;");
        let naive_samples = common::vm_values(&compile_program(&naive_source, 256));
        let polyblep_samples = common::vm_values(&compile_program(&polyblep_source, 256));
        assert_eq!(polyblep_samples.len(), 48_000);
        let naive_energy = alias_energy(&naive_samples);
        let polyblep_energy = alias_energy(&polyblep_samples);
        assert!(naive_energy > 0.0 && polyblep_energy > 0.0);
        let reduction_db = 10.0 * (naive_energy / polyblep_energy).log10();
        assert!(
            reduction_db >= 20.0,
            "{polyblep}: alias reduction {reduction_db} dB, naive {naive_energy}, PolyBLEP {polyblep_energy}"
        );
    }
}

#[test]
fn adr_0016_scalar_frequency_operand_matches_audio_operand_reference() {
    use linen::bytecode::{
        AudioRegister, Bytecode, Instruction, Operand, OutputLayout, ScalarRegister, StateId,
    };
    use linen::vm::{Vm, VmConfig};

    for id in [BuiltinId::SAW_POLYBLEP, BuiltinId::SQUARE_POLYBLEP] {
        let oscillator = |state| match id {
            BuiltinId::SAW_POLYBLEP => Instruction::SawPolyblep {
                dst: AudioRegister::new(0),
                freq: Operand::Scalar(ScalarRegister::new(0)),
                state,
            },
            BuiltinId::SQUARE_POLYBLEP => Instruction::SquarePolyblep {
                dst: AudioRegister::new(0),
                freq: Operand::Scalar(ScalarRegister::new(0)),
                state,
            },
            _ => panic!("expected a PolyBLEP oscillator"),
        };
        let bytecode = Bytecode {
            instructions: vec![
                Instruction::ConstScalar {
                    dst: ScalarRegister::new(0),
                    value: 3_000.0,
                },
                oscillator(StateId::new(0)),
                Instruction::SetOutput {
                    src: AudioRegister::new(0),
                },
            ],
            scalar_register_count: 1,
            audio_register_count: 2,
            state_count: 1,
            output: OutputLayout::Mono(AudioRegister::new(1)),
            debug_origins: None,
        };
        let mut vm = Vm::new(
            &bytecode,
            VmConfig {
                sample_rate: 10_000,
                block_size: 3,
            },
        )
        .unwrap();
        let reference_program = oscillator_program(id, 3_000.0, 8);
        let mut reference = ReferenceInterpreter::new(&reference_program);
        let mut output = [0.0; 8];
        for block in output.chunks_mut(3) {
            vm.render_into(block).unwrap();
        }
        for (index, actual) in output.into_iter().enumerate() {
            let expected = reference.next_sample(index as u64, 10_000).unwrap();
            assert_close(actual, expected, index);
        }
    }
}
