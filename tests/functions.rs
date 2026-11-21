#[path = "../test-support/reference/mod.rs"]
mod reference;

use linen::ir::{CallOrigin, IrVerifyError, NodeOp, NodeOrigin, verify as verify_ir};
use linen::sema::{FunctionId, FunctionLimits, TypedExprKind};
use linen::syntax::{Expr, ExprKind, TokenKind, TypeName, lex, parse};
use linen::vm::VmRuntimeError;
use linen::{
    CompileConfig, CompileError, CompiledProgram, Diagnostic, ErrorCode, RenderConfig, SourceId,
    SourceMap, SourceSpan, check, check_with_config, compile, compile_with_config, create_vm,
};
use reference::{ReferenceErrorKind, ReferenceInterpreter};

fn test_source_id() -> SourceId {
    let mut sources = SourceMap::new();
    sources.add_source("stage-04.ln", "")
}

fn compile_program(source: &str) -> CompiledProgram {
    compile(source, RenderConfig::default())
        .expect("the default render config is valid")
        .result
        .expect("the stage 4 source should compile")
}

fn compile_with_block_size(source: &str, block_size: usize) -> CompiledProgram {
    compile(
        source,
        RenderConfig {
            block_size,
            ..RenderConfig::default()
        },
    )
    .expect("the render config is valid")
    .result
    .expect("the stage 4 source should compile")
}

fn expression(source: &str) -> Expr {
    let wrapped = format!("let out = {source}; render 1s;");
    let program = parse(&wrapped, test_source_id()).expect("the expression should parse");
    program.bindings()[0].value.clone()
}

fn diagnostic(source: &str, code: ErrorCode) -> Diagnostic {
    let outcome = check(source);
    let diagnostic = outcome.result.expect_err("the source should be rejected");
    assert_eq!(diagnostic.code(), code);
    assert_eq!(diagnostic.primary().span().source().index(), 0);
    diagnostic
}

fn assert_primary(source: &str, diagnostic: &Diagnostic, needle: &str) {
    let start = source
        .find(needle)
        .expect("the expected primary text exists");
    assert_eq!(
        (
            diagnostic.primary().span().start(),
            diagnostic.primary().span().end()
        ),
        (start, start + needle.len())
    );
}

fn assert_primary_from(_source: &str, diagnostic: &Diagnostic, start: usize, len: usize) {
    assert_eq!(
        (
            diagnostic.primary().span().start(),
            diagnostic.primary().span().end()
        ),
        (start, start + len)
    );
}

fn vm_samples(program: &CompiledProgram) -> Vec<f32> {
    let mut vm = create_vm(program).expect("verified bytecode must create a VM");
    let sample_count = program.typed.render.render_samples as usize;
    let mut samples = Vec::with_capacity(sample_count);
    let mut scratch = vec![0.0; program.config.block_size];

    while samples.len() < sample_count {
        let length = scratch.len().min(sample_count - samples.len());
        vm.render_into(&mut scratch[..length])
            .expect("the render must succeed");
        samples.extend_from_slice(&scratch[..length]);
    }

    samples
}

fn reference_samples(program: &CompiledProgram) -> Vec<f32> {
    let mut reference = ReferenceInterpreter::new(&program.typed);
    (0..program.typed.render.render_samples)
        .map(|index| {
            reference
                .next_sample(index, program.config.sample_rate)
                .expect("the finite reference corpus must render")
        })
        .collect()
}

fn assert_close(actual: f32, expected: f32, sample_index: usize) {
    let tolerance = 2e-5 + 2e-5 * expected.abs();
    assert!(
        (actual - expected).abs() <= tolerance,
        "sample {sample_index}: actual {actual}, expected {expected}, tolerance {tolerance}"
    );
}

fn span_text(source: &str, span: SourceSpan) -> &str {
    &source[span.start()..span.end()]
}

#[test]
fn spec_4_5_lexer_recognizes_function_block_and_pipeline_punctuation() {
    let tokens = lex("{} => |> =", test_source_id()).expect("the punctuation should lex");
    let kinds: Vec<_> = tokens.into_iter().map(|token| token.kind).collect();
    assert_eq!(
        kinds,
        [
            TokenKind::LeftBrace,
            TokenKind::RightBrace,
            TokenKind::FatArrow,
            TokenKind::PipeGreater,
            TokenKind::Equal,
            TokenKind::Eof,
        ]
    );

    let single_pipe = lex("|", test_source_id()).expect_err("a single pipe is invalid");
    assert_eq!(single_pipe.code(), ErrorCode::UnknownCharacter);
    assert_eq!(
        (
            single_pipe.primary().span().start(),
            single_pipe.primary().span().end()
        ),
        (0, 1)
    );
}

#[test]
fn spec_4_5_parser_preserves_function_declarations_and_signature_spans() {
    let source = include_str!("../fixtures/valid/stage-04-mix.ln");
    let program = parse(source, test_source_id()).expect("the mix fixture should parse");
    assert_eq!(program.declarations.len(), 4);
    assert!(matches!(
        &program.declarations[3],
        linen::syntax::TopDecl::Let(_)
    ));
    let linen::syntax::TopDecl::Function(function) = &program.declarations[0] else {
        panic!("the first declaration must be a function")
    };
    assert_eq!(function.name.name, "mix");
    assert_eq!(function.parameters.len(), 3);
    assert_eq!(function.result.kind, TypeName::Sig);
    assert_eq!(
        span_text(source, function.span),
        &source[..source.find(";\n\n").unwrap() + 1]
    );
    for parameter in &function.parameters {
        assert_eq!(parameter.name.span.source().index(), 0);
        assert_eq!(parameter.annotation.span.source().index(), 0);
        assert_eq!(
            span_text(source, parameter.span),
            match parameter.name.name.as_str() {
                "a" => "a: Sig",
                "b" => "b: Sig",
                "balance" => "balance: Sig",
                other => panic!("unexpected parameter {other}"),
            }
        );
    }
}

#[test]
fn spec_4_5_parser_builds_blocks_with_locals_and_required_results() {
    let source = include_str!("../fixtures/valid/stage-04-blocks-pipeline.ln");
    let program = parse(source, test_source_id()).expect("the block fixture should parse");
    let output = program.bindings().pop().expect("out exists");
    let ExprKind::Block { bindings, result } = &output.value.kind else {
        panic!("top-level out must be a block")
    };
    assert_eq!(bindings.len(), 1);
    assert_eq!(bindings[0].name.name, "wave");
    assert!(matches!(result.kind, ExprKind::Call { .. }));
    assert_eq!(span_text(source, output.value.span).as_bytes()[0], b'{');
    assert_eq!(
        span_text(source, output.value.span).as_bytes().last(),
        Some(&b'}')
    );
}

#[test]
fn spec_4_5_pipeline_desugars_with_lhs_as_first_argument_and_exact_span() {
    let wrapped = "let out = 1 + 2 |> f(3); render 1s;";
    let program = parse(wrapped, test_source_id()).expect("pipeline syntax should parse");
    let output = &program.bindings()[0].value;
    let ExprKind::Call { callee, arguments } = &output.kind else {
        panic!("pipeline must desugar to a call")
    };
    assert_eq!(callee.name, "f");
    assert_eq!(arguments.len(), 2);
    assert!(matches!(arguments[0].kind, ExprKind::Binary { .. }));
    assert!(matches!(arguments[1].kind, ExprKind::Number(3.0)));
    assert_eq!(span_text(wrapped, output.span), "1 + 2 |> f(3)");
    assert_eq!(span_text(wrapped, callee.span), "f");
}

#[test]
fn spec_4_5_pipeline_is_left_associative_and_weaker_than_addition() {
    let chained = expression("x |> f |> g(2)");
    let ExprKind::Call { callee, arguments } = &chained.kind else {
        panic!("outer pipeline must be a call")
    };
    assert_eq!(callee.name, "g");
    assert!(matches!(&arguments[0].kind, ExprKind::Call { callee, .. } if callee.name == "f"));

    let additive = expression("1 + 2 |> f");
    let ExprKind::Call { arguments, .. } = &additive.kind else {
        panic!("pipeline must contain the additive lhs")
    };
    assert!(matches!(arguments[0].kind, ExprKind::Binary { .. }));
}

#[test]
fn spec_4_5_malformed_functions_blocks_and_pipeline_are_rejected() {
    for source in [
        "let out = {}; render 1s;",
        "let out = { let x = 1; }; render 1s;",
        "let out = 1 |> (f); render 1s;",
        "let f = (x Sig): Sig => x; let out = 1; render 1s;",
    ] {
        let diagnostic = parse(source, test_source_id()).expect_err("malformed syntax must fail");
        assert_eq!(diagnostic.code(), ErrorCode::UnexpectedToken);
    }
}

#[test]
fn spec_5_5_typed_ast_resolves_function_ids_parameters_and_pipeline_args() {
    let source = include_str!("../fixtures/valid/stage-04-mix.ln");
    let outcome = compile(source, RenderConfig::default()).expect("the render config is valid");
    let program = outcome.result.as_ref().expect("mix should compile");
    assert_eq!(outcome.symbols.function_len(), 1);
    let function = program
        .typed
        .functions()
        .next()
        .expect("one function exists");
    assert_eq!(function.id, FunctionId::new(0));
    assert_eq!(function.parameters.len(), 3);
    assert_eq!(
        outcome.symbols.get_function(function.id).unwrap().name,
        "mix"
    );
    let output = program.typed.bindings().pop().unwrap();
    let TypedExprKind::FunctionCall {
        function: id,
        arguments,
        ..
    } = &output.value.kind
    else {
        panic!("pipeline must become a typed user call")
    };
    assert_eq!(*id, function.id);
    assert_eq!(arguments.len(), 3);
    assert!(matches!(
        arguments[2].kind,
        TypedExprKind::LiftFloatToSig(_)
    ));
}

#[test]
fn spec_4_5_capture_and_nested_shadowing_use_distinct_stable_binding_ids() {
    let source = "let base = 2;\nlet f = (base: Float): Float => { let x = base; { let x = base + 1; x } };\nlet alias = base;\nlet out = f(alias);\nrender 1s;";
    let outcome = compile(source, RenderConfig::default()).expect("the render config is valid");
    outcome
        .result
        .as_ref()
        .expect("capture and shadowing should compile");
    assert_eq!(outcome.symbols.len(), 6);
    let names: Vec<_> = (0..6)
        .map(|index| {
            let symbol = outcome
                .symbols
                .get(linen::sema::BindingId::new(index))
                .unwrap();
            (symbol.id.index(), symbol.name.as_str())
        })
        .collect();
    assert_eq!(
        names,
        [
            (0, "base"),
            (1, "base"),
            (2, "x"),
            (3, "x"),
            (4, "alias"),
            (5, "out")
        ]
    );
}

#[test]
fn spec_6_2_duplicate_parameters_locals_and_function_names_have_exact_spans() {
    let duplicate = include_str!("../fixtures/invalid/stage-04-duplicate-param.ln");
    let error = diagnostic(duplicate, ErrorCode::DuplicateBinding);
    let start = duplicate.find("x: Float").unwrap();
    assert_primary_from(duplicate, &error, start, 1);
    assert_eq!(error.secondary().len(), 1);
    assert_eq!(span_text(duplicate, error.secondary()[0].span()), "x");

    let local = "let f = (x: Float): Float => { let x = 1; x }; let out = 1; render 1s;";
    let error = diagnostic(local, ErrorCode::DuplicateBinding);
    let start = local.find("x = 1").unwrap();
    assert_primary_from(local, &error, start, 1);

    for source in [
        "let sine = (x: Sig): Sig => x; let out = 1; render 1s;",
        "let f = 1; let f = (x: Float): Float => x; let out = 1; render 1s;",
    ] {
        let error = diagnostic(source, ErrorCode::DuplicateBinding);
        assert_eq!(error.primary().span().source().index(), 0);
    }
}

#[test]
fn spec_4_5_forward_self_and_first_class_function_uses_are_rejected() {
    let recursive = include_str!("../fixtures/invalid/stage-04-recursion.ln");
    let error = diagnostic(recursive, ErrorCode::UnknownFunction);
    let start = recursive.rfind("recursive").unwrap();
    assert_primary_from(recursive, &error, start, "recursive".len());

    let forward =
        "let f = (x: Sig): Sig => g(x); let g = (x: Sig): Sig => x; let out = 1; render 1s;";
    let error = diagnostic(forward, ErrorCode::UnknownFunction);
    let start = forward.find("g(x)").unwrap();
    assert_primary_from(forward, &error, start, 1);

    let value = "let id = (x: Sig): Sig => x; let alias = id; let out = 1; render 1s;";
    let error = diagnostic(value, ErrorCode::UnknownName);
    let start = value.find("id;").unwrap();
    assert_primary_from(value, &error, start, 2);
}

#[test]
fn spec_6_2_user_call_arity_precedes_arguments_and_type_spans_are_stable() {
    let arity = "let f = (x: Sig): Sig => x; let out = f(unknown, 2); render 1s;";
    let error = diagnostic(arity, ErrorCode::InvalidArity);
    assert_primary_from(arity, &error, arity.rfind("f(").unwrap(), 1);

    let argument = "let f = (x: Float): Float => x; let out = f(sine(1)); render 1s;";
    let error = diagnostic(argument, ErrorCode::TypeMismatch);
    assert_primary(argument, &error, "sine(1)");
    assert_eq!(error.secondary().len(), 1);
    assert_eq!(span_text(argument, error.secondary()[0].span()), "Float");

    let result = "let f = (x: Sig): Float => x; let out = 1; render 1s;";
    let error = diagnostic(result, ErrorCode::TypeMismatch);
    assert_primary_from(result, &error, result.rfind("=> x").unwrap() + 3, 1);
    assert_eq!(span_text(result, error.secondary()[0].span()), "Float");
}

#[test]
fn spec_4_5_unused_function_checks_known_constants_and_per_call_float_origins() {
    let unused = "let bad = (): Float => 1 / 0; let out = 1; render 1s;";
    let error = diagnostic(unused, ErrorCode::ConstantDivisionByZero);
    assert_primary_from(unused, &error, unused.rfind("0").unwrap(), 1);

    let per_call = include_str!("../fixtures/invalid/stage-04-per-call-float.ln");
    let error = diagnostic(per_call, ErrorCode::InvalidFloat);
    assert_primary(per_call, &error, "x * 1e308");
    assert_eq!(error.secondary().len(), 1);
    assert_eq!(
        span_text(per_call, error.secondary()[0].span()),
        "scale(1e308)"
    );
}

#[test]
fn spec_10_1_function_limits_have_defaults_and_zero_only_rejects_user_calls() {
    let defaults = FunctionLimits::default();
    assert_eq!(defaults.max_call_depth, 128);
    assert_eq!(defaults.max_expanded_nodes, 10_000);
    let config = CompileConfig {
        functions: FunctionLimits {
            max_call_depth: 0,
            max_expanded_nodes: 0,
        },
        ..CompileConfig::default()
    };
    check_with_config("beep.ln", "let out = sine(1); render 1s;", config)
        .result
        .expect("builtins do not consume function expansion budget");

    let outcome = check_with_config(
        "call.ln",
        "let id = (x: Sig): Sig => x; let out = id(1); render 1s;",
        config,
    );
    let error = outcome.result.expect_err("zero limits reject a user call");
    assert_eq!(error.code(), ErrorCode::FunctionExpansionLimit);
}

#[test]
fn spec_10_1_call_depth_and_expanded_node_boundaries_are_exact() {
    let nested = "let inner = (x: Sig): Sig => x; let outer = (x: Sig): Sig => inner(x); let out = outer(1); render 1s;";
    for (limit, succeeds) in [(1, false), (2, true)] {
        let config = CompileConfig {
            functions: FunctionLimits {
                max_call_depth: limit,
                ..FunctionLimits::default()
            },
            ..CompileConfig::default()
        };
        let result = check_with_config("depth.ln", nested, config).result;
        assert_eq!(result.is_ok(), succeeds, "depth limit {limit}");
        if let Err(error) = result {
            assert_eq!(error.code(), ErrorCode::FunctionExpansionLimit);
        }
    }

    let identity = "let id = (x: Sig): Sig => x; let out = id(1); render 1s;";
    for (limit, succeeds) in [(2, false), (3, true)] {
        let config = CompileConfig {
            functions: FunctionLimits {
                max_expanded_nodes: limit,
                ..FunctionLimits::default()
            },
            ..CompileConfig::default()
        };
        let result = check_with_config("nodes.ln", identity, config).result;
        assert_eq!(result.is_ok(), succeeds, "node limit {limit}");
        if let Err(error) = result {
            assert_eq!(error.code(), ErrorCode::FunctionExpansionLimit);
            assert_primary(identity, &error, "id(1)");
        }
    }
}

#[test]
fn adr_0007_shared_binding_and_parameter_reuse_one_sine_and_eager_roots() {
    let program = compile_program(include_str!("../fixtures/valid/stage-04-sharing.ln"));
    let operations: Vec<_> = program
        .ir
        .nodes
        .iter()
        .map(|node| node.op.clone())
        .collect();
    assert_eq!(operations.len(), 4);
    assert!(matches!(operations[2], NodeOp::Sine(_)));
    assert_eq!(
        operations[3],
        NodeOp::Add(linen::ir::NodeId::new(2), linen::ir::NodeId::new(2))
    );
    assert_eq!(
        program.ir.roots,
        [
            linen::ir::NodeId::new(2),
            linen::ir::NodeId::new(2),
            linen::ir::NodeId::new(3)
        ]
    );
    assert_eq!(program.bytecode.state_count, 1);
}

#[test]
fn adr_0007_each_user_call_creates_fresh_sine_state() {
    let program = compile_program(include_str!("../fixtures/valid/stage-04-two-calls.ln"));
    assert_eq!(
        program
            .ir
            .nodes
            .iter()
            .filter(|node| matches!(node.op, NodeOp::Sine(_)))
            .count(),
        2
    );
    assert_eq!(program.bytecode.state_count, 2);
}

#[test]
fn adr_0007_unused_parameter_still_has_an_eager_root() {
    let program = compile_program(
        "let ignore = (x: Sig): Sig => 1; let out = ignore(sine(480)); render 1ms;",
    );
    assert!(
        program
            .ir
            .nodes
            .iter()
            .any(|node| matches!(node.op, NodeOp::Sine(_)))
    );
    assert!(program.ir.roots.contains(&program.ir.output));
    assert_eq!(program.ir.roots.len(), 2);
    assert_eq!(program.bytecode.state_count, 1);
}

#[test]
fn adr_0007_eager_unused_arguments_and_locals_match_reference_errors() {
    for source in [
        "let ignore = (x: Sig): Sig => 1; let out = ignore(sine(sine(0) / 0)); render 1ms;",
        include_str!("../fixtures/invalid/stage-04-eager-unused-local.ln"),
    ] {
        let program = compile_program(source);
        let mut reference = ReferenceInterpreter::new(&program.typed);
        let reference_error = reference
            .next_sample(0, 48_000)
            .expect_err("reference must fail eagerly");
        assert_eq!(reference_error.kind, ReferenceErrorKind::NonFiniteFrequency);
        assert_eq!(reference_error.sample_index, 0);
        let mut vm = create_vm(&program).expect("the eager graph must verify");
        let error = vm
            .render_into(&mut [0.0])
            .expect_err("VM must preserve the eager error");
        assert!(matches!(
            error,
            VmRuntimeError::NonFiniteFrequency {
                sample_index: 0,
                ..
            }
        ));
    }
}

#[test]
fn adr_0007_nested_origins_keep_function_ids_and_inner_to_outer_call_spans() {
    let source = include_str!("../fixtures/valid/stage-04-nested-origins.ln");
    let outcome = compile(source, RenderConfig::default()).expect("the render config is valid");
    let program = outcome
        .result
        .as_ref()
        .expect("nested calls should compile");
    let sine = program
        .ir
        .nodes
        .iter()
        .find(|node| matches!(node.op, NodeOp::Sine(_)))
        .expect("nested body contains sine");
    assert_eq!(span_text(source, sine.origin.primary), "x");
    assert_eq!(sine.origin.inlined_from.len(), 2);
    assert_eq!(sine.origin.inlined_from[0].function, FunctionId::new(0));
    assert_eq!(sine.origin.inlined_from[1].function, FunctionId::new(1));
    assert_eq!(
        span_text(source, sine.origin.inlined_from[0].call_span),
        "inner(x)"
    );
    assert_eq!(
        span_text(source, sine.origin.inlined_from[1].call_span),
        "440 |> outer"
    );
    verify_ir(&program.ir, &outcome.sources).expect("all call origins must verify");
    linen::bytecode::verify_origins(&program.bytecode, &outcome.sources)
        .expect("bytecode retains the same valid origins");
}

#[test]
fn spec_5_6_and_5_7_origin_verifiers_reject_invalid_inline_spans() {
    let mut sources = SourceMap::new();
    let source = sources.add_source("origin.ln", "0");
    let mut other = SourceMap::new();
    other.add_source("zero.ln", "0");
    let unknown = other.add_source("one.ln", "1");
    let span = SourceSpan::new(source, 0, 1).unwrap();
    let invalid_span = SourceSpan::empty(unknown, 0);
    let mut graph = linen::ir::Graph {
        nodes: vec![
            linen::ir::Node {
                id: linen::ir::NodeId::new(0),
                op: NodeOp::ConstFloat(1.0),
                result_type: linen::ir::ValueType::Float,
                origin: NodeOrigin::new(span),
            },
            linen::ir::Node {
                id: linen::ir::NodeId::new(1),
                op: NodeOp::LiftFloatToSig(linen::ir::NodeId::new(0)),
                result_type: linen::ir::ValueType::Sig,
                origin: NodeOrigin::new(span),
            },
            linen::ir::Node {
                id: linen::ir::NodeId::new(2),
                op: NodeOp::Sine(linen::ir::NodeId::new(1)),
                result_type: linen::ir::ValueType::Sig,
                origin: NodeOrigin::new(span),
            },
        ],
        roots: vec![linen::ir::NodeId::new(2)],
        output: linen::ir::NodeId::new(2),
        render_samples: 1,
    };
    graph.nodes[0].origin.inlined_from.push(CallOrigin {
        function: FunctionId::new(0),
        call_span: invalid_span,
    });
    assert_eq!(
        verify_ir(&graph, &sources),
        Err(IrVerifyError::InvalidInlineOrigin {
            node: linen::ir::NodeId::new(0),
            inline_index: 0,
            span: invalid_span,
        })
    );

    let mut outcome = compile("let out = sine(480); render 1ms;", RenderConfig::default())
        .expect("render config is valid");
    let program = outcome.result.as_mut().expect("Beep should compile");
    program.bytecode.debug_origins.as_mut().unwrap()[0]
        .inlined_from
        .push(CallOrigin {
            function: FunctionId::new(0),
            call_span: invalid_span,
        });
    assert_eq!(
        linen::bytecode::verify_origins(&program.bytecode, &outcome.sources),
        Err(linen::bytecode::BytecodeVerifyError::InvalidOrigin { instruction: 0 })
    );
}

#[test]
fn spec_7_4_and_7_5_mix_matches_reference_at_every_required_block_size() {
    let source = include_str!("../fixtures/valid/stage-04-mix.ln");
    for block_size in [1, 3, 64, 128, 256] {
        let program = compile_with_block_size(source, block_size);
        let actual = vm_samples(&program);
        let expected = reference_samples(&program);
        for (index, (actual, expected)) in actual.iter().zip(&expected).enumerate() {
            assert_close(*actual, *expected, index);
        }
    }
}

#[test]
fn spec_4_5_pipeline_and_explicit_call_have_identical_ir_ops_and_samples() {
    let pipeline_source = include_str!("../fixtures/valid/stage-04-mix.ln");
    let explicit_source =
        pipeline_source.replace("left |> mix(right, 0.25)", "mix(left, right, 0.25)");
    let pipeline = compile_program(pipeline_source);
    let explicit = compile_program(&explicit_source);
    let pipeline_ops: Vec<_> = pipeline
        .ir
        .nodes
        .iter()
        .map(|node| node.op.clone())
        .collect();
    let explicit_ops: Vec<_> = explicit
        .ir
        .nodes
        .iter()
        .map(|node| node.op.clone())
        .collect();
    assert_eq!(pipeline_ops, explicit_ops);
    assert_eq!(pipeline.ir.roots, explicit.ir.roots);
    assert_eq!(vm_samples(&pipeline), vm_samples(&explicit));
}

#[test]
fn spec_4_8_function_state_reset_and_debug_stripping_preserve_samples() {
    let program = compile_program(include_str!("../fixtures/valid/stage-04-two-calls.ln"));
    let mut vm = create_vm(&program).expect("VM creation works");
    let mut first = vec![0.0; 48];
    vm.render_into(&mut first).expect("first render works");
    vm.reset();
    let mut second = vec![0.0; 48];
    vm.render_into(&mut second).expect("second render works");
    assert_eq!(first, second);

    let mut reference = ReferenceInterpreter::new(&program.typed);
    let expected = reference
        .next_sample(0, 48_000)
        .expect("reference sample works");
    reference
        .next_sample(1, 48_000)
        .expect("second reference sample works");
    reference.reset();
    assert_eq!(
        reference.next_sample(0, 48_000).unwrap().to_bits(),
        expected.to_bits()
    );

    let mut stripped = program.clone();
    stripped.bytecode.strip_debug_info();
    assert_eq!(vm_samples(&program), vm_samples(&stripped));
}

#[test]
fn spec_6_2_runtime_frequency_diagnostic_contains_definition_and_call_chain() {
    let source = "let inner = (x: Sig): Sig => sine(x);\nlet outer = (x: Sig): Sig => inner(x);\nlet out = outer(sine(0) / 0);\nrender 1ms;";
    let outcome = compile(source, RenderConfig::default()).expect("the render config is valid");
    let program = outcome
        .result
        .as_ref()
        .expect("runtime-error source should compile");
    let error = linen::render(program, &mut Vec::new()).expect_err("sample zero must fail");
    let linen::RenderError::Diagnostic(error) = error else {
        panic!("the runtime failure must become a source diagnostic")
    };
    assert_eq!(error.code(), ErrorCode::NonFiniteOutput);
    assert_eq!(error.sample_index(), Some(0));
    let primary_start = source.find("sine(x)").unwrap() + 5;
    assert_primary_from(source, &error, primary_start, 1);
    let secondary: Vec<_> = error
        .secondary()
        .iter()
        .map(|label| span_text(source, label.span()))
        .collect();
    assert_eq!(secondary, ["inner(x)", "outer(sine(0) / 0)"]);
}

#[test]
fn spec_5_11_compilation_with_functions_is_structurally_deterministic() {
    let source = include_str!("../fixtures/valid/stage-04-blocks-pipeline.ln");
    let first = compile_with_config(
        "one.ln",
        source,
        CompileConfig::default(),
        RenderConfig::default(),
    )
    .expect("config is valid");
    let second = compile_with_config(
        "two.ln",
        source,
        CompileConfig::default(),
        RenderConfig::default(),
    )
    .expect("config is valid");
    let first = first.result.expect("first compilation works");
    let second = second.result.expect("second compilation works");
    assert_eq!(first.typed, second.typed);
    assert_eq!(first.ir, second.ir);
    assert_eq!(first.bytecode, second.bytecode);
}

#[test]
fn spec_5_8_compile_error_keeps_stage_4_source_diagnostics_distinct_from_invariants() {
    let source = include_str!("../fixtures/invalid/stage-04-recursion.ln");
    let outcome = compile(source, RenderConfig::default()).expect("render config is valid");
    let error = outcome.result.expect_err("recursion is a source error");
    assert!(matches!(error, CompileError::Diagnostic(_)));
    assert_eq!(
        error.as_diagnostic().unwrap().code(),
        ErrorCode::UnknownFunction
    );
}
