use linen_lexer::{lex, Token};

// ---------- Keywords ----------

#[test]
fn lex_keyword_let() {
    let tokens = lex("let").unwrap();
    assert_eq!(tokens.len(), 1);
    assert_eq!(tokens[0].token, Token::Let);
}

#[test]
fn lex_keyword_fn() {
    let tokens = lex("fn").unwrap();
    assert_eq!(tokens[0].token, Token::Fn);
}

#[test]
fn lex_keyword_type() {
    let tokens = lex("type").unwrap();
    assert_eq!(tokens[0].token, Token::Type);
}

#[test]
fn lex_keyword_match() {
    let tokens = lex("match").unwrap();
    assert_eq!(tokens[0].token, Token::Match);
}

#[test]
fn lex_keyword_with() {
    let tokens = lex("with").unwrap();
    assert_eq!(tokens[0].token, Token::With);
}

#[test]
fn lex_keyword_import() {
    let tokens = lex("import").unwrap();
    assert_eq!(tokens[0].token, Token::Import);
}

#[test]
fn lex_keyword_export() {
    let tokens = lex("export").unwrap();
    assert_eq!(tokens[0].token, Token::Export);
}

#[test]
fn lex_keyword_as() {
    let tokens = lex("as").unwrap();
    assert_eq!(tokens[0].token, Token::As);
}

#[test]
fn lex_keyword_if() {
    let tokens = lex("if").unwrap();
    assert_eq!(tokens[0].token, Token::If);
}

#[test]
fn lex_keyword_then() {
    let tokens = lex("then").unwrap();
    assert_eq!(tokens[0].token, Token::Then);
}

#[test]
fn lex_keyword_else() {
    let tokens = lex("else").unwrap();
    assert_eq!(tokens[0].token, Token::Else);
}

#[test]
fn lex_keyword_true() {
    let tokens = lex("true").unwrap();
    assert_eq!(tokens[0].token, Token::True);
}

#[test]
fn lex_keyword_false() {
    let tokens = lex("false").unwrap();
    assert_eq!(tokens[0].token, Token::False);
}

// ---------- Literals ----------

#[test]
fn lex_int_positive() {
    let tokens = lex("42").unwrap();
    assert_eq!(tokens.len(), 1);
    assert!(matches!(tokens[0].token, Token::Int(42)));
}

#[test]
fn lex_int_negative() {
    let tokens = lex("-7").unwrap();
    assert_eq!(tokens.len(), 2);
    assert_eq!(tokens[0].token, Token::Minus);
    assert!(matches!(tokens[1].token, Token::Int(7)));
}

#[test]
fn lex_float_simple() {
    let tokens = lex("3.14").unwrap();
    assert!(matches!(tokens[0].token, Token::Float(x) if (x - 3.14).abs() < 1e-9));
}

#[test]
fn lex_float_exponent() {
    let tokens = lex("1.5e2").unwrap();
    assert!(matches!(tokens[0].token, Token::Float(x) if (x - 150.0).abs() < 1e-6));
}

#[test]
fn lex_float_negative_exponent() {
    let tokens = lex("2.0e-3").unwrap();
    assert!(matches!(tokens[0].token, Token::Float(x) if (x - 0.002).abs() < 1e-9));
}

#[test]
fn lex_string_hello() {
    let tokens = lex(r#""hello""#).unwrap();
    assert!(matches!(tokens[0].token, Token::String(ref s) if s == "hello"));
}

#[test]
fn lex_string_with_escapes() {
    let tokens = lex(r#""hello\nworld""#).unwrap();
    assert!(matches!(tokens[0].token, Token::String(ref s) if s == "hello\\nworld"));
}

// ---------- Operators ----------

#[test]
fn lex_operators() {
    let source = "+ - * / == != < > <= >= && || |> -> => .";
    let tokens = lex(source).unwrap();
    let expected = vec![
        Token::Plus,
        Token::Minus,
        Token::Star,
        Token::Slash,
        Token::Eq,
        Token::Neq,
        Token::Lt,
        Token::Gt,
        Token::Lte,
        Token::Gte,
        Token::And,
        Token::Or,
        Token::Pipeline,
        Token::Arrow,
        Token::FatArrow,
        Token::Dot,
    ];
    assert_eq!(tokens.len(), expected.len());
    for (t, e) in tokens.iter().zip(expected.iter()) {
        assert_eq!(&t.token, e);
    }
}

// ---------- Delimiters ----------

#[test]
fn lex_delimiters() {
    let source = "( ) [ ] { } , ; = : |";
    let tokens = lex(source).unwrap();
    let expected = vec![
        Token::LParen,
        Token::RParen,
        Token::LBracket,
        Token::RBracket,
        Token::LBrace,
        Token::RBrace,
        Token::Comma,
        Token::Semicolon,
        Token::Equals,
        Token::Colon,
        Token::Pipe,
    ];
    assert_eq!(tokens.len(), expected.len());
    for (t, e) in tokens.iter().zip(expected.iter()) {
        assert_eq!(&t.token, e);
    }
}

// ---------- Identifiers ----------

#[test]
fn lex_ident_lowercase() {
    let tokens = lex("foo").unwrap();
    assert!(matches!(tokens[0].token, Token::Ident(ref s) if s == "foo"));
}

#[test]
fn lex_ident_uppercase() {
    let tokens = lex("Bar").unwrap();
    assert!(matches!(tokens[0].token, Token::Ident(ref s) if s == "Bar"));
}

#[test]
fn lex_ident_with_underscore() {
    let tokens = lex("foo_bar").unwrap();
    assert!(matches!(tokens[0].token, Token::Ident(ref s) if s == "foo_bar"));
}

#[test]
fn lex_ident_underscore_only() {
    let tokens = lex("_").unwrap();
    assert!(matches!(tokens[0].token, Token::Ident(ref s) if s == "_"));
}

#[test]
fn lex_ident_with_numbers() {
    let tokens = lex("x1").unwrap();
    assert!(matches!(tokens[0].token, Token::Ident(ref s) if s == "x1"));
}

// ---------- Comments ----------

#[test]
fn lex_line_comment() {
    let tokens = lex("let x = 5 -- this is a comment\nlet y = 6").unwrap();
    assert_eq!(tokens.len(), 8);
    assert_eq!(tokens[0].token, Token::Let);
    assert!(matches!(tokens[1].token, Token::Ident(ref s) if s == "x"));
    assert_eq!(tokens[2].token, Token::Equals);
    assert!(matches!(tokens[3].token, Token::Int(5)));
    assert_eq!(tokens[4].token, Token::Let);
    assert!(matches!(tokens[5].token, Token::Ident(ref s) if s == "y"));
    assert_eq!(tokens[6].token, Token::Equals);
    assert!(matches!(tokens[7].token, Token::Int(6)));
}

#[test]
fn lex_block_comment() {
    let tokens = lex("let x = {- block comment -} 5").unwrap();
    assert_eq!(tokens.len(), 4);
    assert_eq!(tokens[0].token, Token::Let);
    assert!(matches!(tokens[1].token, Token::Ident(ref s) if s == "x"));
    assert_eq!(tokens[2].token, Token::Equals);
    assert!(matches!(tokens[3].token, Token::Int(5)));
}

#[test]
fn lex_multiline_block_comment() {
    let source = "let x = {-\n  multiline\n  comment\n-} 5";
    let tokens = lex(source).unwrap();
    assert_eq!(tokens.len(), 4);
    assert!(matches!(tokens[3].token, Token::Int(5)));
}

// ---------- Integration / Complex ----------

#[test]
fn lex_let_binding() {
    let tokens = lex("let x = 42").unwrap();
    assert_eq!(tokens[0].token, Token::Let);
    assert!(matches!(tokens[1].token, Token::Ident(ref s) if s == "x"));
    assert_eq!(tokens[2].token, Token::Equals);
    assert!(matches!(tokens[3].token, Token::Int(42)));
}

#[test]
fn lex_pipeline() {
    let tokens = lex("sin 440.0 |> gain 0.5").unwrap();
    assert!(matches!(tokens[0].token, Token::Ident(ref s) if s == "sin"));
    assert!(matches!(tokens[1].token, Token::Float(440.0)));
    assert_eq!(tokens[2].token, Token::Pipeline);
    assert!(matches!(tokens[3].token, Token::Ident(ref s) if s == "gain"));
    assert!(matches!(tokens[4].token, Token::Float(0.5)));
}

#[test]
fn lex_function_definition() {
    let source = "fn add x y : Int -> Int -> x + y";
    let tokens = lex(source).unwrap();
    assert_eq!(tokens[0].token, Token::Fn);
    assert!(matches!(tokens[1].token, Token::Ident(ref s) if s == "add"));
    assert!(matches!(tokens[2].token, Token::Ident(ref s) if s == "x"));
    assert!(matches!(tokens[3].token, Token::Ident(ref s) if s == "y"));
    assert_eq!(tokens[4].token, Token::Colon);
    assert!(matches!(tokens[5].token, Token::Ident(ref s) if s == "Int"));
    assert_eq!(tokens[6].token, Token::Arrow);
    assert!(matches!(tokens[7].token, Token::Ident(ref s) if s == "Int"));
    assert_eq!(tokens[8].token, Token::Arrow);
    assert!(matches!(tokens[9].token, Token::Ident(ref s) if s == "x"));
    assert_eq!(tokens[10].token, Token::Plus);
    assert!(matches!(tokens[11].token, Token::Ident(ref s) if s == "y"));
}

#[test]
fn lex_type_declaration() {
    let source = "type Option a = None | Some a";
    let tokens = lex(source).unwrap();
    assert_eq!(tokens[0].token, Token::Type);
    assert!(matches!(tokens[1].token, Token::Ident(ref s) if s == "Option"));
    assert!(matches!(tokens[2].token, Token::Ident(ref s) if s == "a"));
    assert_eq!(tokens[3].token, Token::Equals);
    assert!(matches!(tokens[4].token, Token::Ident(ref s) if s == "None"));
    assert_eq!(tokens[5].token, Token::Pipe);
    assert!(matches!(tokens[6].token, Token::Ident(ref s) if s == "Some"));
    assert!(matches!(tokens[7].token, Token::Ident(ref s) if s == "a"));
}

#[test]
fn lex_match_expression() {
    let source = "match x with | A -> 1 | B -> 2";
    let tokens = lex(source).unwrap();
    assert_eq!(tokens[0].token, Token::Match);
    assert!(matches!(tokens[1].token, Token::Ident(ref s) if s == "x"));
    assert_eq!(tokens[2].token, Token::With);
    assert_eq!(tokens[3].token, Token::Pipe);
    assert!(matches!(tokens[4].token, Token::Ident(ref s) if s == "A"));
    assert_eq!(tokens[5].token, Token::Arrow);
    assert!(matches!(tokens[6].token, Token::Int(1)));
    assert_eq!(tokens[7].token, Token::Pipe);
    assert!(matches!(tokens[8].token, Token::Ident(ref s) if s == "B"));
    assert_eq!(tokens[9].token, Token::Arrow);
    assert!(matches!(tokens[10].token, Token::Int(2)));
}

#[test]
fn lex_import_statement() {
    let source = r#"import "std/osc" as Osc"#;
    let tokens = lex(source).unwrap();
    assert_eq!(tokens[0].token, Token::Import);
    assert!(matches!(tokens[1].token, Token::String(ref s) if s == "std/osc"));
    assert_eq!(tokens[2].token, Token::As);
    assert!(matches!(tokens[3].token, Token::Ident(ref s) if s == "Osc"));
}

#[test]
fn lex_record_and_list() {
    let source = "{ x: 1, y: 2 } [1, 2, 3]";
    let tokens = lex(source).unwrap();
    assert_eq!(tokens[0].token, Token::LBrace);
    assert!(matches!(tokens[1].token, Token::Ident(ref s) if s == "x"));
    assert_eq!(tokens[2].token, Token::Colon);
    assert!(matches!(tokens[3].token, Token::Int(1)));
    assert_eq!(tokens[4].token, Token::Comma);
    assert!(matches!(tokens[5].token, Token::Ident(ref s) if s == "y"));
    assert_eq!(tokens[6].token, Token::Colon);
    assert!(matches!(tokens[7].token, Token::Int(2)));
    assert_eq!(tokens[8].token, Token::RBrace);
    assert_eq!(tokens[9].token, Token::LBracket);
    assert!(matches!(tokens[10].token, Token::Int(1)));
    assert_eq!(tokens[11].token, Token::Comma);
    assert!(matches!(tokens[12].token, Token::Int(2)));
    assert_eq!(tokens[13].token, Token::Comma);
    assert!(matches!(tokens[14].token, Token::Int(3)));
    assert_eq!(tokens[15].token, Token::RBracket);
}

// ---------- Error handling ----------

#[test]
fn lex_invalid_character() {
    let result = lex("@");
    assert!(result.is_err());
}

#[test]
fn lex_unterminated_string() {
    // logos regex won't match unterminated string, so it becomes an error
    let result = lex(r#""hello"#);
    assert!(result.is_err());
}

// ---------- Span tracking ----------

#[test]
fn lex_spans_are_correct() {
    let tokens = lex("let foo").unwrap();
    assert_eq!(tokens[0].span.start, 0);
    assert_eq!(tokens[0].span.end, 3); // "let"
    assert_eq!(tokens[1].span.start, 4);
    assert_eq!(tokens[1].span.end, 7); // "foo"
}
