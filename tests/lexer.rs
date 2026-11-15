use std::panic::catch_unwind;

use linen::syntax::{DurationUnit, Token, TokenKind, lex};
use linen::{ErrorCode, Span};

fn span(start: usize, end: usize) -> Span {
    Span::new(start, end).expect("the test span is valid")
}

fn assert_tokens(source: &str, expected: &[(TokenKind, (usize, usize))]) {
    let actual = lex(source).expect("the source should lex");
    let mut expected: Vec<_> = expected
        .iter()
        .map(|(kind, (start, end))| Token::new(kind.clone(), span(*start, *end)))
        .collect();
    expected.push(Token::new(TokenKind::Eof, Span::empty(source.len())));

    assert_eq!(actual, expected);
}

fn assert_lex_error(source: &str, code: ErrorCode, start: usize, end: usize) {
    let diagnostic = lex(source).expect_err("the source should not lex");

    assert_eq!(diagnostic.code(), code);
    assert_eq!(diagnostic.primary().span(), span(start, end));
}

#[test]
fn spec_4_3_empty_whitespace_and_comments_leave_only_eof() {
    for source in ["", " \t\r\n", "// comment", "// comment\r\n// final"] {
        assert_tokens(source, &[]);
    }
}

#[test]
fn spec_4_3_numbers_follow_the_grammar_and_minus_is_separate() {
    assert_tokens(
        "0 42 0.5 42.0 1e3 1.2e-3 -42",
        &[
            (TokenKind::Number(0.0), (0, 1)),
            (TokenKind::Number(42.0), (2, 4)),
            (TokenKind::Number(0.5), (5, 8)),
            (TokenKind::Number(42.0), (9, 13)),
            (TokenKind::Number(1000.0), (14, 17)),
            (TokenKind::Number(0.0012), (18, 24)),
            (TokenKind::Minus, (25, 26)),
            (TokenKind::Number(42.0), (26, 28)),
        ],
    );
}

#[test]
fn spec_4_3_durations_keep_their_unit_and_byte_span() {
    assert_tokens(
        "1ms 2s 0.5s",
        &[
            (
                TokenKind::Duration {
                    value: 1.0,
                    unit: DurationUnit::Milliseconds,
                },
                (0, 3),
            ),
            (
                TokenKind::Duration {
                    value: 2.0,
                    unit: DurationUnit::Seconds,
                },
                (4, 6),
            ),
            (
                TokenKind::Duration {
                    value: 0.5,
                    unit: DurationUnit::Seconds,
                },
                (7, 11),
            ),
        ],
    );
}

#[test]
fn spec_4_3_keywords_are_reserved_and_out_is_an_identifier() {
    assert_tokens(
        "let render Float Sig true false type match mod use feedback event mut where out Out _x x1",
        &[
            (TokenKind::Let, (0, 3)),
            (TokenKind::Render, (4, 10)),
            (TokenKind::Float, (11, 16)),
            (TokenKind::Sig, (17, 20)),
            (TokenKind::True, (21, 25)),
            (TokenKind::False, (26, 31)),
            (TokenKind::Type, (32, 36)),
            (TokenKind::Match, (37, 42)),
            (TokenKind::Mod, (43, 46)),
            (TokenKind::Use, (47, 50)),
            (TokenKind::Feedback, (51, 59)),
            (TokenKind::Event, (60, 65)),
            (TokenKind::Mut, (66, 69)),
            (TokenKind::Where, (70, 75)),
            (TokenKind::Identifier("out".into()), (76, 79)),
            (TokenKind::Identifier("Out".into()), (80, 83)),
            (TokenKind::Identifier("_x".into()), (84, 86)),
            (TokenKind::Identifier("x1".into()), (87, 89)),
        ],
    );
}

#[test]
fn spec_4_3_punctuation_has_independent_tokens() {
    assert_tokens(
        "+ - * / ( ) , : = ;",
        &[
            (TokenKind::Plus, (0, 1)),
            (TokenKind::Minus, (2, 3)),
            (TokenKind::Star, (4, 5)),
            (TokenKind::Slash, (6, 7)),
            (TokenKind::LeftParen, (8, 9)),
            (TokenKind::RightParen, (10, 11)),
            (TokenKind::Comma, (12, 13)),
            (TokenKind::Colon, (14, 15)),
            (TokenKind::Equal, (16, 17)),
            (TokenKind::Semicolon, (18, 19)),
        ],
    );
}

#[test]
fn spec_4_3_crlf_and_comments_preserve_byte_offsets() {
    let source = "let x = 1; // comment\r\n// final";

    assert_tokens(
        source,
        &[
            (TokenKind::Let, (0, 3)),
            (TokenKind::Identifier("x".into()), (4, 5)),
            (TokenKind::Equal, (6, 7)),
            (TokenKind::Number(1.0), (8, 9)),
            (TokenKind::Semicolon, (9, 10)),
        ],
    );
    assert_eq!(source.len(), 31);
}

#[test]
fn spec_4_3_malformed_numbers_and_suffixes_are_e1001() {
    for source in [
        "1.", "1e", "1e+", "1e-", "123abc", "1sfoo", "1m", "1MS", "1.2.3",
    ] {
        assert_lex_error(source, ErrorCode::InvalidNumberOrDuration, 0, source.len());
    }
}

#[test]
fn spec_4_3_nonfinite_f64_values_are_not_malformed_syntax() {
    assert_tokens("1e999", &[(TokenKind::Number(f64::INFINITY), (0, 5))]);
    assert_tokens(
        "1e999s",
        &[(
            TokenKind::Duration {
                value: f64::INFINITY,
                unit: DurationUnit::Seconds,
            },
            (0, 6),
        )],
    );
}

#[test]
fn spec_4_3_unknown_ascii_and_unicode_characters_are_e1000() {
    assert_lex_error("@", ErrorCode::UnknownCharacter, 0, 1);
    assert_lex_error(".5", ErrorCode::UnknownCharacter, 0, 1);
    assert_lex_error("let é", ErrorCode::UnknownCharacter, 4, 6);
    assert_lex_error("💡", ErrorCode::UnknownCharacter, 0, 4);
}

#[test]
fn spec_4_3_unknown_character_fixture_has_the_exact_primary_span() {
    let source = include_str!("../fixtures/invalid/stage-01-unknown-character.ln");
    assert_lex_error(source, ErrorCode::UnknownCharacter, 10, 11);
}

#[test]
fn spec_7_6_lexer_does_not_panic_on_deterministic_unicode_corpus() {
    const ALPHABET: &[char] = &[
        'a', '_', '0', '9', '.', 'e', 'E', '+', '-', 'm', 's', '/', '\r', '\n', '\t', ' ', 'é',
        '💡', '@', '(', ')', ';', ',', '\0',
    ];
    let mut state = 0x4c45_5845_5231_u64;

    for case_index in 0..64 {
        let mut source = String::new();
        for _ in 0..32 {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let index = ((state >> 32) % ALPHABET.len() as u64) as usize;
            source.push(ALPHABET[index]);
        }

        let result = catch_unwind(|| lex(&source));
        assert!(
            result.is_ok(),
            "lexer panicked for corpus case {case_index}"
        );
    }
}
