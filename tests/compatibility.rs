use linen::ir::verify;
use linen::{CompileError, ErrorCode, RenderConfig, RenderError, compile, render};

#[test]
fn spec_9_stage_3_preserves_semantically_valid_previous_fixtures() {
    let fixtures = [
        include_str!("../fixtures/valid/stage-01-beep.ln"),
        include_str!("../fixtures/valid/stage-01-expression-node-limit.ln"),
        include_str!("../fixtures/valid/stage-01a-keyword-prefixes.ln"),
        include_str!("../fixtures/valid/stage-02-mixed-types.ln"),
        include_str!("../fixtures/regression/stage-02-repeated-session.ln"),
        include_str!("../fixtures/regression/stage-02-atomic-output.ln"),
    ];

    for source in fixtures {
        let outcome =
            compile(source, RenderConfig::default()).expect("the default config is valid");
        let program = outcome
            .result
            .as_ref()
            .expect("previous valid fixture must compile");
        verify(&program.ir, &outcome.sources).expect("previous fixture IR must verify");
        assert!(program.ir.roots.contains(&program.ir.output));
        assert_eq!(
            program.ir.render_samples,
            program.typed.render.render_samples
        );
    }
}

#[test]
fn spec_9_stage_3_preserves_deferred_semantics_diagnostic() {
    let outcome = compile(
        include_str!("../fixtures/valid/stage-01-semantics-deferred.ln"),
        RenderConfig::default(),
    )
    .expect("the default config is valid");
    let error = outcome
        .result
        .as_ref()
        .expect_err("the old deferred-semantics fixture is invalid since stage 2");
    let CompileError::Diagnostic(diagnostic) = error else {
        panic!("source rejection must remain a diagnostic");
    };

    assert_eq!(diagnostic.code(), ErrorCode::UnknownName);
    assert_eq!(diagnostic.sample_index(), None);
    let span = diagnostic.primary().span();
    assert_eq!(
        (span.source().index(), span.start(), span.end()),
        (0, 12, 17)
    );
}

#[test]
fn adr_0004_unused_bindings_keep_runtime_error_span_and_order() {
    let outcome = compile(
        include_str!("../fixtures/invalid/stage-03-unused-runtime-error.ln"),
        RenderConfig::default(),
    )
    .expect("the default config is valid");
    let program = outcome
        .result
        .as_ref()
        .expect("runtime-error source must compile");
    assert_eq!(program.ir.roots.len(), 3);
    assert_eq!(program.ir.roots[2], program.ir.output);

    let error = render(program, &mut Vec::new()).expect_err("unused binding must still fail");
    let RenderError::Diagnostic(diagnostic) = error else {
        panic!("runtime failure must remain a source diagnostic");
    };
    assert_eq!(diagnostic.code(), ErrorCode::NonFiniteOutput);
    assert_eq!(diagnostic.sample_index(), Some(0));
    assert_eq!(
        diagnostic.message(),
        "sine frequency contains NaN or infinity"
    );
    let span = diagnostic.primary().span();
    assert_eq!(
        (span.source().index(), span.start(), span.end()),
        (0, 38, 49)
    );
}

fn runtime_diagnostic_at_block_size(source: &str, block_size: usize) -> linen::Diagnostic {
    let outcome = compile(
        source,
        RenderConfig {
            block_size,
            ..RenderConfig::default()
        },
    )
    .expect("the render config is valid");
    let program = outcome
        .result
        .as_ref()
        .expect("the runtime-error source must compile");

    match render(program, &mut Vec::new()).expect_err("the source must fail during rendering") {
        RenderError::Diagnostic(diagnostic) => diagnostic,
        other => panic!("runtime failure must remain a source diagnostic: {other:?}"),
    }
}

#[test]
fn regression_stage_03_frequency_error_order_is_independent_of_block_size() {
    let source = include_str!("../fixtures/regression/stage-03-runtime-frequency-order.ln");

    for block_size in [1, 3, 64, 128, 256] {
        let diagnostic = runtime_diagnostic_at_block_size(source, block_size);
        assert_eq!(diagnostic.code(), ErrorCode::NonFiniteOutput);
        assert_eq!(
            diagnostic.sample_index(),
            Some(0),
            "block size {block_size}"
        );
        assert_eq!(
            diagnostic.message(),
            "sine frequency contains NaN or infinity"
        );
        let span = diagnostic.primary().span();
        assert_eq!(
            (span.source().index(), span.start(), span.end()),
            (0, 55, 66),
            "block size {block_size}"
        );
    }
}

#[test]
fn regression_stage_03_output_error_precedes_later_frequency_errors() {
    let source = include_str!("../fixtures/regression/stage-03-runtime-output-order.ln");

    for block_size in [1, 3, 64, 128, 256] {
        let diagnostic = runtime_diagnostic_at_block_size(source, block_size);
        assert_eq!(diagnostic.code(), ErrorCode::NonFiniteOutput);
        assert_eq!(
            diagnostic.sample_index(),
            Some(0),
            "block size {block_size}"
        );
        assert_eq!(
            diagnostic.message(),
            "output signal contains NaN or infinity"
        );
        let span = diagnostic.primary().span();
        assert_eq!(
            (span.source().index(), span.start(), span.end()),
            (0, 10, 21),
            "block size {block_size}"
        );
    }
}

#[test]
fn spec_9_beep_wav_contract_is_unchanged_on_the_new_execution_path() {
    let program = compile(
        include_str!("../fixtures/valid/stage-01-beep.ln"),
        RenderConfig::default(),
    )
    .expect("the default config is valid")
    .result
    .expect("Beep must compile");
    let mut wav = Vec::new();
    let report = render(&program, &mut wav).expect("Beep must render through the VM");

    assert_eq!(report.samples, 96_000);
    assert_eq!(report.sample_rate, 48_000);
    assert_eq!(report.duration_seconds, 2.0);
    assert_eq!(wav.len(), 192_044);
    assert_eq!(u32::from_le_bytes(wav[4..8].try_into().unwrap()), 192_036);
    assert_eq!(u32::from_le_bytes(wav[40..44].try_into().unwrap()), 192_000);
}

#[test]
fn spec_9_repeated_compilation_keeps_structurally_deterministic_ir_and_bytecode() {
    let source = include_str!("../fixtures/valid/stage-03-block-partitions.ln");
    let first = compile(source, RenderConfig::default())
        .expect("the default config is valid")
        .result
        .expect("the fixture must compile");
    let second = compile(source, RenderConfig::default())
        .expect("the default config is valid")
        .result
        .expect("the repeated fixture must compile");

    assert_eq!(first, second);
}

#[test]
fn adr_0005_stripped_runtime_error_returns_explicit_missing_origin() {
    let mut program = compile(
        include_str!("../fixtures/invalid/stage-03-unused-runtime-error.ln"),
        RenderConfig::default(),
    )
    .expect("the default config is valid")
    .result
    .expect("runtime-error source must compile");
    program.bytecode.strip_debug_info();

    let error = render(&program, &mut Vec::new())
        .expect_err("stripped runtime error cannot invent a source origin");
    assert!(matches!(
        error,
        RenderError::MissingDebugOrigin { instruction: 4 }
    ));
}

#[test]
fn spec_5_8_renderer_returns_vm_creation_errors_before_writing() {
    let mut program = compile("let out = 1; render 1ms;", RenderConfig::default())
        .expect("the default config is valid")
        .result
        .expect("the source must compile");
    program.bytecode.instructions[1] = linen::bytecode::Instruction::Lift {
        dst: linen::bytecode::AudioRegister::new(0),
        src: linen::bytecode::ScalarRegister::new(9),
    };
    let mut output = Vec::new();

    let error = render(&program, &mut output).expect_err("invalid bytecode must not render");
    assert!(matches!(
        error,
        RenderError::VmCreate(linen::vm::VmCreateError::InvalidBytecode(
            linen::bytecode::BytecodeVerifyError::UnknownScalarRegister {
                instruction: 1,
                register,
            }
        )) if register == linen::bytecode::ScalarRegister::new(9)
    ));
    assert!(output.is_empty());
}

#[test]
fn spec_9_stage_5d_polyblep_fixture_preserves_verified_compiler_contracts() {
    let outcome = compile(
        include_str!("../fixtures/valid/stage-05d-polyblep.ln"),
        RenderConfig::default(),
    )
    .expect("the default config is valid");
    let program = outcome
        .result
        .as_ref()
        .expect("the stage 5D fixture must compile");
    verify(&program.ir, &outcome.sources).expect("stage 5D IR must verify");
    linen::bytecode::verify(&program.bytecode).expect("stage 5D bytecode must verify");
    linen::bytecode::verify_origins(&program.bytecode, &outcome.sources)
        .expect("stage 5D origins must verify");
    assert_eq!(program.bytecode.state_count, 4);
}
