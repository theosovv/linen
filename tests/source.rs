use linen::syntax::ParserConfig;
use linen::{
    CheckOutcome, CompileConfig, CompileSession, Diagnostic, DiagnosticLabel, ErrorCode,
    LineColumn, SourceId, SourceMap, SourceSpan, check, check_with_config, render_diagnostic,
};

fn unknown_source_id() -> SourceId {
    let mut sources = SourceMap::new();
    sources.add_source("first.ln", "first");
    sources.add_source("second.ln", "second");
    sources.add_source("third.ln", "third")
}

fn diagnostic(span: SourceSpan) -> Diagnostic {
    let diagnostic = Diagnostic::error(
        ErrorCode::UnknownCharacter,
        "unexpected character",
        DiagnosticLabel::new(span, "this character is not recognized"),
    );

    assert_eq!(diagnostic.code(), ErrorCode::UnknownCharacter);
    assert_eq!(diagnostic.primary().span(), span);
    diagnostic
}

fn assert_rendered_error(outcome: &CheckOutcome) -> String {
    let diagnostic = outcome
        .result
        .as_ref()
        .expect_err("the source should be rejected");

    render_diagnostic(&outcome.sources, diagnostic).expect("the diagnostic should render")
}

#[test]
fn spec_5_8_source_map_assigns_deterministic_map_local_ids() {
    let mut first = SourceMap::new();
    let first_id = first.add_source("same.ln", "first text");
    let second_id = first.add_source("same.ln", "second text");

    assert_eq!(first_id.index(), 0);
    assert_eq!(second_id.index(), 1);
    assert_ne!(first_id, second_id);

    let first_file = first.get(first_id).expect("the first source should exist");
    assert_eq!(first_file.id(), first_id);
    assert_eq!(first_file.logical_name(), "same.ln");
    assert_eq!(first_file.text(), "first text");

    let second_file = first
        .get(second_id)
        .expect("the second source should exist");
    assert_eq!(second_file.id(), second_id);
    assert_eq!(second_file.logical_name(), "same.ln");
    assert_eq!(second_file.text(), "second text");

    let mut repeated = SourceMap::new();
    assert_eq!(repeated.add_source("same.ln", "first text"), first_id);
    assert_eq!(repeated.add_source("same.ln", "second text"), second_id);
}

#[test]
fn spec_4_3_source_span_is_a_source_aware_half_open_byte_range() {
    let mut sources = SourceMap::new();
    let source = sources.add_source("input.ln", "é");
    let span = SourceSpan::new(source, 0, 2).expect("the test span is valid");

    assert_eq!(span.source(), source);
    assert_eq!(span.start(), 0);
    assert_eq!(span.end(), 2);
    assert_eq!(span.len(), 2);
    assert!(!span.is_empty());

    let empty = SourceSpan::empty(source, 0);
    assert_eq!(empty.source(), source);
    assert_eq!(empty.start(), 0);
    assert_eq!(empty.end(), 0);
    assert_eq!(empty.len(), 0);
    assert!(empty.is_empty());

    let eof = SourceSpan::empty(source, 2);
    assert_eq!(eof.source(), source);
    assert_eq!(eof.start(), 2);
    assert_eq!(eof.end(), 2);
}

#[test]
fn spec_6_2_source_span_rejects_reversed_offsets_without_panicking() {
    let mut sources = SourceMap::new();
    let source = sources.add_source("input.ln", "0123456789");
    let error = SourceSpan::new(source, 9, 4).expect_err("a reversed span must be rejected");

    assert_eq!(error.start(), 9);
    assert_eq!(error.end(), 4);
    assert_eq!(
        error.to_string(),
        "invalid source span: start 9 is greater than end 4"
    );
}

#[test]
fn spec_4_3_line_index_counts_unicode_characters_and_crlf_once() {
    let source = "aé\r\n\tb";
    let mut sources = SourceMap::new();
    let id = sources.add_source("unicode.ln", source);
    let file = sources.get(id).expect("the source should exist");
    let cases = [
        (0, LineColumn::new(1, 1)),
        (1, LineColumn::new(1, 2)),
        (5, LineColumn::new(2, 1)),
        (6, LineColumn::new(2, 2)),
        (7, LineColumn::new(2, 3)),
    ];

    for (offset, expected) in cases {
        assert_eq!(file.line_column(offset), Some(expected));
    }
}

#[test]
fn spec_4_3_line_index_handles_lf_lone_cr_tab_empty_input_and_eof() {
    let mut sources = SourceMap::new();
    let lone_cr = sources.add_source("lone-cr.ln", "a\rb");
    assert_eq!(
        sources
            .get(lone_cr)
            .expect("the source should exist")
            .line_column(2),
        Some(LineColumn::new(1, 3))
    );

    let lf = sources.add_source("lf.ln", "a\nb");
    assert_eq!(
        sources
            .get(lf)
            .expect("the source should exist")
            .line_column(2),
        Some(LineColumn::new(2, 1))
    );

    let empty = sources.add_source("empty.ln", "");
    assert_eq!(
        sources
            .get(empty)
            .expect("the source should exist")
            .line_column(0),
        Some(LineColumn::new(1, 1))
    );
}

#[test]
fn spec_4_3_line_index_rejects_non_utf8_boundaries_and_out_of_bounds_offsets() {
    let mut sources = SourceMap::new();
    let id = sources.add_source("unicode.ln", "é");
    let file = sources.get(id).expect("the source should exist");

    assert_eq!(file.line_column(1), None);
    assert_eq!(file.line_column(3), None);
}

#[test]
fn spec_6_2_renderer_distinguishes_duplicate_names_and_equal_offsets_by_source_id() {
    let mut sources = SourceMap::new();
    let at = sources.add_source("duplicate.ln", "@");
    let hash = sources.add_source("duplicate.ln", "#");
    let at_span = SourceSpan::new(at, 0, 1).expect("the first span is valid");
    let hash_span = SourceSpan::new(hash, 0, 1).expect("the second span is valid");

    assert_eq!(at_span.start(), hash_span.start());
    assert_eq!(at_span.end(), hash_span.end());
    assert_ne!(at_span.source(), hash_span.source());

    let at_rendered = render_diagnostic(&sources, &diagnostic(at_span))
        .expect("the first source should resolve by ID");
    let hash_rendered = render_diagnostic(&sources, &diagnostic(hash_span))
        .expect("the second source should resolve by ID");

    assert!(at_rendered.contains("duplicate.ln:1:1"));
    assert!(at_rendered.contains("1 | @"));
    assert!(!at_rendered.contains("1 | #"));
    assert!(hash_rendered.contains("duplicate.ln:1:1"));
    assert!(hash_rendered.contains("1 | #"));
    assert!(!hash_rendered.contains("1 | @"));
}

#[test]
fn spec_6_2_renderer_rejects_unknown_sources_and_invalid_offsets() {
    let mut sources = SourceMap::new();
    let source = sources.add_source("unicode.ln", "é");
    let unknown = unknown_source_id();
    let invalid_spans = [
        SourceSpan::empty(unknown, 0),
        SourceSpan::new(source, 0, 3).expect("ordered offsets can exceed the source"),
        SourceSpan::empty(source, 3),
        SourceSpan::new(source, 1, 2).expect("ordered offsets can split a code point"),
        SourceSpan::new(source, 0, 1).expect("ordered offsets can split a code point"),
    ];

    for span in invalid_spans {
        render_diagnostic(&sources, &diagnostic(span))
            .expect_err("invalid source/span relationships must be typed errors");
    }
}

#[test]
fn spec_6_2_renderer_validates_secondary_labels_without_mixing_sources() {
    let mut sources = SourceMap::new();
    let primary_source = sources.add_source("primary.ln", "@");
    let secondary_source = sources.add_source("secondary.ln", "#");
    let primary = SourceSpan::new(primary_source, 0, 1).expect("the primary span is valid");
    let secondary = SourceSpan::new(secondary_source, 0, 1).expect("the secondary span is valid");
    let multi_source = diagnostic(primary)
        .with_secondary(DiagnosticLabel::new(secondary, "related source location"));

    assert_eq!(multi_source.primary().span().source(), primary_source);
    assert_eq!(
        multi_source.secondary()[0].span().source(),
        secondary_source
    );
    let rendered = render_diagnostic(&sources, &multi_source)
        .expect("both source-aware labels should resolve");
    assert!(rendered.contains("primary.ln:1:1"));
    assert!(rendered.contains("1 | @"));

    let invalid = diagnostic(primary).with_secondary(DiagnosticLabel::new(
        SourceSpan::empty(unknown_source_id(), 0),
        "unknown related source",
    ));
    render_diagnostic(&sources, &invalid)
        .expect_err("an unknown secondary SourceId must not be ignored");
}

#[test]
fn spec_5_8_simple_check_outcome_owns_source_map_and_diagnostic() {
    let source = "let out = @; render 1s;";
    let outcome = check(source);
    let diagnostic = outcome
        .result
        .as_ref()
        .expect_err("the unknown character should be rejected");
    let span = diagnostic.primary().span();

    assert_eq!(diagnostic.code(), ErrorCode::UnknownCharacter);
    assert_eq!((span.start(), span.end()), (10, 11));
    let file = outcome
        .sources
        .get(span.source())
        .expect("the outcome should retain the primary source");
    assert_eq!(file.logical_name(), "input.ln");
    assert_eq!(file.text(), source);

    let rendered = render_diagnostic(&outcome.sources, diagnostic)
        .expect("the retained SourceMap should resolve the diagnostic");
    assert!(rendered.contains("input.ln:1:11"));
}

#[test]
fn spec_5_8_simple_check_outcome_owns_source_map_and_ast() {
    let source = "let out = 1; render 1s;";
    let outcome = check(source);
    let program = outcome
        .result
        .as_ref()
        .expect("the valid source should parse");
    let span = program.span;

    assert_eq!((span.start(), span.end()), (0, source.len()));
    let file = outcome
        .sources
        .get(span.source())
        .expect("the outcome should retain the AST source");
    assert_eq!(file.logical_name(), "input.ln");
    assert_eq!(file.text(), source);
}

#[test]
fn spec_5_8_source_aware_check_uses_explicit_name_and_compile_config() {
    let config = CompileConfig {
        parser: ParserConfig {
            max_depth: 128,
            max_nodes: 2,
        },
        ..CompileConfig::default()
    };
    let outcome = check_with_config("configured.ln", "let out = 1; render 1s;", config);
    let diagnostic = outcome
        .result
        .as_ref()
        .expect_err("the third AST node should exceed the configured limit");
    let span = diagnostic.primary().span();

    assert_eq!(diagnostic.code(), ErrorCode::AstNodeLimit);
    assert_eq!((span.start(), span.end()), (13, 19));
    assert_eq!(
        outcome
            .sources
            .get(span.source())
            .expect("the configured source should exist")
            .logical_name(),
        "configured.ln"
    );
    let rendered = assert_rendered_error(&outcome);
    assert!(rendered.contains("configured.ln:1:14"));
}

#[test]
fn spec_5_8_session_checks_added_sources_and_rejects_unknown_ids_at_the_boundary() {
    let mut session = CompileSession::new(CompileConfig::default());
    let source = session
        .sources_mut()
        .add_source("session.ln", "let out = 1; render 1s;");
    let program = session
        .check(source)
        .expect("the SourceId belongs to this session")
        .expect("the source should parse");

    assert_eq!(program.span.source(), source);
    assert_eq!(
        session
            .sources()
            .get(source)
            .expect("the session should retain its source")
            .logical_name(),
        "session.ln"
    );

    session
        .check(unknown_source_id())
        .expect_err("an unknown SourceId should be a typed session error");
}

#[test]
fn spec_5_11_repeated_check_is_structurally_deterministic_within_one_process() {
    for source in ["let out = 1; render 1s;", "let out = @; render 1s;"] {
        let first = check(source);
        let second = check(source);

        assert_eq!(first.result, second.result);

        if first.result.is_err() {
            assert_eq!(
                assert_rendered_error(&first),
                assert_rendered_error(&second)
            );
        }
    }
}
