use std::collections::BTreeSet;

use linen::{Diagnostic, DiagnosticLabel, ErrorCode, Severity, Span};

#[test]
fn span_is_a_half_open_byte_range() {
    let span = Span::new(4, 9).expect("the test span is valid");

    assert_eq!(span.start(), 4);
    assert_eq!(span.end(), 9);
    assert_eq!(span.len(), 5);
    assert!(!span.is_empty());

    let empty = Span::empty(7);
    assert_eq!(empty.start(), 7);
    assert_eq!(empty.end(), 7);
    assert_eq!(empty.len(), 0);
    assert!(empty.is_empty());
}

#[test]
fn span_rejects_reversed_offsets_without_panicking() {
    let error = Span::new(9, 4).expect_err("a reversed span must be rejected");

    assert_eq!(error.start(), 9);
    assert_eq!(error.end(), 4);
    assert_eq!(
        error.to_string(),
        "invalid source span: start 9 is greater than end 4"
    );
}

#[test]
fn diagnostic_exposes_its_complete_structure_without_cli() {
    let primary_span = Span::new(12, 15).expect("the primary span is valid");
    let secondary_span = Span::new(0, 5).expect("the secondary span is valid");
    let diagnostic = Diagnostic::new(
        Severity::Warning,
        ErrorCode::TypeMismatch,
        "incompatible types",
        DiagnosticLabel::new(primary_span, "expected Sig"),
    )
    .with_secondary(DiagnosticLabel::new(secondary_span, "defined here"))
    .with_note("Float can be lifted to Sig when Sig is expected");

    assert_eq!(diagnostic.severity(), Severity::Warning);
    assert_eq!(diagnostic.code(), ErrorCode::TypeMismatch);
    assert_eq!(diagnostic.message(), "incompatible types");
    assert_eq!(diagnostic.primary().span(), primary_span);
    assert_eq!(diagnostic.primary().message(), "expected Sig");
    assert_eq!(diagnostic.secondary().len(), 1);
    assert_eq!(diagnostic.secondary()[0].span(), secondary_span);
    assert_eq!(diagnostic.secondary()[0].message(), "defined here");
    assert_eq!(
        diagnostic.notes(),
        ["Float can be lifted to Sig when Sig is expected"]
    );
}

#[test]
fn error_constructor_sets_error_severity() {
    let span = Span::empty(0);
    let diagnostic = Diagnostic::error(
        ErrorCode::MissingOutput,
        "missing out binding",
        DiagnosticLabel::new(span, "add let out = ...;"),
    );

    assert_eq!(diagnostic.severity(), Severity::Error);
    assert!(diagnostic.secondary().is_empty());
    assert!(diagnostic.notes().is_empty());
}

#[test]
fn severity_labels_are_lowercase() {
    assert_eq!(Severity::Error.as_str(), "error");
    assert_eq!(Severity::Warning.to_string(), "warning");
    assert_eq!(Severity::Note.to_string(), "note");
}

#[test]
fn error_codes_match_the_specification_and_are_unique() {
    let expected = [
        (ErrorCode::UnknownCharacter, "E1000"),
        (ErrorCode::InvalidNumberOrDuration, "E1001"),
        (ErrorCode::UnexpectedToken, "E1100"),
        (ErrorCode::MissingSemicolon, "E1101"),
        (ErrorCode::DuplicateBinding, "E2000"),
        (ErrorCode::UnknownName, "E2001"),
        (ErrorCode::MissingOutput, "E2002"),
        (ErrorCode::MultipleOutputs, "E2003"),
        (ErrorCode::MissingOrMisplacedRender, "E2004"),
        (ErrorCode::TypeMismatch, "E2100"),
        (ErrorCode::InvalidArity, "E2101"),
        (ErrorCode::UnknownFunction, "E2102"),
        (ErrorCode::InvalidDuration, "E2103"),
        (ErrorCode::ConstantDivisionByZero, "E2104"),
        (ErrorCode::InvalidRenderConfiguration, "E4000"),
        (ErrorCode::NonFiniteOutput, "E4001"),
        (ErrorCode::RenderOverflow, "E4002"),
        (ErrorCode::FileIo, "E5000"),
    ];

    assert_eq!(ErrorCode::ALL.len(), expected.len());

    for (actual, (expected_variant, expected_code)) in ErrorCode::ALL.iter().zip(expected) {
        assert_eq!(actual, &expected_variant);
        assert_eq!(actual.as_str(), expected_code);
        assert_eq!(actual.to_string(), expected_code);
    }

    let unique_codes: BTreeSet<_> = ErrorCode::ALL.iter().map(|code| code.as_str()).collect();
    assert_eq!(unique_codes.len(), expected.len());
}
