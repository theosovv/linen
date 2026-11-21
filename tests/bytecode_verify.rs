use linen::bytecode::{
    AudioRegister, BinaryOp, Bytecode, BytecodeVerifyError, Instruction, Operand, ScalarRegister,
    StateId, codegen, verify, verify_origins,
};
use linen::ir::NodeOrigin;
use linen::{RenderConfig, SourceMap, SourceSpan, compile};

fn context() -> (SourceMap, SourceSpan) {
    let mut sources = SourceMap::new();
    let source = sources.add_source("bytecode.ln", "0");
    let span = SourceSpan::new(source, 0, 1).expect("the test span is valid");
    (sources, span)
}

fn origin(span: SourceSpan) -> NodeOrigin {
    NodeOrigin { primary: span }
}

fn compiled_output_origin(program: &linen::CompiledProgram) -> NodeOrigin {
    let span = program
        .typed
        .bindings
        .iter()
        .find(|binding| binding.id == program.typed.output)
        .expect("compiled output must have a binding")
        .value
        .span;

    NodeOrigin { primary: span }
}

fn constant_program(span: SourceSpan) -> Bytecode {
    Bytecode {
        instructions: vec![
            Instruction::ConstScalar {
                dst: ScalarRegister::new(0),
                value: 0.25,
            },
            Instruction::Lift {
                dst: AudioRegister::new(0),
                src: ScalarRegister::new(0),
            },
            Instruction::SetOutput {
                src: AudioRegister::new(0),
            },
        ],
        scalar_register_count: 1,
        audio_register_count: 2,
        state_count: 0,
        output: AudioRegister::new(1),
        debug_origins: Some(vec![origin(span), origin(span), origin(span)]),
    }
}

fn sine_program(span: SourceSpan) -> Bytecode {
    Bytecode {
        instructions: vec![
            Instruction::ConstScalar {
                dst: ScalarRegister::new(0),
                value: 480.0,
            },
            Instruction::Lift {
                dst: AudioRegister::new(0),
                src: ScalarRegister::new(0),
            },
            Instruction::Sine {
                dst: AudioRegister::new(1),
                freq: Operand::Audio(AudioRegister::new(0)),
                state: StateId::new(0),
            },
            Instruction::SetOutput {
                src: AudioRegister::new(1),
            },
        ],
        scalar_register_count: 1,
        audio_register_count: 3,
        state_count: 1,
        output: AudioRegister::new(2),
        debug_origins: Some(vec![origin(span); 4]),
    }
}

#[test]
fn adr_0005_codegen_has_explicit_register_state_output_and_origin_layout() {
    let outcome = compile(
        "let out = sine(480) * 0.1; render 1ms;",
        RenderConfig::default(),
    )
    .expect("the default config is valid");
    let program = outcome.result.as_ref().expect("the source should compile");
    let bytecode = &program.bytecode;

    assert_eq!(
        bytecode.instructions,
        vec![
            Instruction::ConstScalar {
                dst: ScalarRegister::new(0),
                value: 480.0,
            },
            Instruction::Lift {
                dst: AudioRegister::new(0),
                src: ScalarRegister::new(0),
            },
            Instruction::Sine {
                dst: AudioRegister::new(1),
                freq: Operand::Audio(AudioRegister::new(0)),
                state: StateId::new(0),
            },
            Instruction::ConstScalar {
                dst: ScalarRegister::new(1),
                value: 0.1,
            },
            Instruction::Lift {
                dst: AudioRegister::new(2),
                src: ScalarRegister::new(1),
            },
            Instruction::BinaryAudio {
                op: BinaryOp::Multiply,
                dst: AudioRegister::new(3),
                lhs: Operand::Audio(AudioRegister::new(1)),
                rhs: Operand::Audio(AudioRegister::new(2)),
            },
            Instruction::SetOutput {
                src: AudioRegister::new(3),
            },
        ]
    );
    assert_eq!(bytecode.scalar_register_count, 2);
    assert_eq!(bytecode.audio_register_count, 5);
    assert_eq!(bytecode.state_count, 1);
    assert_eq!(bytecode.output, AudioRegister::new(4));
    assert_eq!(bytecode.debug_origins.as_ref().map(Vec::len), Some(7));
    verify(bytecode).expect("codegen must produce structurally valid bytecode");
    verify_origins(bytecode, &outcome.sources).expect("codegen origins must be valid");
    assert_eq!(
        *bytecode,
        codegen(&program.ir, compiled_output_origin(program))
    );
}

#[test]
fn adr_0005_set_output_origin_is_the_full_out_value_for_aliases() {
    let outcome = compile(
        include_str!("../fixtures/regression/stage-03-set-output-origin.ln"),
        RenderConfig::default(),
    )
    .expect("the default config is valid");
    let program = outcome.result.as_ref().expect("the source should compile");
    let output_origin = compiled_output_origin(program);
    assert_eq!(
        (
            output_origin.primary.source().index(),
            output_origin.primary.start(),
            output_origin.primary.end()
        ),
        (0, 32, 36)
    );

    let set_output_origin = program
        .bytecode
        .debug_origins
        .as_ref()
        .and_then(|origins| origins.last())
        .expect("SetOutput must retain an origin");
    assert_eq!(*set_output_origin, output_origin);
    verify_origins(&program.bytecode, &outcome.sources)
        .expect("the full output origin must remain source-aware");
}

#[test]
fn spec_5_7_verifier_accepts_manually_built_valid_bytecode() {
    let (sources, span) = context();
    let bytecode = constant_program(span);
    verify(&bytecode).expect("the manual bytecode is structurally valid");
    verify_origins(&bytecode, &sources).expect("the manual origins are valid");

    let mut stripped = bytecode.clone();
    stripped.strip_debug_info();
    verify(&stripped).expect("stripped bytecode remains structurally valid");
    verify_origins(&stripped, &sources).expect("missing origins are explicitly allowed");
}

#[test]
fn spec_5_7_verifier_rejects_unknown_register_indices() {
    let (_, span) = context();
    let mut scalar = constant_program(span);
    scalar.instructions[1] = Instruction::Lift {
        dst: AudioRegister::new(0),
        src: ScalarRegister::new(1),
    };
    assert_eq!(
        verify(&scalar),
        Err(BytecodeVerifyError::UnknownScalarRegister {
            instruction: 1,
            register: ScalarRegister::new(1),
        })
    );

    let mut audio = constant_program(span);
    audio.instructions[2] = Instruction::SetOutput {
        src: AudioRegister::new(2),
    };
    assert_eq!(
        verify(&audio),
        Err(BytecodeVerifyError::UnknownAudioRegister {
            instruction: 2,
            register: AudioRegister::new(2),
        })
    );
}

#[test]
fn spec_5_7_verifier_rejects_uninitialized_inputs() {
    let (_, span) = context();
    let mut scalar = constant_program(span);
    scalar.instructions.swap(0, 1);
    assert_eq!(
        verify(&scalar),
        Err(BytecodeVerifyError::UninitializedScalarRegister {
            instruction: 0,
            register: ScalarRegister::new(0),
        })
    );

    let audio = Bytecode {
        instructions: vec![
            Instruction::NegAudio {
                dst: AudioRegister::new(0),
                src: AudioRegister::new(0),
            },
            Instruction::SetOutput {
                src: AudioRegister::new(0),
            },
        ],
        scalar_register_count: 0,
        audio_register_count: 2,
        state_count: 0,
        output: AudioRegister::new(1),
        debug_origins: None,
    };
    assert_eq!(
        verify(&audio),
        Err(BytecodeVerifyError::UninitializedAudioRegister {
            instruction: 0,
            register: AudioRegister::new(0),
        })
    );
}

#[test]
fn spec_5_7_verifier_rejects_nonfinite_scalar_constants() {
    let (_, span) = context();
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut bytecode = constant_program(span);
        bytecode.instructions[0] = Instruction::ConstScalar {
            dst: ScalarRegister::new(0),
            value,
        };
        assert_eq!(
            verify(&bytecode),
            Err(BytecodeVerifyError::NonFiniteConstant { instruction: 0 })
        );
    }
}

#[test]
fn spec_5_7_verifier_rejects_unknown_duplicate_and_missing_states() {
    let (_, span) = context();
    let mut unknown = sine_program(span);
    unknown.instructions[2] = Instruction::Sine {
        dst: AudioRegister::new(1),
        freq: Operand::Audio(AudioRegister::new(0)),
        state: StateId::new(1),
    };
    assert_eq!(
        verify(&unknown),
        Err(BytecodeVerifyError::UnknownState {
            instruction: 2,
            state: StateId::new(1),
        })
    );

    let mut duplicate = sine_program(span);
    duplicate.instructions.insert(
        3,
        Instruction::Sine {
            dst: AudioRegister::new(1),
            freq: Operand::Audio(AudioRegister::new(0)),
            state: StateId::new(0),
        },
    );
    assert_eq!(
        verify(&duplicate),
        Err(BytecodeVerifyError::DuplicateState {
            instruction: 3,
            state: StateId::new(0),
        })
    );

    let mut missing = sine_program(span);
    missing.state_count = 2;
    assert_eq!(
        verify(&missing),
        Err(BytecodeVerifyError::MissingState(StateId::new(1)))
    );
}

#[test]
fn spec_5_7_verifier_enforces_the_dedicated_output_protocol() {
    let (_, span) = context();
    let mut missing = constant_program(span);
    missing.instructions.pop();
    assert_eq!(verify(&missing), Err(BytecodeVerifyError::MissingSetOutput));

    let mut early = constant_program(span);
    early.instructions.insert(
        2,
        Instruction::SetOutput {
            src: AudioRegister::new(0),
        },
    );
    assert_eq!(
        verify(&early),
        Err(BytecodeVerifyError::NonFinalSetOutput { instruction: 2 })
    );

    let mut out_of_range = constant_program(span);
    out_of_range.output = AudioRegister::new(2);
    assert_eq!(
        verify(&out_of_range),
        Err(BytecodeVerifyError::OutputOutOfRange(AudioRegister::new(2)))
    );

    let mut written = constant_program(span);
    written.instructions.insert(
        2,
        Instruction::NegAudio {
            dst: AudioRegister::new(1),
            src: AudioRegister::new(0),
        },
    );
    assert_eq!(
        verify(&written),
        Err(BytecodeVerifyError::OutputWritten { instruction: 2 })
    );
}

#[test]
fn spec_5_7_origin_verifier_rejects_count_and_source_corruption() {
    let (sources, span) = context();
    let mut count = constant_program(span);
    count.debug_origins = Some(vec![origin(span)]);
    assert_eq!(
        verify_origins(&count, &sources),
        Err(BytecodeVerifyError::DebugOriginCount {
            instructions: 3,
            origins: 1,
        })
    );

    let mut other = SourceMap::new();
    other.add_source("zero.ln", "0");
    let unknown = other.add_source("one.ln", "1");
    let mut invalid = constant_program(span);
    invalid.debug_origins = Some(vec![origin(SourceSpan::empty(unknown, 0)); 3]);
    assert_eq!(
        verify_origins(&invalid, &sources),
        Err(BytecodeVerifyError::InvalidOrigin { instruction: 0 })
    );

    let mut unicode = SourceMap::new();
    let unicode_id = unicode.add_source("unicode.ln", "ё");
    let non_boundary = SourceSpan::new(unicode_id, 0, 1).expect("offsets are ordered");
    let mut non_utf8 = constant_program(non_boundary);
    non_utf8.debug_origins = Some(vec![origin(non_boundary); 3]);
    assert_eq!(
        verify_origins(&non_utf8, &unicode),
        Err(BytecodeVerifyError::InvalidOrigin { instruction: 0 })
    );
}

#[test]
fn adr_0005_stripping_debug_info_changes_only_origins() {
    let (_, span) = context();
    let original = sine_program(span);
    let mut stripped = original.clone();
    stripped.strip_debug_info();

    assert_eq!(stripped.instructions, original.instructions);
    assert_eq!(
        stripped.scalar_register_count,
        original.scalar_register_count
    );
    assert_eq!(stripped.audio_register_count, original.audio_register_count);
    assert_eq!(stripped.state_count, original.state_count);
    assert_eq!(stripped.output, original.output);
    assert_eq!(stripped.debug_origins, None);
}
