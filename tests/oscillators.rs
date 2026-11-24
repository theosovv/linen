mod common;
#[path = "../test-support/reference/generator.rs"]
mod generator;
#[path = "../test-support/reference/mod.rs"]
mod reference;

use std::collections::BTreeSet;

use common::{
    assert_close, assert_primary, builtin, compile_program, lift, number, typed_program, vm_values,
};
use linen::bytecode::{Instruction, OutputLayout, codegen, verify as verify_bytecode};
use linen::dsp::{Phasor, Saw, Square, Triangle};
use linen::ir::{Graph, Node, NodeId, NodeOp, NodeOrigin, ValueType, verify as verify_ir};
use linen::sema::{
    BUILTINS, BuiltinId, ResourceModel, RuntimeCapability, Statefulness, Type, builtin_descriptor,
};
use linen::{ErrorCode, RenderConfig, SourceMap, SourceSpan, check, compile, create_vm, render};
use reference::ReferenceInterpreter;

fn oscillator_program(builtin_id: BuiltinId, frequency: f64) -> linen::sema::TypedProgram {
    typed_program(
        builtin(builtin_id, vec![lift(number(frequency))], Type::Sig),
        16,
    )
}

fn analytic_waveforms(builtin_id: BuiltinId) -> [f32; 8] {
    match builtin_id {
        BuiltinId::PHASOR => [0.0, 0.125, 0.25, 0.375, 0.5, 0.625, 0.75, 0.875],
        BuiltinId::SAW => [-1.0, -0.75, -0.5, -0.25, 0.0, 0.25, 0.5, 0.75],
        BuiltinId::SQUARE => [1.0, 1.0, 1.0, 1.0, -1.0, -1.0, -1.0, -1.0],
        BuiltinId::TRIANGLE => [-1.0, -0.5, 0.0, 0.5, 1.0, 0.5, 0.0, -0.5],
        _ => panic!("expected a stage 5 phase oscillator"),
    }
}

#[test]
fn spec_5_5_stage_5_builtins_have_unique_complete_descriptors() {
    assert_eq!(BUILTINS.len(), 12);
    let mut names = BTreeSet::new();
    for (index, descriptor) in BUILTINS.iter().enumerate() {
        assert_eq!(descriptor.id.index(), index);
        assert!(names.insert(descriptor.name));
        assert_eq!(descriptor.documentation_name, descriptor.name);
        assert_eq!(builtin_descriptor(descriptor.id), Some(descriptor));
    }

    for id in [
        BuiltinId::SINE,
        BuiltinId::PHASOR,
        BuiltinId::SAW,
        BuiltinId::SQUARE,
        BuiltinId::TRIANGLE,
        BuiltinId::SAW_POLYBLEP,
        BuiltinId::SQUARE_POLYBLEP,
    ] {
        let descriptor = builtin_descriptor(id).unwrap();
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
    assert_eq!(
        builtin_descriptor(BuiltinId::GAIN)
            .unwrap()
            .signature
            .parameters,
        &[Type::Sig; 2]
    );
    assert_eq!(
        builtin_descriptor(BuiltinId::MIX)
            .unwrap()
            .signature
            .parameters,
        &[Type::Sig; 3]
    );
    assert_eq!(
        builtin_descriptor(BuiltinId::NOISE)
            .unwrap()
            .signature
            .parameters,
        &[Type::Float]
    );
}

#[test]
fn spec_0_6_builtin_function_names_are_an_explicit_compatibility_break() {
    for descriptor in &BUILTINS {
        let source = format!(
            "let {} = (x: Sig): Sig => x; let out = 1; render 1ms;",
            descriptor.name
        );
        let diagnostic = check(&source)
            .result
            .expect_err("builtin replacement must fail");
        assert_eq!(diagnostic.code(), ErrorCode::DuplicateBinding);
        assert_primary(&source, &diagnostic, descriptor.name);
    }

    let source = include_str!("../fixtures/invalid/stage-05-builtin-function-conflict.ln");
    let diagnostic = check(source)
        .result
        .expect_err("legacy mix declaration must fail");
    assert_eq!(diagnostic.code(), ErrorCode::DuplicateBinding);
    assert_primary(source, &diagnostic, "mix");
}

#[test]
fn adr_0013_reference_waveforms_match_explicit_phase_vectors_and_edges() {
    for id in [
        BuiltinId::PHASOR,
        BuiltinId::SAW,
        BuiltinId::SQUARE,
        BuiltinId::TRIANGLE,
    ] {
        let program = oscillator_program(id, 1_000.0);
        let mut reference = ReferenceInterpreter::new(&program);
        for (index, expected) in analytic_waveforms(id).into_iter().enumerate() {
            let actual = reference.next_sample(index as u64, 8_000).unwrap();
            assert_close(actual, expected, index);
        }

        let zero = oscillator_program(id, 0.0);
        let mut reference = ReferenceInterpreter::new(&zero);
        for index in 0..8 {
            assert_close(
                reference.next_sample(index, 8_000).unwrap(),
                analytic_waveforms(id)[0],
                index as usize,
            );
        }
    }

    let negative = oscillator_program(BuiltinId::PHASOR, -1_000.0);
    let mut reference = ReferenceInterpreter::new(&negative);
    for (index, expected) in [0.0, 0.875, 0.75, 0.625, 0.5].into_iter().enumerate() {
        assert_close(
            reference.next_sample(index as u64, 8_000).unwrap(),
            expected,
            index,
        );
    }
    let high = oscillator_program(BuiltinId::PHASOR, 9_000.0);
    let mut reference = ReferenceInterpreter::new(&high);
    for (index, expected) in analytic_waveforms(BuiltinId::PHASOR)[..5]
        .iter()
        .enumerate()
    {
        assert_close(
            reference.next_sample(index as u64, 8_000).unwrap(),
            *expected,
            index,
        );
    }
}

#[test]
fn adr_0013_production_waveforms_match_vectors_at_block_boundaries() {
    for id in [
        BuiltinId::PHASOR,
        BuiltinId::SAW,
        BuiltinId::SQUARE,
        BuiltinId::TRIANGLE,
    ] {
        let name = builtin_descriptor(id).unwrap().name;
        let source = format!("let out = {name}(1000); render 1ms;");
        for block_size in [1, 3, 64, 128, 256] {
            let outcome = compile(
                &source,
                RenderConfig {
                    sample_rate: 8_000,
                    block_size,
                    ..RenderConfig::default()
                },
            )
            .unwrap();
            let program = outcome.result.expect("the oscillator source must compile");
            let actual = vm_values(&program);
            assert_eq!(actual.len(), 8);
            for (index, expected) in analytic_waveforms(id).into_iter().enumerate() {
                assert_close(actual[index], expected, index);
            }
            common::assert_reset_repeats(&program, &actual);
        }
    }
}

#[test]
fn adr_0013_nonfinite_frequency_preserves_phase_and_runtime_diagnostic() {
    let mut phasor = Phasor::new();
    let mut saw = Saw::new();
    let mut square = Square::new();
    let mut triangle = Triangle::new();
    for result in [
        phasor.next_sample(f64::NAN, 48_000).map(|_| ()),
        saw.next_sample(f64::INFINITY, 48_000).map(|_| ()),
        square.next_sample(f64::NEG_INFINITY, 48_000).map(|_| ()),
        triangle.next_sample(f64::NAN, 48_000).map(|_| ()),
    ] {
        assert!(result.is_err());
    }
    assert_eq!(phasor.phase(), 0.0);
    assert_eq!(saw.phase(), 0.0);
    assert_eq!(square.phase(), 0.0);
    assert_eq!(triangle.phase(), 0.0);

    for _ in 0..100_000 {
        phasor.next_sample(1_000.0, 8_000).unwrap();
        saw.next_sample(1_000.0, 8_000).unwrap();
        square.next_sample(1_000.0, 8_000).unwrap();
        triangle.next_sample(1_000.0, 8_000).unwrap();
    }
    for phase in [
        phasor.phase(),
        saw.phase(),
        square.phase(),
        triangle.phase(),
    ] {
        assert!(phase.is_finite() && (0.0..1.0).contains(&phase));
    }

    for id in [
        BuiltinId::PHASOR,
        BuiltinId::SAW,
        BuiltinId::SQUARE,
        BuiltinId::TRIANGLE,
    ] {
        let name = builtin_descriptor(id).unwrap().name;
        let source = format!("let zero = sine(0); let out = {name}(zero / zero); render 1ms;");
        let program = compile_program(&source, 128);
        let error = render(&program, &mut Vec::new()).expect_err("NaN frequency must fail");
        let linen::RenderError::Diagnostic(diagnostic) = error else {
            panic!("runtime frequency error must be source-aware")
        };
        assert_eq!(diagnostic.code(), ErrorCode::NonFiniteOutput);
        assert_eq!(diagnostic.sample_index(), Some(0));
        assert_primary(&source, &diagnostic, "zero / zero");
    }
}

#[test]
fn adr_0013_production_frequency_edges_match_reference() {
    for id in [
        BuiltinId::PHASOR,
        BuiltinId::SAW,
        BuiltinId::SQUARE,
        BuiltinId::TRIANGLE,
    ] {
        let name = builtin_descriptor(id).unwrap().name;
        for frequency in [0.0, -1_000.0, 25_000.0] {
            let source = format!("let out = {name}({frequency}); render 1ms;");
            let program = compile_program(&source, 3);
            let actual = vm_values(&program);
            let mut reference = ReferenceInterpreter::new(&program.typed);
            for (index, actual) in actual.into_iter().enumerate() {
                let expected = reference.next_sample(index as u64, 48_000).unwrap();
                assert_close(actual, expected, index);
            }
        }
    }
}

#[test]
fn adr_0013_gain_and_mix_match_exact_arithmetic_vectors() {
    let gain = typed_program(
        builtin(
            BuiltinId::GAIN,
            vec![lift(number(0.5)), lift(number(-0.25))],
            Type::Sig,
        ),
        1,
    );
    assert_eq!(
        ReferenceInterpreter::new(&gain)
            .next_sample(0, 48_000)
            .unwrap(),
        -0.125
    );
    let production = compile_program("let out = gain(0.5, -0.25); render 1ms;", 3);
    assert_eq!(vm_values(&production)[0], -0.125);

    for (balance, expected) in [
        (-1.0, -0.25),
        (0.0, 0.25),
        (0.5, 0.5),
        (1.0, 0.75),
        (2.0, 1.25),
    ] {
        let mix = typed_program(
            builtin(
                BuiltinId::MIX,
                vec![
                    lift(number(0.25)),
                    lift(number(0.75)),
                    lift(number(balance)),
                ],
                Type::Sig,
            ),
            1,
        );
        assert_eq!(
            ReferenceInterpreter::new(&mix)
                .next_sample(0, 48_000)
                .unwrap(),
            expected
        );

        let source = format!("let out = mix(0.25, 0.75, {balance}); render 1ms;");
        let production = compile_program(&source, 3);
        assert_eq!(vm_values(&production)[0], expected);
    }
}

#[test]
fn adr_0013_gain_mix_lower_with_one_time_args_and_reference_parity() {
    let source = "let balance = saw(1); let out = mix(sine(10), sine(20), balance) |> gain(0.1); render 1ms;";
    let program = compile_program(source, 3);
    assert_eq!(
        program
            .ir
            .nodes
            .iter()
            .filter(|node| matches!(node.op, NodeOp::Saw(_)))
            .count(),
        1
    );
    assert_eq!(
        program
            .ir
            .nodes
            .iter()
            .filter(|node| matches!(node.op, NodeOp::Sine(_)))
            .count(),
        2
    );
    assert_eq!(program.bytecode.state_count, 3);

    let actual = vm_values(&program);
    let mut reference = ReferenceInterpreter::new(&program.typed);
    for (index, actual) in actual.into_iter().enumerate() {
        assert_close(
            actual,
            reference.next_sample(index as u64, 48_000).unwrap(),
            index,
        );
    }
}

#[test]
fn adr_0013_function_calls_fresh_state_reset_and_fixture_match_reference() {
    let source = "let voice = (f: Sig): Sig => saw(f); let shared = voice(100); let out = voice(100) + shared; render 1ms;";
    let program = compile_program(source, 3);
    assert_eq!(
        program
            .ir
            .nodes
            .iter()
            .filter(|node| matches!(node.op, NodeOp::Saw(_)))
            .count(),
        2
    );
    let first = vm_values(&program);
    let mut vm = create_vm(&program).unwrap();
    let mut repeated = vec![0.0; first.len()];
    for block in repeated.chunks_mut(program.config.block_size) {
        vm.render_into(block).unwrap();
    }

    vm.reset();

    let mut after_reset = vec![0.0; repeated.len()];

    for block in after_reset.chunks_mut(program.config.block_size) {
        vm.render_into(block).unwrap();
    }

    assert_eq!(repeated, after_reset);

    let fixture = compile_program(include_str!("../fixtures/valid/stage-05-oscillators.ln"), 3);
    let actual = vm_values(&fixture);
    let mut reference = ReferenceInterpreter::new(&fixture.typed);
    for (index, actual) in actual.into_iter().enumerate() {
        assert_close(
            actual,
            reference.next_sample(index as u64, 48_000).unwrap(),
            index,
        );
    }
}

#[test]
fn adr_0013_phase_ir_codegen_verifiers_and_resources_are_explicit() {
    let mut sources = SourceMap::new();
    let source = sources.add_source("phase.ln", "0");
    let span = SourceSpan::new(source, 0, 1).unwrap();
    for (op, id) in [
        (NodeOp::Phasor(NodeId::new(1)), BuiltinId::PHASOR),
        (NodeOp::Saw(NodeId::new(1)), BuiltinId::SAW),
        (NodeOp::Square(NodeId::new(1)), BuiltinId::SQUARE),
        (NodeOp::Triangle(NodeId::new(1)), BuiltinId::TRIANGLE),
    ] {
        let graph = Graph {
            nodes: vec![
                Node {
                    id: NodeId::new(0),
                    op: NodeOp::ConstFloat(1_000.0),
                    result_type: ValueType::Float,
                    origin: NodeOrigin::new(span),
                },
                Node {
                    id: NodeId::new(1),
                    op: NodeOp::LiftFloatToSig(NodeId::new(0)),
                    result_type: ValueType::Sig,
                    origin: NodeOrigin::new(span),
                },
                Node {
                    id: NodeId::new(2),
                    op: op.clone(),
                    result_type: ValueType::Sig,
                    origin: NodeOrigin::new(span),
                },
            ],
            roots: vec![NodeId::new(2)],
            output: NodeId::new(2),
            render_samples: 8,
        };
        verify_ir(&graph, &sources).unwrap();
        assert_eq!(op.descriptor(ValueType::Sig).resources.state_slots, 1);
        assert_eq!(
            op.descriptor(ValueType::Sig).resources.state_bytes,
            size_of::<f64>()
        );
        let bytecode = codegen(&graph, NodeOrigin::new(span));
        verify_bytecode(&bytecode).unwrap();
        assert_eq!(bytecode.state_count, 1);
        assert!(matches!(bytecode.output, OutputLayout::Mono(_)));
        assert!(match id {
            BuiltinId::PHASOR => matches!(&bytecode.instructions[2], Instruction::Phasor { .. }),
            BuiltinId::SAW => matches!(&bytecode.instructions[2], Instruction::Saw { .. }),
            BuiltinId::SQUARE => matches!(&bytecode.instructions[2], Instruction::Square { .. }),
            BuiltinId::TRIANGLE =>
                matches!(&bytecode.instructions[2], Instruction::Triangle { .. }),
            _ => false,
        });
        assert_eq!(builtin_descriptor(id).unwrap().resources.state_slots, 1);
    }
}

#[test]
fn spec_7_5_generated_stage_5_mono_corpus_matches_reference() {
    for seed in 0..8 {
        let source = generator::stage5_mono_program(seed);
        let program = compile_program(&source, 1 + seed as usize % 7);
        let actual = vm_values(&program);
        let mut reference = ReferenceInterpreter::new(&program.typed);
        for (index, actual) in actual.into_iter().enumerate() {
            assert_close(
                actual,
                reference.next_sample(index as u64, 48_000).unwrap(),
                index,
            );
        }
    }
}
