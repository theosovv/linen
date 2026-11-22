mod common;
#[path = "../test-support/reference/generator.rs"]
mod generator;
#[path = "../test-support/reference/mod.rs"]
mod reference;

use common::{assert_primary, builtin, compile_program, number, typed_program, vm_values};
use linen::bytecode::{BytecodeVerifyError, OutputLayout, verify as verify_bytecode};
use linen::ir::{
    Graph, IrVerifyError, Node, NodeId, NodeOp, NodeOrigin, RandomStreamId, ValueType,
    verify as verify_ir,
};
use linen::sema::{
    BuiltinId, ResourceModel, RuntimeCapability, Statefulness, Type, builtin_descriptor,
};
use linen::{CompiledProgram, ErrorCode, RenderConfig, SourceMap, check, create_vm};
use reference::{ReferenceFrame, ReferenceInterpreter};

const MAX_SEED: u64 = 9_007_199_254_740_991;

fn expected(seed: u64) -> [f32; 4] {
    match seed {
        0 => [0x3F444150, 0xBE0C3B10, 0xBF727746, 0x3F711770],
        1 => [0x3E085168, 0x3EFBAE34, 0x3F712744, 0xBDE3E7A0],
        MAX_SEED => [0xBF368D62, 0xBF1E781C, 0x3D709C20, 0xBEFA3F14],
        _ => panic!("no independent vector for this seed"),
    }
    .map(f32::from_bits)
}

fn noise_program(seed: u64) -> CompiledProgram {
    let typed = typed_program(
        builtin(BuiltinId::NOISE, vec![number(seed as f64)], Type::Sig),
        4,
    );
    let graph = Graph {
        nodes: vec![
            Node {
                id: NodeId::new(0),
                op: NodeOp::ConstFloat(seed as f64),
                result_type: ValueType::Float,
                origin: NodeOrigin::new(common::span()),
            },
            Node {
                id: NodeId::new(1),
                op: NodeOp::Noise {
                    seed: NodeId::new(0),
                    stream: RandomStreamId::new(seed),
                },
                result_type: ValueType::Sig,
                origin: NodeOrigin::new(common::span()),
            },
        ],
        roots: vec![NodeId::new(1)],
        output: NodeId::new(1),
        render_samples: 4,
    };
    let bytecode = linen::bytecode::codegen(&graph, NodeOrigin::new(common::span()));
    CompiledProgram {
        typed,
        ir: graph,
        bytecode,
        config: RenderConfig::default(),
    }
}

fn reference_stereo(left_seed: u64, right_seed: u64) -> linen::sema::TypedProgram {
    typed_program(
        builtin(
            BuiltinId::STEREO,
            vec![
                builtin(BuiltinId::NOISE, vec![number(left_seed as f64)], Type::Sig),
                builtin(BuiltinId::NOISE, vec![number(right_seed as f64)], Type::Sig),
            ],
            Type::Stereo,
        ),
        4,
    )
}

#[test]
fn adr_0015_noise_descriptor_and_random_stream_id_are_explicit() {
    let descriptor = builtin_descriptor(BuiltinId::NOISE).unwrap();
    assert_eq!(descriptor.signature.parameters, &[Type::Float]);
    assert_eq!(descriptor.signature.result, Type::Sig);
    assert_eq!(descriptor.statefulness, Statefulness::Stateful);
    assert_eq!(descriptor.capability, RuntimeCapability::CoreAudio);
    assert_eq!(
        descriptor.resources,
        ResourceModel {
            scalar_registers: 0,
            audio_registers: 1,
            state_slots: 1,
            state_bytes: 2 * size_of::<u64>(),
        }
    );
    for seed in [0, 1, MAX_SEED] {
        assert_eq!(RandomStreamId::new(seed).seed(), seed);
    }
}

#[test]
fn adr_0015_reference_splitmix64_matches_independent_fixed_bit_vectors() {
    for seed in [0, 1, MAX_SEED] {
        let program = noise_program(seed);
        let mut reference = ReferenceInterpreter::new(&program.typed);
        for (index, expected) in expected(seed).into_iter().enumerate() {
            let actual = reference.next_sample(index as u64, 48_000).unwrap();
            assert_eq!(
                actual.to_bits(),
                expected.to_bits(),
                "seed {seed}, sample {index}"
            );
            assert!((-1.0..1.0).contains(&actual));
        }
    }
}

#[test]
fn adr_0015_seed_boundaries_and_exact_diagnostics() {
    for source in [
        "let out = noise(0); render 1ms;",
        "let out = noise(-0); render 1ms;",
        "let out = noise(9007199254740991); render 1ms;",
        include_str!("../fixtures/regression/stage-05-noise-computed-seed.ln"),
        "let white = (seed: Float): Sig => noise(seed); let out = white(1 + 1); render 1ms;",
    ] {
        let program = compile_program(source, 3);
        let actual = vm_values(&program);
        let mut reference = ReferenceInterpreter::new(&program.typed);
        for (index, actual) in actual.into_iter().enumerate() {
            assert_eq!(
                actual.to_bits(),
                reference
                    .next_sample(index as u64, 48_000)
                    .unwrap()
                    .to_bits()
            );
        }
    }

    for (source, needle) in [
        (
            include_str!("../fixtures/invalid/stage-05-noise-negative.ln"),
            "-1",
        ),
        (
            include_str!("../fixtures/invalid/stage-05-noise-fraction.ln"),
            "0.5",
        ),
        (
            include_str!("../fixtures/invalid/stage-05-noise-seed-overflow.ln"),
            "9007199254740992",
        ),
    ] {
        let diagnostic = check(source)
            .result
            .expect_err("invalid finite seed must fail");
        assert_eq!(diagnostic.code(), ErrorCode::InvalidNoiseSeed);
        assert_primary(source, &diagnostic, needle);
    }

    for (source, code) in [
        (
            "let out = noise(1e999); render 1ms;",
            ErrorCode::InvalidFloat,
        ),
        (
            "let out = noise(1 / 0); render 1ms;",
            ErrorCode::ConstantDivisionByZero,
        ),
    ] {
        assert_eq!(
            check(source)
                .result
                .expect_err("Float error must win")
                .code(),
            code
        );
    }
}

#[test]
fn adr_0015_parameter_seed_error_keeps_definition_primary_and_call_chain() {
    let source = "let white = (seed: Float): Sig => noise(seed); let out = white(0.5); render 1ms;";
    let diagnostic = check(source)
        .result
        .expect_err("concrete invalid seed must fail");
    assert_eq!(diagnostic.code(), ErrorCode::InvalidNoiseSeed);
    let seed_start = source.find("noise(seed)").unwrap() + "noise(".len();
    assert_eq!(
        (
            diagnostic.primary().span().start(),
            diagnostic.primary().span().end()
        ),
        (seed_start, seed_start + "seed".len())
    );
    assert_eq!(diagnostic.secondary().len(), 1);
    let call_start = source.find("white(0.5)").unwrap();
    assert_eq!(
        (
            diagnostic.secondary()[0].span().start(),
            diagnostic.secondary()[0].span().end()
        ),
        (call_start, call_start + "white(0.5)".len())
    );
}

#[test]
fn adr_0015_production_noise_matches_fixed_vectors_reset_partitions_and_strip() {
    for seed in [0, 1, MAX_SEED] {
        let program = noise_program(seed);
        let actual = vm_values(&program);
        for (index, expected) in expected(seed).into_iter().enumerate() {
            assert_eq!(
                actual[index].to_bits(),
                expected.to_bits(),
                "seed {seed}, sample {index}"
            );
        }
        for block_size in [1, 3, 64, 128, 256] {
            let mut blocked = program.clone();
            blocked.config.block_size = block_size;
            assert_eq!(vm_values(&blocked), actual);
        }

        let mut vm = create_vm(&program).unwrap();
        let mut partitioned = Vec::new();
        for length in [1, 2, 1] {
            let mut block = vec![0.0; length];
            vm.render_into(&mut block).unwrap();
            partitioned.extend_from_slice(&block);
        }
        assert_eq!(partitioned, actual);
        vm.reset();
        let mut reset = vec![0.0; 4];
        vm.render_into(&mut reset).unwrap();
        assert_eq!(reset, actual);

        let mut stripped = program.clone();
        stripped.bytecode.strip_debug_info();
        assert_eq!(vm_values(&stripped), actual);
    }
}

#[test]
fn adr_0015_reference_same_seed_matches_and_different_seed_is_independent() {
    let program = reference_stereo(0, 0);
    let mut reference = ReferenceInterpreter::new(&program);
    let mut first = Vec::new();
    for frame in 0..4 {
        let ReferenceFrame::Stereo(channels) = reference.next_frame(frame, 48_000).unwrap() else {
            panic!("stereo reference expected")
        };
        assert_eq!(channels[0].to_bits(), channels[1].to_bits());
        first.push(channels);
    }
    reference.reset();
    for (frame, expected) in first.into_iter().enumerate() {
        let ReferenceFrame::Stereo(actual) = reference.next_frame(frame as u64, 48_000).unwrap()
        else {
            panic!("stereo reference expected")
        };
        assert_eq!(actual, expected);
    }

    let program = reference_stereo(0, 1);
    let mut reference = ReferenceInterpreter::new(&program);
    let ReferenceFrame::Stereo(channels) = reference.next_frame(0, 48_000).unwrap() else {
        panic!("stereo reference expected")
    };
    assert_ne!(channels[0].to_bits(), channels[1].to_bits());
}

#[test]
fn adr_0015_ir_and_bytecode_verifiers_reject_seed_stream_mismatch() {
    let mut sources = SourceMap::new();
    sources.add_source("stage-05-test.ln", "");
    let program = noise_program(0);
    verify_ir(&program.ir, &sources).unwrap();
    verify_bytecode(&program.bytecode).unwrap();

    let mut mismatch = program.ir.clone();
    let NodeOp::Noise { stream, .. } = &mut mismatch.nodes[1].op else {
        panic!("noise node expected")
    };
    *stream = RandomStreamId::new(1);
    assert_eq!(
        verify_ir(&mismatch, &sources),
        Err(IrVerifyError::InvalidNoiseSeed {
            node: NodeId::new(1)
        })
    );

    let mut invalid_seed = program.ir.clone();
    invalid_seed.nodes[0].op = NodeOp::ConstFloat(0.5);
    assert_eq!(
        verify_ir(&invalid_seed, &sources),
        Err(IrVerifyError::InvalidNoiseSeed {
            node: NodeId::new(1)
        })
    );

    let mut invalid_stream = program.bytecode.clone();
    let linen::bytecode::Instruction::Noise { stream, .. } = &mut invalid_stream.instructions[1]
    else {
        panic!("noise instruction expected")
    };
    *stream = RandomStreamId::new(MAX_SEED + 1);
    assert_eq!(
        verify_bytecode(&invalid_stream),
        Err(BytecodeVerifyError::InvalidNoiseStream { instruction: 1 })
    );
    assert!(matches!(program.bytecode.output, OutputLayout::Mono(_)));
}

#[test]
fn spec_7_5_generated_noise_function_fresh_state_and_fixture_match_reference() {
    for seed in 0..8 {
        let source = generator::stage5_noise_program(seed);
        let program = compile_program(&source, 1 + seed as usize % 7);
        let actual = vm_values(&program);
        let mut reference = ReferenceInterpreter::new(&program.typed);
        for frame in 0..program.typed.render.render_samples as usize {
            let ReferenceFrame::Stereo(expected) =
                reference.next_frame(frame as u64, 48_000).unwrap()
            else {
                panic!("generated noise output must be Stereo")
            };
            assert_eq!(actual[frame * 2].to_bits(), expected[0].to_bits());
            assert_eq!(actual[frame * 2 + 1].to_bits(), expected[1].to_bits());
        }
    }

    let source = "let voice = (seed: Float): Sig => noise(seed); let a = voice(7); let b = voice(7); let out = stereo(a, b); render 1ms;";
    let program = compile_program(source, 3);
    assert_eq!(program.bytecode.state_count, 2);
    let actual = vm_values(&program);
    for frame in actual.as_chunks::<2>().0 {
        assert_eq!(frame[0].to_bits(), frame[1].to_bits());
    }

    let fixture = compile_program(include_str!("../fixtures/valid/stage-05-noise.ln"), 3);
    assert_eq!(fixture.bytecode.state_count, 2);
    assert!(matches!(
        fixture.bytecode.output,
        OutputLayout::Stereo { .. }
    ));
    let actual = vm_values(&fixture);
    let mut reference = ReferenceInterpreter::new(&fixture.typed);
    let ReferenceFrame::Stereo(expected) = reference.next_frame(0, 48_000).unwrap() else {
        panic!("fixture output must be Stereo")
    };
    assert_eq!(actual[0].to_bits(), expected[0].to_bits());
    assert_eq!(actual[1].to_bits(), expected[1].to_bits());
}
