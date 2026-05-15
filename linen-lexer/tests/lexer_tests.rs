use linen_lexer::{lex, Token};

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
fn lex_comments_are_skipped() {
    let tokens = lex("let x = 5 -- this is a comment\nlet y = 6").unwrap();
    assert_eq!(tokens.len(), 8); // let x = 5 let y = 6
}
