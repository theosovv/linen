use linen::render::{RenderConfigError, RenderError};
use linen::sema::{
    BUILTINS, BindingId, BuiltinId, Determinism, Purity, Statefulness, SymbolStorage, Type,
    TypedExpr, TypedExprKind, builtin_by_name, builtin_descriptor,
};
use linen::{CompileOutcome, ErrorCode, RenderConfig, SourceMap, SourceSpan, compile, render};

fn outcome(source: &str) -> CompileOutcome {
    compile(source, RenderConfig::default()).expect("the default render config is valid")
}

fn program(source: &str) -> linen::CompiledProgram {
    outcome(source).result.expect("the source should compile")
}

fn assert_error(source: &str, code: ErrorCode, start: usize, end: usize) -> CompileOutcome {
    let outcome = outcome(source);
    let diagnostic = outcome
        .result
        .as_ref()
        .expect_err("the source should be rejected")
        .as_diagnostic()
        .expect("a rejected source must have a source diagnostic");
    let span = diagnostic.primary().span();

    assert_eq!(diagnostic.code(), code);
    assert_eq!(diagnostic.sample_index(), None);
    assert_eq!(
        (span.source().index(), span.start(), span.end()),
        (0, start, end)
    );
    outcome
}

fn assert_expression_spans(expression: &TypedExpr, sources: &SourceMap) {
    assert_valid_span(expression.span, sources);
    match &expression.kind {
        TypedExprKind::Number(_) => {}
        TypedExprKind::Identifier { identifier, .. } => {
            assert_valid_span(identifier.span, sources);
        }
        TypedExprKind::BuiltinCall {
            callee, arguments, ..
        }
        | TypedExprKind::FunctionCall {
            callee, arguments, ..
        } => {
            assert_valid_span(callee.span, sources);
            for argument in arguments {
                assert_expression_spans(argument, sources);
            }
        }
        TypedExprKind::Unary { operand, .. } => assert_expression_spans(operand, sources),
        TypedExprKind::Binary { left, right, .. } => {
            assert_expression_spans(left, sources);
            assert_expression_spans(right, sources);
        }
        TypedExprKind::Group(inner) | TypedExprKind::LiftFloatToSig(inner) => {
            assert_expression_spans(inner, sources);
        }
        TypedExprKind::Block { bindings, result } => {
            for binding in bindings {
                assert_expression_spans(&binding.value, sources);
            }
            assert_expression_spans(result, sources);
        }
    }
}

fn assert_valid_span(span: SourceSpan, sources: &SourceMap) {
    let source = sources
        .get(span.source())
        .expect("every typed span must resolve in the outcome source map");
    assert!(span.start() <= span.end());
    assert!(span.end() <= source.text().len());
    assert!(source.text().is_char_boundary(span.start()));
    assert!(source.text().is_char_boundary(span.end()));
}

#[test]
fn spec_5_5_sine_has_a_complete_static_builtin_descriptor() {
    assert_eq!(BUILTINS.len(), 10);
    let descriptor = builtin_descriptor(BuiltinId::SINE).expect("sine must have a descriptor");

    assert_eq!(descriptor.id, BuiltinId::SINE);
    assert_eq!(descriptor.name, "sine");
    assert_eq!(descriptor.signature.parameters, [Type::Sig]);
    assert_eq!(descriptor.signature.result, Type::Sig);
    assert_eq!(descriptor.purity, Purity::NoSideEffects);
    assert_eq!(descriptor.determinism, Determinism::Deterministic);
    assert_eq!(descriptor.statefulness, Statefulness::Stateful);
    assert_eq!(descriptor.documentation_name, "sine");
    assert!(std::ptr::eq(descriptor, builtin_by_name("sine").unwrap()));
    assert!(builtin_by_name("noize").is_none());
}

#[test]
fn spec_6_4_render_config_validates_sample_rate_and_block_size() {
    let valid = RenderConfig::default();
    assert_eq!(valid.sample_rate, 48_000);
    assert_eq!(valid.block_size, 128);
    assert_eq!(valid.max_output_seconds, 600);
    assert_eq!(valid.validate(), Ok(()));

    for block_size in [1, 3, 64, 128, 256, usize::MAX] {
        assert_eq!(
            RenderConfig {
                block_size,
                ..valid
            }
            .validate(),
            Ok(())
        );
    }
    for sample_rate in [8_000, 384_000] {
        assert_eq!(
            RenderConfig {
                sample_rate,
                ..valid
            }
            .validate(),
            Ok(())
        );
    }
    assert_eq!(
        RenderConfig {
            sample_rate: 7_999,
            ..valid
        }
        .validate(),
        Err(RenderConfigError::InvalidSampleRate(7_999))
    );
    assert_eq!(
        RenderConfig {
            sample_rate: 384_001,
            ..valid
        }
        .validate(),
        Err(RenderConfigError::InvalidSampleRate(384_001))
    );
    assert_eq!(
        RenderConfig {
            block_size: 0,
            ..valid
        }
        .validate(),
        Err(RenderConfigError::ZeroBlockSize)
    );
}

#[test]
fn spec_5_5_typed_ast_preserves_ids_types_and_explicit_lifts() {
    let outcome =
        outcome("let value = 1;\nlet wave = sine(value);\nlet out = wave + value;\nrender 1s;\n");
    let program = outcome
        .result
        .as_ref()
        .expect("the typed source should compile");
    let bindings = &program.typed.bindings();

    assert_eq!(bindings.len(), 3);
    assert_eq!(bindings[0].id, BindingId::new(0));
    assert_eq!(bindings[1].id, BindingId::new(1));
    assert_eq!(bindings[2].id, BindingId::new(2));
    assert_eq!(program.typed.output, BindingId::new(2));
    assert_eq!(bindings[0].value.ty, Type::Float);
    assert_eq!(bindings[1].value.ty, Type::Sig);
    assert_eq!(bindings[2].value.ty, Type::Sig);
    assert_eq!(outcome.symbols.len(), 3);
    assert!(!outcome.symbols.is_empty());

    let TypedExprKind::BuiltinCall {
        builtin, arguments, ..
    } = &bindings[1].value.kind
    else {
        panic!("wave must be a resolved sine call");
    };
    assert_eq!(*builtin, BuiltinId::SINE);
    let TypedExprKind::LiftFloatToSig(lifted) = &arguments[0].kind else {
        panic!("the Float sine argument must be lifted explicitly");
    };
    let TypedExprKind::Identifier { id, .. } = lifted.kind else {
        panic!("the lifted argument must reference the first binding");
    };
    assert_eq!(id, BindingId::new(0));

    let TypedExprKind::Binary { right, .. } = &bindings[2].value.kind else {
        panic!("the output must be a binary expression");
    };
    let TypedExprKind::LiftFloatToSig(lifted) = &right.kind else {
        panic!("the mixed arithmetic operand must be lifted");
    };
    let TypedExprKind::Identifier { id, .. } = lifted.kind else {
        panic!("the mixed arithmetic operand must preserve its resolved ID");
    };
    assert_eq!(id, BindingId::new(0));

    for (index, name) in ["value", "wave", "out"].into_iter().enumerate() {
        let symbol = outcome
            .symbols
            .get(BindingId::new(index))
            .expect("every binding must have a stored symbol");
        assert_eq!(symbol.name, name);
        assert_eq!(symbol.id, BindingId::new(index));
    }
}

#[test]
fn spec_4_4_mixed_arithmetic_and_out_annotations_insert_only_required_lifts() {
    let mixed = program(
        "let signal = sine(1);\nlet left = 1 + signal;\nlet right = signal + 1;\nlet out = left * right;\nrender 1s;\n",
    );
    let TypedExprKind::Binary { left, .. } = &mixed.typed.bindings()[1].value.kind else {
        panic!("left binding must be binary");
    };
    assert!(matches!(left.kind, TypedExprKind::LiftFloatToSig(_)));
    let TypedExprKind::Binary { right, .. } = &mixed.typed.bindings()[2].value.kind else {
        panic!("right binding must be binary");
    };
    assert!(matches!(right.kind, TypedExprKind::LiftFloatToSig(_)));
    assert_eq!(mixed.typed.bindings()[3].value.ty, Type::Sig);

    let output = program("let out: Sig = 1; render 1ms;");
    assert!(matches!(
        output.typed.bindings()[0].value.kind,
        TypedExprKind::LiftFloatToSig(_)
    ));
}

#[test]
fn spec_5_5_every_typed_ast_node_keeps_a_valid_source_span() {
    let outcome = outcome(include_str!("../fixtures/valid/stage-02-mixed-types.ln"));
    let program = outcome
        .result
        .as_ref()
        .expect("the mixed-type fixture should compile");

    assert_valid_span(program.typed.span, &outcome.sources);
    assert_valid_span(program.typed.render.span, &outcome.sources);
    assert_valid_span(program.typed.render.duration.span, &outcome.sources);
    for binding in &program.typed.bindings() {
        assert_valid_span(binding.span, &outcome.sources);
        assert_valid_span(binding.name.span, &outcome.sources);
        if let Some(annotation) = &binding.annotation {
            assert_valid_span(annotation.span, &outcome.sources);
        }
        assert_expression_spans(&binding.value, &outcome.sources);
    }
}

#[test]
fn spec_6_2_unknown_forward_and_self_references_use_identifier_spans() {
    assert_error(
        include_str!("../fixtures/invalid/stage-02-unknown-name.ln"),
        ErrorCode::UnknownName,
        10,
        17,
    );
    assert_error(
        "let first = later;\nlet later = 1;\nlet out = first;\nrender 1s;\n",
        ErrorCode::UnknownName,
        12,
        17,
    );
    assert_error("let out = out; render 1s;", ErrorCode::UnknownName, 10, 13);
}

#[test]
fn spec_6_2_duplicate_binding_precedes_value_errors_and_has_secondary_span() {
    let outcome = assert_error(
        include_str!("../fixtures/invalid/stage-02-duplicate-name.ln"),
        ErrorCode::DuplicateBinding,
        19,
        24,
    );
    let diagnostic = outcome
        .result
        .as_ref()
        .unwrap_err()
        .as_diagnostic()
        .expect("source error must be diagnostic");
    assert_eq!(diagnostic.secondary().len(), 1);
    assert_eq!(diagnostic.secondary()[0].span().start(), 4);
    assert_eq!(diagnostic.secondary()[0].span().end(), 9);

    assert_error(
        "let value = 1;\nlet value = missing;\nlet out = 0;\nrender 1s;\n",
        ErrorCode::DuplicateBinding,
        19,
        24,
    );
}

#[test]
fn spec_6_2_call_resolution_precedes_arity_and_argument_checking() {
    assert_error(
        include_str!("../fixtures/invalid/stage-02-unknown-function.ln"),
        ErrorCode::UnknownFunction,
        10,
        15,
    );
    assert_error(
        "let out = noize(missing); render 1s;",
        ErrorCode::UnknownFunction,
        10,
        15,
    );
    for source in [
        "let out = sine(); render 1s;",
        include_str!("../fixtures/invalid/stage-02-invalid-arity.ln"),
        "let out = sine(missing, 1); render 1s;",
    ] {
        assert_error(source, ErrorCode::InvalidArity, 10, 14);
    }
}

#[test]
fn spec_6_2_annotation_mismatches_have_deterministic_primary_and_secondary_spans() {
    let outcome = assert_error(
        include_str!("../fixtures/invalid/stage-02-type-mismatch.ln"),
        ErrorCode::TypeMismatch,
        19,
        26,
    );
    let diagnostic = outcome
        .result
        .as_ref()
        .unwrap_err()
        .as_diagnostic()
        .expect("source error must be diagnostic");
    assert_eq!(diagnostic.secondary().len(), 1);
    assert_eq!(diagnostic.secondary()[0].span().start(), 11);
    assert_eq!(diagnostic.secondary()[0].span().end(), 16);

    assert_error(
        "let out: Float = sine(1); render 1s;",
        ErrorCode::TypeMismatch,
        9,
        14,
    );
}

#[test]
fn spec_6_2_semantic_messages_are_specific_and_do_not_repeat_the_primary_label() {
    let assert_messages = |outcome: &CompileOutcome, message: &str, primary: &str| {
        let diagnostic = outcome
            .result
            .as_ref()
            .expect_err("expected a diagnostic")
            .as_diagnostic()
            .expect("source error must be diagnostic");
        assert_eq!(diagnostic.message(), message);
        assert_eq!(diagnostic.primary().message(), primary);
        assert_ne!(diagnostic.message(), diagnostic.primary().message());
    };

    let duplicate = assert_error(
        include_str!("../fixtures/invalid/stage-02-duplicate-name.ln"),
        ErrorCode::DuplicateBinding,
        19,
        24,
    );
    assert_messages(
        &duplicate,
        "duplicate binding `value`",
        "this name was already declared in the same scope",
    );
    assert_eq!(
        duplicate
            .result
            .as_ref()
            .unwrap_err()
            .as_diagnostic()
            .expect("source error must be diagnostic")
            .secondary()[0]
            .message(),
        "first definition of `value` is here"
    );

    let unknown = assert_error(
        include_str!("../fixtures/invalid/stage-02-unknown-name.ln"),
        ErrorCode::UnknownName,
        10,
        17,
    );
    assert_messages(
        &unknown,
        "unknown name `missing`",
        "no previous binding defines `missing`",
    );
    assert_eq!(
        unknown
            .result
            .as_ref()
            .unwrap_err()
            .as_diagnostic()
            .expect("source error must be diagnostic")
            .notes(),
        ["declare bindings before use; self and forward references are not allowed"]
    );

    let unknown_function = assert_error(
        include_str!("../fixtures/invalid/stage-02-unknown-function.ln"),
        ErrorCode::UnknownFunction,
        10,
        15,
    );
    assert_messages(
        &unknown_function,
        "unknown function `noize`",
        "this callee is not a builtin function",
    );

    let arity = assert_error(
        include_str!("../fixtures/invalid/stage-02-invalid-arity.ln"),
        ErrorCode::InvalidArity,
        10,
        14,
    );
    assert_messages(
        &arity,
        "invalid argument count in call to `sine`",
        "expected 1 argument, found 2 arguments",
    );

    let zero_arity = assert_error(
        "let out = sine(); render 1s;",
        ErrorCode::InvalidArity,
        10,
        14,
    );
    assert_messages(
        &zero_arity,
        "invalid argument count in call to `sine`",
        "expected 1 argument, found 0 arguments",
    );

    let mismatch = assert_error(
        include_str!("../fixtures/invalid/stage-02-type-mismatch.ln"),
        ErrorCode::TypeMismatch,
        19,
        26,
    );
    assert_messages(
        &mismatch,
        "type mismatch: expected `Float`, found `Sig`",
        "this value has type `Sig`",
    );
    assert_eq!(
        mismatch
            .result
            .as_ref()
            .unwrap_err()
            .as_diagnostic()
            .expect("source error must be diagnostic")
            .secondary()[0]
            .message(),
        "this annotation requires type `Float`"
    );

    let output_mismatch = assert_error(
        "let out: Float = sine(1); render 1s;",
        ErrorCode::TypeMismatch,
        9,
        14,
    );
    assert_messages(
        &output_mismatch,
        "`out` must have type `Sig` or `Stereo`, not `Float`",
        "remove this annotation or change it to `Sig` or `Stereo`",
    );
}

#[test]
fn spec_4_4_constant_zero_division_and_nonfinite_float_are_distinct_errors() {
    assert_error(
        include_str!("../fixtures/invalid/stage-02-constant-division-zero.ln"),
        ErrorCode::ConstantDivisionByZero,
        14,
        15,
    );
    assert_error(
        include_str!("../fixtures/invalid/stage-02-nonfinite-float.ln"),
        ErrorCode::InvalidFloat,
        10,
        15,
    );
    assert_error(
        "let out = 1e308 * 1e308; render 1s;",
        ErrorCode::InvalidFloat,
        10,
        23,
    );
    assert_error(
        "let out = 1e100; render 1s;",
        ErrorCode::InvalidFloat,
        10,
        15,
    );
    assert_error(
        "let unused = 1e999;\nlet out = 0;\nrender 1s;\n",
        ErrorCode::InvalidFloat,
        13,
        18,
    );
}

#[test]
fn regression_stage_04_known_float_errors_precede_later_semantic_errors() {
    assert_error(
        "let out: Float = 1 / 0; render 1s;",
        ErrorCode::ConstantDivisionByZero,
        21,
        22,
    );
    assert_error(
        "let out = (1 / 0) + unknown; render 1s;",
        ErrorCode::ConstantDivisionByZero,
        15,
        16,
    );
    assert_error(
        "let out = (1e308 * 1e308) + unknown; render 1s;",
        ErrorCode::InvalidFloat,
        11,
        24,
    );
}

#[test]
fn spec_6_2_numeric_duration_and_limit_messages_describe_the_failed_contract() {
    let assert_messages = |outcome: &CompileOutcome, message: &str, primary: &str| {
        let diagnostic = outcome
            .result
            .as_ref()
            .expect_err("expected a diagnostic")
            .as_diagnostic()
            .expect("source error must be diagnostic");
        assert_eq!(diagnostic.message(), message);
        assert_eq!(diagnostic.primary().message(), primary);
        assert_ne!(diagnostic.message(), diagnostic.primary().message());
    };

    let division = assert_error(
        include_str!("../fixtures/invalid/stage-02-constant-division-zero.ln"),
        ErrorCode::ConstantDivisionByZero,
        14,
        15,
    );
    assert_messages(
        &division,
        "compile-time division by zero",
        "this divisor evaluates to zero",
    );

    let nonfinite = assert_error(
        include_str!("../fixtures/invalid/stage-02-nonfinite-float.ln"),
        ErrorCode::InvalidFloat,
        10,
        15,
    );
    assert_messages(
        &nonfinite,
        "compile-time `Float` must be finite",
        "this expression evaluates to NaN or infinity",
    );

    let lifted = assert_error(
        "let out = 1e100; render 1s;",
        ErrorCode::InvalidFloat,
        10,
        15,
    );
    assert_messages(
        &lifted,
        "`Float` value is too large for `Sig`",
        "converting this expression to `f32` produces infinity",
    );

    let duration = assert_error(
        "let out = 0; render 0s;",
        ErrorCode::InvalidDuration,
        20,
        22,
    );
    assert_messages(
        &duration,
        "invalid render duration",
        "expected a finite, positive duration that rounds to at least one sample at 48000 Hz",
    );

    let overflow = assert_error(
        include_str!("../fixtures/invalid/stage-02-duration-overflow.ln"),
        ErrorCode::RenderOverflow,
        20,
        26,
    );
    assert_messages(
        &overflow,
        "render duration exceeds WAV size limits",
        "sample count or RIFF/PCM size overflows its supported integer range",
    );

    let limited = program("let out = 0; render 601s;");
    let error = render(&limited, &mut Vec::new()).expect_err("the render limit must fail");
    let RenderError::Diagnostic(diagnostic) = error else {
        panic!("a protective render limit must be a source diagnostic");
    };
    assert_eq!(
        diagnostic.message(),
        "render exceeds the configured duration limit"
    );
    assert_eq!(
        diagnostic.primary().message(),
        "requested 601 seconds, but max_output_seconds is 600"
    );
}

#[test]
fn adr_0003_finite_f64_intermediate_and_f32_underflow_are_accepted() {
    program("let value = 1e300 / 1e300;\nlet out = value;\nrender 1s;\n");
    program("let unused = 1e100;\nlet out = 0;\nrender 1s;\n");
    program("let out = 1e-300; render 1s;");
}

#[test]
fn spec_4_4_duration_samples_use_checked_rounding() {
    let seconds = program("let out = 0; render 2s;");
    assert_eq!(seconds.typed.render.render_samples, 96_000);
    assert_eq!(seconds.typed.render.duration_seconds, 2.0);

    let milliseconds = program("let out = 0; render 1ms;");
    assert_eq!(milliseconds.typed.render.render_samples, 48);
    assert_eq!(milliseconds.typed.render.duration_seconds, 0.001);

    let rounded = program("let out = 0; render 0.00002s;");
    assert_eq!(rounded.typed.render.render_samples, 1);

    let config = RenderConfig {
        sample_rate: 10_000,
        ..RenderConfig::default()
    };
    let half = compile("let out = 0; render 0.00005s;", config)
        .expect("the custom sample rate is valid")
        .result
        .expect("half a sample must round away from zero");
    assert_eq!(half.typed.render.render_samples, 1);

    let below_half = compile("let out = 0; render 0.000049s;", config)
        .expect("the custom sample rate is valid")
        .result
        .expect_err("a positive duration below half a sample must be rejected");
    assert_eq!(
        below_half
            .as_diagnostic()
            .expect("invalid duration must be a source diagnostic")
            .code(),
        ErrorCode::InvalidDuration
    );
}

#[test]
fn spec_6_2_invalid_durations_have_literal_spans() {
    for (source, start, end) in [
        ("let out = 0; render 0s;", 20, 22),
        ("let out = 0; render 0ms;", 20, 23),
        (
            include_str!("../fixtures/invalid/stage-02-invalid-duration.ln"),
            21,
            23,
        ),
        ("let out = 0; render 1e999s;", 20, 26),
        ("let out = 0; render 0.00001s;", 20, 28),
    ] {
        assert_error(source, ErrorCode::InvalidDuration, start, end);
    }
}

#[test]
fn spec_6_2_duration_and_sample_size_overflow_are_e4002() {
    assert_error(
        include_str!("../fixtures/invalid/stage-02-duration-overflow.ln"),
        ErrorCode::RenderOverflow,
        20,
        26,
    );
}

#[test]
fn spec_6_4_protective_limit_is_e4003_only_when_rendering() {
    let program = program("let out = 0; render 601s;");
    let error = render(&program, &mut Vec::new()).expect_err("the default limit must reject 601s");
    let RenderError::Diagnostic(diagnostic) = error else {
        panic!("a protective render limit must be a source diagnostic");
    };
    let span = diagnostic.primary().span();

    assert_eq!(diagnostic.code(), ErrorCode::RenderLimitExceeded);
    assert_eq!(diagnostic.sample_index(), None);
    assert_eq!(
        (span.source().index(), span.start(), span.end()),
        (0, 20, 24)
    );

    let zero_limit = compile(
        "let out = 0; render 1ms;",
        RenderConfig {
            max_output_seconds: 0,
            ..RenderConfig::default()
        },
    )
    .expect("a zero protective limit is valid configuration")
    .result
    .expect("the source itself is valid");
    let error = render(&zero_limit, &mut Vec::new())
        .expect_err("a zero limit rejects every positive duration before writing");
    let RenderError::Diagnostic(diagnostic) = error else {
        panic!("a zero protective limit must be a source diagnostic");
    };
    assert_eq!(diagnostic.code(), ErrorCode::RenderLimitExceeded);
}

#[test]
fn spec_5_11_repeated_compilation_is_structurally_deterministic() {
    let source = include_str!("../fixtures/valid/stage-02-mixed-types.ln");
    let first = outcome(source);
    let second = outcome(source);

    assert_eq!(first.symbols, second.symbols);
    assert_eq!(first.result, second.result);
}

#[test]
fn regression_stage_02_repeated_session_compilation_restarts_binding_ids() {
    let mut session = linen::CompileSession::new(linen::CompileConfig::default());
    let source = session.sources_mut().add_source(
        "repeated-session.ln",
        include_str!("../fixtures/regression/stage-02-repeated-session.ln"),
    );
    let first = session
        .compile(source, RenderConfig::default())
        .expect("the source belongs to the session")
        .expect("the first compilation should succeed");
    let first_symbols = session.symbols().clone();
    let second = session
        .compile(source, RenderConfig::default())
        .expect("the source belongs to the session")
        .expect("the second compilation should succeed");

    assert_eq!(first, second);
    assert_eq!(session.symbols(), &first_symbols);
    assert_eq!(session.symbols().len(), 2);
    for (index, binding) in second.typed.bindings().iter().enumerate() {
        assert_eq!(binding.id, BindingId::new(index));
    }
}

#[test]
fn spec_5_8_symbol_storage_resolves_only_known_binding_indices() {
    let storage = SymbolStorage::new();
    assert!(storage.is_empty());
    assert_eq!(storage.len(), 0);
    assert!(storage.get(BindingId::new(0)).is_none());
}
