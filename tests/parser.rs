use std::panic::catch_unwind;

use linen::syntax::{
    BinaryOp, DurationUnit, Expr, ExprKind, ParserConfig, Token, TokenKind, TypeName, UnaryOp,
    parse, parse_tokens_with_config, parse_with_config,
};
use linen::{ErrorCode, Span};

fn span(start: usize, end: usize) -> Span {
    Span::new(start, end).expect("the test span is valid")
}

fn expression(source: &str) -> Expr {
    let source = format!("let out = {source}; render 1s;");
    let program = parse(&source).expect("the expression should parse");
    program
        .bindings
        .into_iter()
        .next()
        .expect("there is one binding")
        .value
}

fn as_binary(expression: &Expr) -> (BinaryOp, &Expr, &Expr) {
    match &expression.kind {
        ExprKind::Binary {
            operator,
            left,
            right,
        } => (*operator, left, right),
        other => panic!("expected a binary expression, got {other:?}"),
    }
}

fn assert_parse_error(source: &str, code: ErrorCode, primary: Span) {
    let diagnostic = parse(source).expect_err("the source should not parse");

    assert_eq!(diagnostic.code(), code);
    assert_eq!(diagnostic.primary().span(), primary);
}

#[test]
fn spec_4_3_beep_fixture_builds_the_expected_ast() {
    let source = include_str!("../fixtures/valid/stage-01-beep.ln");
    let program = parse(source).expect("Beep should parse");

    assert_eq!(program.span, span(0, 37));
    assert_eq!(program.bindings.len(), 1);
    let binding = &program.bindings[0];
    assert_eq!(binding.name.name, "out");
    assert_eq!(binding.name.span, span(4, 7));
    assert_eq!(binding.span, span(0, 26));
    assert!(binding.annotation.is_none());

    let (operator, left, right) = as_binary(&binding.value);
    assert_eq!(operator, BinaryOp::Multiply);
    assert_eq!(binding.value.span, span(10, 25));
    assert!(matches!(right.kind, ExprKind::Number(value) if value == 0.1));
    assert_eq!(right.span, span(22, 25));

    match &left.kind {
        ExprKind::Call { callee, arguments } => {
            assert_eq!(callee.name, "sine");
            assert_eq!(callee.span, span(10, 14));
            assert_eq!(arguments.len(), 1);
            assert!(matches!(arguments[0].kind, ExprKind::Number(value) if value == 440.0));
            assert_eq!(arguments[0].span, span(15, 18));
        }
        other => panic!("expected sine call, got {other:?}"),
    }
    assert_eq!(left.span, span(10, 19));

    assert!(!program.render.negative);
    assert_eq!(program.render.duration.value, 2.0);
    assert_eq!(program.render.duration.unit, DurationUnit::Seconds);
    assert_eq!(program.render.duration.span, span(34, 36));
    assert_eq!(program.render.span, span(27, 37));
}

#[test]
fn spec_4_3_multiplication_binds_more_strongly_than_addition() {
    let expression = expression("1 + 2 * 3");
    let (operator, left, right) = as_binary(&expression);

    assert_eq!(operator, BinaryOp::Add);
    assert!(matches!(left.kind, ExprKind::Number(1.0)));
    assert_eq!(as_binary(right).0, BinaryOp::Multiply);
}

#[test]
fn spec_4_3_subtraction_and_division_are_left_associative() {
    let subtraction = expression("a - b - c");
    let (operator, left, right) = as_binary(&subtraction);
    assert_eq!(operator, BinaryOp::Subtract);
    assert_eq!(as_binary(left).0, BinaryOp::Subtract);
    assert!(matches!(&right.kind, ExprKind::Identifier(name) if name.name == "c"));

    let division = expression("a / b / c");
    let (operator, left, right) = as_binary(&division);
    assert_eq!(operator, BinaryOp::Divide);
    assert_eq!(as_binary(left).0, BinaryOp::Divide);
    assert!(matches!(&right.kind, ExprKind::Identifier(name) if name.name == "c"));
}

#[test]
fn spec_4_3_parentheses_and_unary_minus_preserve_their_structure() {
    let grouped = expression("(1 + 2) * 3");
    let (operator, left, _) = as_binary(&grouped);
    assert_eq!(operator, BinaryOp::Multiply);
    match &left.kind {
        ExprKind::Group(inner) => assert_eq!(as_binary(inner).0, BinaryOp::Add),
        other => panic!("expected a group, got {other:?}"),
    }

    let negated = expression("-sine(440) * 2");
    let (operator, left, _) = as_binary(&negated);
    assert_eq!(operator, BinaryOp::Multiply);
    match &left.kind {
        ExprKind::Unary { operator, operand } => {
            assert_eq!(*operator, UnaryOp::Negate);
            assert!(matches!(operand.kind, ExprKind::Call { .. }));
        }
        other => panic!("expected unary minus, got {other:?}"),
    }
}

#[test]
fn spec_4_3_type_annotations_and_negative_durations_are_syntax() {
    let program = parse("let out: Sig = 1; render - 0s;").expect("syntax should parse");

    let annotation = program.bindings[0]
        .annotation
        .as_ref()
        .expect("there is an annotation");
    assert_eq!(annotation.kind, TypeName::Sig);
    assert_eq!(annotation.span, span(9, 12));
    assert!(program.render.negative);
    assert_eq!(program.render.duration.value, 0.0);
}

#[test]
fn spec_4_3_call_arguments_are_a_comma_separated_expression_list() {
    let empty = expression("sine()");
    assert!(matches!(empty.kind, ExprKind::Call { ref arguments, .. } if arguments.is_empty()));

    let multiple = expression("unknown(1, 2 + 3)");
    match multiple.kind {
        ExprKind::Call { callee, arguments } => {
            assert_eq!(callee.name, "unknown");
            assert_eq!(arguments.len(), 2);
            assert_eq!(as_binary(&arguments[1]).0, BinaryOp::Add);
        }
        other => panic!("expected a call, got {other:?}"),
    }
}

#[test]
fn spec_4_3_missing_and_trailing_call_arguments_are_e1100() {
    assert_parse_error(
        "let out = sine(,); render 1s;",
        ErrorCode::UnexpectedToken,
        span(15, 16),
    );
    assert_parse_error(
        "let out = sine(1,); render 1s;",
        ErrorCode::UnexpectedToken,
        span(17, 18),
    );
    assert_parse_error(
        "let out = sine(1",
        ErrorCode::UnexpectedToken,
        Span::empty(16),
    );
}

#[test]
fn spec_4_3_only_an_identifier_can_be_a_call_target() {
    for (source, unexpected_paren) in [
        ("let out = (sine)(1); render 1s;", 16),
        ("let out = 1(2); render 1s;", 11),
        ("let out = sine(1)(2); render 1s;", 17),
    ] {
        assert_parse_error(
            source,
            ErrorCode::UnexpectedToken,
            span(unexpected_paren, unexpected_paren + 1),
        );
    }
}

#[test]
fn spec_4_3_missing_semicolon_is_e1101_at_the_next_token() {
    let source = include_str!("../fixtures/invalid/stage-01-missing-semicolon.ln");
    assert_parse_error(source, ErrorCode::MissingSemicolon, Span::empty(12));
    assert_parse_error("let out = 1", ErrorCode::MissingSemicolon, Span::empty(11));
}

#[test]
fn spec_4_3_render_is_checked_before_out_and_must_be_final() {
    assert_parse_error("", ErrorCode::MissingOrMisplacedRender, Span::empty(0));
    assert_parse_error(
        "let out = 1;",
        ErrorCode::MissingOrMisplacedRender,
        Span::empty(12),
    );

    let misplaced = include_str!("../fixtures/invalid/stage-01-misplaced-render.ln");
    let diagnostic = parse(misplaced).expect_err("render must be final");
    assert_eq!(diagnostic.code(), ErrorCode::MissingOrMisplacedRender);
    assert_eq!(diagnostic.primary().span(), span(0, 6));
    assert_eq!(diagnostic.secondary().len(), 1);
    assert_eq!(diagnostic.secondary()[0].span(), span(11, 14));
}

#[test]
fn spec_4_3_missing_and_duplicate_out_have_stable_spans() {
    assert_parse_error("render 1s;", ErrorCode::MissingOutput, span(0, 6));

    let source = include_str!("../fixtures/invalid/stage-01-duplicate-out.ln");
    let diagnostic = parse(source).expect_err("duplicate out should fail");
    assert_eq!(diagnostic.code(), ErrorCode::MultipleOutputs);
    assert_eq!(diagnostic.primary().span(), span(17, 20));
    assert_eq!(diagnostic.secondary().len(), 1);
    assert_eq!(diagnostic.secondary()[0].span(), span(4, 7));
}

#[test]
fn spec_4_3_top_level_expressions_and_durations_outside_render_are_rejected() {
    assert_parse_error("1;", ErrorCode::UnexpectedToken, span(0, 1));
    assert_parse_error(
        "let out = true; render 1s;",
        ErrorCode::UnexpectedToken,
        span(10, 14),
    );
    assert_parse_error(
        "let out = 1s; render 1s;",
        ErrorCode::UnexpectedToken,
        span(10, 12),
    );
}

#[test]
fn spec_9_stage_1_does_not_run_name_type_or_duration_checks() {
    let source = include_str!("../fixtures/valid/stage-01-semantics-deferred.ln");
    let program = parse(source).expect("semantic checks belong to stage 2");

    assert_eq!(program.bindings.len(), 4);
    assert_eq!(program.bindings[0].name.name, "value");
    assert_eq!(program.bindings[1].name.name, "later");
    assert_eq!(program.bindings[2].name.name, "value");
    assert_eq!(
        program.bindings[3].annotation.as_ref().map(|a| a.kind),
        Some(TypeName::Float)
    );
    assert!(program.render.negative);
}

#[test]
fn spec_10_1_expression_depth_has_an_explicit_boundary_and_e1102() {
    let config = ParserConfig {
        max_depth: 2,
        max_nodes: 10_000,
    };
    let accepted = "let out = ((1)); render 1s;";
    let rejected = "let out = (((1))); render 1s;";

    parse_with_config(accepted, config).expect("two nested groups should be accepted");
    let diagnostic = parse_with_config(rejected, config)
        .expect_err("the third nested group exceeds the configured depth");
    assert_eq!(diagnostic.code(), ErrorCode::ExpressionDepthLimit);
    assert_eq!(diagnostic.primary().span(), span(12, 13));
    assert_eq!(ParserConfig::default().max_depth, 128);
}

#[test]
fn spec_10_1_ast_node_limit_has_an_explicit_boundary_and_e1103() {
    let source = "let out = 1; render 1s;";
    let accepted = ParserConfig {
        max_depth: 128,
        max_nodes: 3,
    };
    let rejected = ParserConfig {
        max_nodes: 2,
        ..accepted
    };

    parse_with_config(source, accepted).expect("let, number and render are three nodes");
    let diagnostic = parse_with_config(source, rejected)
        .expect_err("the render declaration exceeds the configured node count");
    assert_eq!(diagnostic.code(), ErrorCode::AstNodeLimit);
    assert_eq!(diagnostic.primary().span(), span(13, 19));
    assert_eq!(ParserConfig::default().max_nodes, 10_000);
}

#[test]
fn spec_6_2_ast_node_limit_uses_the_expression_start_token() {
    let binary = include_str!("../fixtures/valid/stage-01-expression-node-limit.ln");
    parse(binary).expect("the fixture is valid with the default node limit");

    let cases = [
        (binary, 3),
        ("let out = 1 * 2; render 1s;", 3),
        ("let out = -1; render 1s;", 2),
        ("let out = (1); render 1s;", 2),
    ];

    for (source, max_nodes) in cases {
        let diagnostic = parse_with_config(
            source,
            ParserConfig {
                max_depth: 128,
                max_nodes,
            },
        )
        .expect_err("the composite expression should exceed the node limit");

        assert_eq!(diagnostic.code(), ErrorCode::AstNodeLimit);
        assert_eq!(diagnostic.primary().span(), span(10, 11));
    }
}

#[test]
fn spec_7_6_parser_does_not_panic_on_deterministic_arbitrary_token_streams() {
    let mut state = 0x4c49_4e45_4e31_u64;

    for case_index in 0..64 {
        let mut tokens = Vec::new();
        for token_index in 0..24 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let kind = match (state >> 32) % 14 {
                0 => TokenKind::Let,
                1 => TokenKind::Render,
                2 => TokenKind::Number(1.0),
                3 => TokenKind::Identifier("name".into()),
                4 => TokenKind::Plus,
                5 => TokenKind::Minus,
                6 => TokenKind::Star,
                7 => TokenKind::Slash,
                8 => TokenKind::LeftParen,
                9 => TokenKind::RightParen,
                10 => TokenKind::Comma,
                11 => TokenKind::Semicolon,
                12 => TokenKind::Eof,
                _ => TokenKind::Duration {
                    value: 1.0,
                    unit: DurationUnit::Seconds,
                },
            };
            tokens.push(Token::new(kind, Span::empty(token_index)));
        }

        let result = catch_unwind(|| parse_tokens_with_config(&tokens, ParserConfig::default()));
        assert!(
            result.is_ok(),
            "parser panicked for corpus case {case_index}"
        );
    }
}
