mod common;
#[path = "../test-support/reference/generator.rs"]
mod generator;
#[path = "../test-support/reference/mod.rs"]
mod reference;

use common::{
    assert_close, assert_primary, builtin, compile_program, lift, number, typed_program, vm_values,
};
use linen::bytecode::{
    AudioRegister, Bytecode, BytecodeVerifyError, Instruction, Operand, OutputLayout,
    ScalarRegister, codegen, verify as verify_bytecode,
};
use linen::ir::{
    Graph, IrVerifyError, Node, NodeId, NodeOp, NodeOrigin, ValueType, verify as verify_ir,
};
use linen::sema::{FunctionLimits, SymbolStorage, Type, TypedProgram, check_program};
use linen::syntax::{TopDecl, TypeName, parse};
use linen::vm::{Vm, VmConfig, VmRuntimeError};
use linen::{CompiledProgram, ErrorCode, RenderConfig, SourceMap, check, render};
use reference::{ReferenceErrorKind, ReferenceFrame, ReferenceInterpreter};

fn stereo_parts(frames: u64) -> (TypedProgram, Graph) {
    let typed = typed_program(
        builtin(
            linen::sema::BuiltinId::STEREO,
            vec![lift(number(0.25)), lift(number(-0.5))],
            Type::Stereo,
        ),
        frames,
    );
    let origin = NodeOrigin::new(common::span());
    let graph = Graph {
        nodes: vec![
            Node {
                id: NodeId::new(0),
                op: NodeOp::ConstFloat(0.25),
                result_type: ValueType::Float,
                origin: origin.clone(),
            },
            Node {
                id: NodeId::new(1),
                op: NodeOp::LiftFloatToSig(NodeId::new(0)),
                result_type: ValueType::Sig,
                origin: origin.clone(),
            },
            Node {
                id: NodeId::new(2),
                op: NodeOp::ConstFloat(-0.5),
                result_type: ValueType::Float,
                origin: origin.clone(),
            },
            Node {
                id: NodeId::new(3),
                op: NodeOp::LiftFloatToSig(NodeId::new(2)),
                result_type: ValueType::Sig,
                origin: origin.clone(),
            },
            Node {
                id: NodeId::new(4),
                op: NodeOp::Stereo(NodeId::new(1), NodeId::new(3)),
                result_type: ValueType::Stereo,
                origin,
            },
        ],
        roots: vec![NodeId::new(4)],
        output: NodeId::new(4),
        render_samples: frames,
    };
    (typed, graph)
}

fn stereo_program(frames: u64) -> CompiledProgram {
    let (typed, graph) = stereo_parts(frames);
    let bytecode = codegen(&graph, NodeOrigin::new(common::span()));
    CompiledProgram {
        typed,
        ir: graph,
        bytecode,
        config: RenderConfig::default(),
    }
}

fn pan_program(position: f64) -> CompiledProgram {
    let typed = typed_program(
        builtin(
            linen::sema::BuiltinId::PAN,
            vec![lift(number(0.5)), lift(number(position))],
            Type::Stereo,
        ),
        1,
    );
    let origin = NodeOrigin::new(common::span());
    let graph = Graph {
        nodes: vec![
            Node {
                id: NodeId::new(0),
                op: NodeOp::ConstFloat(0.5),
                result_type: ValueType::Float,
                origin: origin.clone(),
            },
            Node {
                id: NodeId::new(1),
                op: NodeOp::LiftFloatToSig(NodeId::new(0)),
                result_type: ValueType::Sig,
                origin: origin.clone(),
            },
            Node {
                id: NodeId::new(2),
                op: NodeOp::ConstFloat(position),
                result_type: ValueType::Float,
                origin: origin.clone(),
            },
            Node {
                id: NodeId::new(3),
                op: NodeOp::LiftFloatToSig(NodeId::new(2)),
                result_type: ValueType::Sig,
                origin: origin.clone(),
            },
            Node {
                id: NodeId::new(4),
                op: NodeOp::Pan(NodeId::new(1), NodeId::new(3)),
                result_type: ValueType::Stereo,
                origin,
            },
        ],
        roots: vec![NodeId::new(4)],
        output: NodeId::new(4),
        render_samples: 1,
    };
    let bytecode = codegen(&graph, NodeOrigin::new(common::span()));
    CompiledProgram {
        typed,
        ir: graph,
        bytecode,
        config: RenderConfig::default(),
    }
}

#[test]
fn spec_4_8_2_stereo_annotations_functions_lifts_and_contextual_identifier() {
    let source = include_str!("../fixtures/valid/stage-05-stereo.ln");
    let mut sources = SourceMap::new();
    let source_id = sources.add_source("stage-05-stereo.ln", source);
    let parsed = parse(source, source_id).unwrap();
    let output = parsed
        .declarations
        .iter()
        .find_map(|declaration| match declaration {
            TopDecl::Let(binding) if binding.name.name == "out" => Some(binding),
            _ => None,
        })
        .unwrap();
    assert_eq!(output.annotation.as_ref().unwrap().kind, TypeName::Stereo);
    let mut symbols = SymbolStorage::new();
    let typed = check_program(&parsed, &mut symbols, 48_000, FunctionLimits::default()).unwrap();
    assert_eq!(typed.bindings().pop().unwrap().value.ty, Type::Stereo);

    let function_source =
        "let route = (x: Stereo): Stereo => x; let out = route(stereo(1, 2)); render 1ms;";
    let mut function_sources = SourceMap::new();
    let function_source_id = function_sources.add_source("stage-05-function.ln", function_source);
    let parsed = parse(function_source, function_source_id).unwrap();
    let mut symbols = SymbolStorage::new();
    let program = check_program(&parsed, &mut symbols, 48_000, FunctionLimits::default()).unwrap();
    assert_eq!(
        program.functions().next().unwrap().parameters[0]
            .annotation
            .kind,
        TypeName::Stereo
    );
    assert_eq!(program.bindings().pop().unwrap().value.ty, Type::Stereo);

    assert!(
        check("let Stereo = 1; let out = Stereo; render 1ms;")
            .result
            .is_ok()
    );
}

#[test]
fn spec_4_8_2_stereo_rejects_arithmetic_and_invalid_output_conversions() {
    for (source, needle) in [
        ("let out: Stereo = 1; render 1ms;", "1"),
        ("let out: Sig = stereo(1, 2); render 1ms;", "stereo(1, 2)"),
        ("let out: Stereo = sine(1); render 1ms;", "sine(1)"),
    ] {
        let diagnostic = check(source)
            .result
            .expect_err("invalid Stereo conversion must fail");
        assert_eq!(diagnostic.code(), ErrorCode::TypeMismatch);
        assert_primary(source, &diagnostic, needle);
    }

    let source = include_str!("../fixtures/invalid/stage-05-stereo-arithmetic.ln");
    let diagnostic = check(source)
        .result
        .expect_err("Stereo arithmetic must fail");
    assert_eq!(diagnostic.code(), ErrorCode::TypeMismatch);
    let start = source.rfind("pair").unwrap();
    assert_eq!(
        (
            diagnostic.primary().span().start(),
            diagnostic.primary().span().end()
        ),
        (start, start + "pair".len())
    );
}

#[test]
fn adr_0014_stereo_ir_codegen_has_explicit_register_and_output_layout() {
    let (typed, graph) = stereo_parts(3);
    assert_eq!(typed.bindings()[0].value.ty, Type::Stereo);
    let mut sources = SourceMap::new();
    sources.add_source("stage-05-test.ln", "");
    verify_ir(&graph, &sources).unwrap();
    let bytecode = codegen(&graph, NodeOrigin::new(common::span()));
    verify_bytecode(&bytecode).unwrap();
    assert_eq!(bytecode.scalar_register_count, 2);
    assert_eq!(bytecode.audio_register_count, 6);
    assert_eq!(bytecode.state_count, 0);
    assert_eq!(
        bytecode.output,
        OutputLayout::Stereo {
            left: AudioRegister::new(4),
            right: AudioRegister::new(5),
        }
    );
    assert_eq!(
        bytecode.instructions[4],
        Instruction::MakeStereo {
            dst_left: AudioRegister::new(2),
            dst_right: AudioRegister::new(3),
            src_left: AudioRegister::new(0),
            src_right: AudioRegister::new(1),
        }
    );
    assert_eq!(
        bytecode.instructions.last(),
        Some(&Instruction::SetStereoOutput {
            src_left: AudioRegister::new(2),
            src_right: AudioRegister::new(3),
        })
    );
}

#[test]
fn adr_0014_vm_interleaves_frames_and_rejects_invalid_slices_before_dispatch() {
    let program = stereo_program(3);
    let mut vm = Vm::new(
        &program.bytecode,
        VmConfig {
            sample_rate: 48_000,
            block_size: 2,
        },
    )
    .unwrap();
    assert_eq!(
        vm.render_into(&mut [0.0; 3]),
        Err(VmRuntimeError::InvalidInterleavedBlock { length: 3 })
    );
    assert_eq!(vm.sample_index(), 0);
    assert_eq!(
        vm.render_into(&mut [0.0; 6]),
        Err(VmRuntimeError::BlockTooLarge {
            requested: 3,
            maximum: 2,
        })
    );
    assert_eq!(vm.sample_index(), 0);

    let mut first = [0.0; 2];
    let mut rest = [0.0; 4];
    vm.render_into(&mut first).unwrap();
    vm.render_into(&mut rest).unwrap();
    assert_eq!(first, [0.25, -0.5]);
    assert_eq!(rest, [0.25, -0.5, 0.25, -0.5]);
    assert_eq!(vm.sample_index(), 3);
    vm.reset();
    let mut repeated = [0.0; 2];
    vm.render_into(&mut repeated).unwrap();
    assert_eq!(repeated, first);
}

#[test]
fn adr_0014_stereo_renderer_writes_manual_header_and_interleaved_pcm16() {
    let program = stereo_program(3);
    let mut bytes = Vec::new();
    let report = render(&program, &mut bytes).unwrap();
    assert_eq!(report.samples, 3);
    assert_eq!(report.channels, 2);

    let expected_header: [u8; 44] = [
        b'R', b'I', b'F', b'F', 48, 0, 0, 0, b'W', b'A', b'V', b'E', b'f', b'm', b't', b' ', 16, 0,
        0, 0, 1, 0, 2, 0, 128, 187, 0, 0, 0, 238, 2, 0, 4, 0, 16, 0, b'd', b'a', b't', b'a', 12, 0,
        0, 0,
    ];
    assert_eq!(&bytes[..44], &expected_header);
    assert_eq!(&bytes[44..], &[0, 32, 0, 192, 0, 32, 0, 192, 0, 32, 0, 192]);
    assert_eq!(bytes.len(), 56);
}

#[test]
fn adr_0014_reference_pan_matches_explicit_equal_power_vectors() {
    for (position, expected) in [
        (-2.0, [0.5, 0.0]),
        (-1.0, [0.5, 0.0]),
        (0.0, [0.353_553_38, 0.353_553_38]),
        (1.0, [0.0, 0.5]),
        (2.0, [0.0, 0.5]),
    ] {
        let program = pan_program(position);
        let mut reference = ReferenceInterpreter::new(&program.typed);
        let ReferenceFrame::Stereo(actual) = reference.next_frame(0, 48_000).unwrap() else {
            panic!("pan reference must return Stereo")
        };
        assert_close(actual[0], expected[0], 0);
        assert_close(actual[1], expected[1], 0);
    }
}

#[test]
fn adr_0014_production_pan_matches_reference_and_checks_position() {
    for position in [-2.0, -1.0, 0.0, 1.0, 2.0] {
        let program = pan_program(position);
        let actual = vm_values(&program);
        let mut reference = ReferenceInterpreter::new(&program.typed);
        let ReferenceFrame::Stereo(expected) = reference.next_frame(0, 48_000).unwrap() else {
            panic!("pan reference must return Stereo")
        };
        assert_close(actual[0], expected[0], 0);
        assert_close(actual[1], expected[1], 1);
    }

    let bytecode = Bytecode {
        instructions: vec![
            Instruction::ConstScalar {
                dst: ScalarRegister::new(0),
                value: 0.5,
            },
            Instruction::Lift {
                dst: AudioRegister::new(0),
                src: ScalarRegister::new(0),
            },
            Instruction::ConstScalar {
                dst: ScalarRegister::new(1),
                value: 0.0,
            },
            Instruction::Lift {
                dst: AudioRegister::new(1),
                src: ScalarRegister::new(1),
            },
            Instruction::BinaryAudio {
                op: linen::bytecode::BinaryOp::Divide,
                dst: AudioRegister::new(2),
                lhs: Operand::Audio(AudioRegister::new(1)),
                rhs: Operand::Audio(AudioRegister::new(1)),
            },
            Instruction::Pan {
                dst_left: AudioRegister::new(3),
                dst_right: AudioRegister::new(4),
                input: Operand::Audio(AudioRegister::new(0)),
                position: Operand::Audio(AudioRegister::new(2)),
            },
            Instruction::SetStereoOutput {
                src_left: AudioRegister::new(3),
                src_right: AudioRegister::new(4),
            },
        ],
        scalar_register_count: 2,
        audio_register_count: 7,
        state_count: 0,
        output: OutputLayout::Stereo {
            left: AudioRegister::new(5),
            right: AudioRegister::new(6),
        },
        debug_origins: None,
    };
    let error = Vm::new(&bytecode, VmConfig::default())
        .unwrap()
        .render_into(&mut [0.0; 2])
        .unwrap_err();
    assert_eq!(
        error,
        VmRuntimeError::NonFinitePosition {
            instruction: 5,
            sample_index: 0
        }
    );

    let invalid = pan_program(f64::NAN);
    let mut reference = ReferenceInterpreter::new(&invalid.typed);
    assert_eq!(
        reference.next_frame(0, 48_000).unwrap_err().kind,
        ReferenceErrorKind::NonFinitePosition
    );
}

#[test]
fn adr_0014_stereo_verifier_corruption_and_debug_stripping_are_explicit() {
    let (_, mut graph) = stereo_parts(1);
    graph.nodes[4].result_type = ValueType::Sig;
    let mut sources = SourceMap::new();
    sources.add_source("stage-05-test.ln", "");
    assert_eq!(
        verify_ir(&graph, &sources),
        Err(IrVerifyError::ResultTypeMismatch {
            node: NodeId::new(4),
            expected: ValueType::Stereo,
            actual: ValueType::Sig,
        })
    );

    let program = stereo_program(2);
    let mut wrong_layout = program.bytecode.clone();
    wrong_layout.output = OutputLayout::Mono(AudioRegister::new(4));
    assert_eq!(
        verify_bytecode(&wrong_layout),
        Err(BytecodeVerifyError::OutputLayoutMismatch)
    );

    let mut stripped = program.bytecode.clone();
    stripped.strip_debug_info();
    let regular = stereo_program(2);
    assert_eq!(vm_values(&regular), {
        let stripped_program = CompiledProgram {
            bytecode: stripped,
            ..regular.clone()
        };
        vm_values(&stripped_program)
    });
}

#[test]
fn regression_stage_05_stereo_register_aliases_are_atomic_and_validated() {
    let mut crossed = stereo_program(1);
    crossed.bytecode.instructions[4] = Instruction::MakeStereo {
        dst_left: AudioRegister::new(1),
        dst_right: AudioRegister::new(2),
        src_left: AudioRegister::new(0),
        src_right: AudioRegister::new(1),
    };
    crossed.bytecode.instructions[5] = Instruction::SetStereoOutput {
        src_left: AudioRegister::new(1),
        src_right: AudioRegister::new(2),
    };
    verify_bytecode(&crossed.bytecode).unwrap();
    assert_eq!(vm_values(&crossed), [0.25, -0.5]);

    let mut stereo_duplicate = stereo_program(1).bytecode;
    let Instruction::MakeStereo { dst_right, .. } = &mut stereo_duplicate.instructions[4] else {
        panic!("MakeStereo instruction expected")
    };
    *dst_right = AudioRegister::new(2);
    assert_eq!(
        verify_bytecode(&stereo_duplicate),
        Err(BytecodeVerifyError::DuplicateStereoDestination {
            instruction: 4,
            register: AudioRegister::new(2),
        })
    );

    let mut pan_duplicate = pan_program(1.0).bytecode;
    let Instruction::Pan {
        dst_left,
        dst_right,
        ..
    } = &mut pan_duplicate.instructions[4]
    else {
        panic!("Pan instruction expected")
    };
    let duplicate = *dst_left;
    *dst_right = duplicate;
    assert_eq!(
        verify_bytecode(&pan_duplicate),
        Err(BytecodeVerifyError::DuplicateStereoDestination {
            instruction: 4,
            register: duplicate,
        })
    );
}

#[test]
fn adr_0014_stereo_functions_preserve_sharing_fresh_state_and_reference() {
    let source = "let pair = stereo(saw(100), saw(200)); let route = (x: Stereo): Stereo => x; let out = route(pair); render 1ms;";
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
    assert_eq!(program.bytecode.state_count, 2);
    let actual = vm_values(&program);
    let mut reference = ReferenceInterpreter::new(&program.typed);
    for frame in 0..program.typed.render.render_samples as usize {
        let ReferenceFrame::Stereo(expected) = reference.next_frame(frame as u64, 48_000).unwrap()
        else {
            panic!("stereo output expected")
        };
        assert_close(actual[frame * 2], expected[0], frame * 2);
        assert_close(actual[frame * 2 + 1], expected[1], frame * 2 + 1);
    }
}

#[test]
fn spec_7_5_generated_stage_5_stereo_corpus_matches_reference() {
    for seed in 0..8 {
        let source = generator::stage5_stereo_program(seed);
        let program = compile_program(&source, 1 + seed as usize % 7);
        let actual = vm_values(&program);
        let mut reference = ReferenceInterpreter::new(&program.typed);
        for frame in 0..program.typed.render.render_samples as usize {
            let ReferenceFrame::Stereo(expected) =
                reference.next_frame(frame as u64, 48_000).unwrap()
            else {
                panic!("generated output must be Stereo")
            };
            assert_close(actual[frame * 2], expected[0], frame * 2);
            assert_close(actual[frame * 2 + 1], expected[1], frame * 2 + 1);
        }
    }
}
