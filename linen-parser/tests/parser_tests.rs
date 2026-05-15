use linen_parser::parse;

#[test]
fn parse_hello_sine() {
    let source = r#"
        let tone = sin 440.0
        export tone "hello.wav" 2.0
    "#;
    let ast = parse(source).unwrap();
    assert_eq!(ast.declarations.len(), 2);
}

#[test]
fn parse_pipeline() {
    let source = r#"let drone = sin 220.0 |> gain 0.5"#;
    let ast = parse(source).unwrap();
    assert_eq!(ast.declarations.len(), 1);
}
