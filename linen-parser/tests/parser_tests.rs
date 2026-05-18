use linen_core::ast::*;
use linen_parser::parse;

// ---------- Basic declarations ----------

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
    let source = "let drone = sin 220.0 |> gain 0.5";
    let ast = parse(source).unwrap();
    assert_eq!(ast.declarations.len(), 1);
    let let_decl = match &ast.declarations[0] {
        Declaration::Let(l) => l,
        _ => panic!("Expected let declaration"),
    };
    assert!(matches!(&let_decl.value, Expr::Pipeline(_, _, _)));
}

#[test]
fn parse_function_declaration() {
    let source = "fn add x y : Int -> Int -> x + y";
    let ast = parse(source).unwrap();
    let fn_decl = match &ast.declarations[0] {
        Declaration::Fn(f) => f,
        _ => panic!("Expected fn declaration"),
    };
    assert_eq!(fn_decl.name, "add");
    assert_eq!(fn_decl.params.len(), 2);
    assert_eq!(fn_decl.params[0].name, "x");
    assert_eq!(fn_decl.params[1].name, "y");
}

#[test]
fn parse_type_declaration() {
    let source = "type Option a = None | Some a";
    let ast = parse(source).unwrap();
    let type_decl = match &ast.declarations[0] {
        Declaration::Type(t) => t,
        _ => panic!("Expected type declaration"),
    };
    assert_eq!(type_decl.name, "Option");
    assert_eq!(type_decl.variants.len(), 2);
    assert_eq!(type_decl.variants[0].name, "None");
    assert_eq!(type_decl.variants[1].name, "Some");
}

#[test]
fn parse_import() {
    let source = r#"import "std/osc" as Osc"#;
    let ast = parse(source).unwrap();
    assert_eq!(ast.imports.len(), 1);
    assert_eq!(ast.imports[0].path, "std/osc");
    assert_eq!(ast.imports[0].alias, "Osc");
}

// ---------- Binary operators & precedence ----------

#[test]
fn parse_addition() {
    let source = "let x = 1 + 2";
    let ast = parse(source).unwrap();
    let let_decl = match &ast.declarations[0] {
        Declaration::Let(l) => l,
        _ => panic!(),
    };
    assert!(matches!(&let_decl.value, Expr::Binary(BinOp::Add, _, _, _)));
}

#[test]
fn parse_multiplication() {
    let source = "let x = 3 * 4";
    let ast = parse(source).unwrap();
    let let_decl = match &ast.declarations[0] {
        Declaration::Let(l) => l,
        _ => panic!(),
    };
    assert!(matches!(&let_decl.value, Expr::Binary(BinOp::Mul, _, _, _)));
}

#[test]
fn parse_mixed_precedence() {
    let source = "let x = 1 + 2 * 3";
    let ast = parse(source).unwrap();
    let let_decl = match &ast.declarations[0] {
        Declaration::Let(l) => l,
        _ => panic!(),
    };
    // Should be: 1 + (2 * 3)
    let Expr::Binary(BinOp::Add, left, right, _) = &let_decl.value else {
        panic!("Expected Add, got {:?}", let_decl.value);
    };
    assert!(matches!(left.as_ref(), Expr::Literal(Literal::Int(1), _)));
    assert!(matches!(right.as_ref(), Expr::Binary(BinOp::Mul, _, _, _)));
}

#[test]
fn parse_comparison() {
    let source = "let x = 1 < 2";
    let ast = parse(source).unwrap();
    let let_decl = match &ast.declarations[0] {
        Declaration::Let(l) => l,
        _ => panic!(),
    };
    assert!(matches!(&let_decl.value, Expr::Binary(BinOp::Lt, _, _, _)));
}

#[test]
fn parse_equality() {
    let source = "let x = 1 == 2";
    let ast = parse(source).unwrap();
    let let_decl = match &ast.declarations[0] {
        Declaration::Let(l) => l,
        _ => panic!(),
    };
    assert!(matches!(&let_decl.value, Expr::Binary(BinOp::Eq, _, _, _)));
}

#[test]
fn parse_boolean_ops() {
    let source = "let x = a && b || c";
    let ast = parse(source).unwrap();
    let let_decl = match &ast.declarations[0] {
        Declaration::Let(l) => l,
        _ => panic!(),
    };
    // Should be: (a && b) || c
    let Expr::Binary(BinOp::Or, left, right, _) = &let_decl.value else {
        panic!("Expected Or, got {:?}", let_decl.value);
    };
    assert!(matches!(right.as_ref(), Expr::Ident(name, _) if name == "c"));
    assert!(matches!(left.as_ref(), Expr::Binary(BinOp::And, _, _, _)));
}

#[test]
fn parse_left_associative() {
    let source = "let x = 1 - 2 - 3";
    let ast = parse(source).unwrap();
    let let_decl = match &ast.declarations[0] {
        Declaration::Let(l) => l,
        _ => panic!(),
    };
    // Should be: (1 - 2) - 3
    let Expr::Binary(BinOp::Sub, left, right, _) = &let_decl.value else {
        panic!("Expected Sub");
    };
    assert!(matches!(right.as_ref(), Expr::Literal(Literal::Int(3), _)));
    assert!(matches!(left.as_ref(), Expr::Binary(BinOp::Sub, _, _, _)));
}

#[test]
fn parse_pipeline_with_arithmetic() {
    let source = "let x = a + b |> f";
    let ast = parse(source).unwrap();
    let let_decl = match &ast.declarations[0] {
        Declaration::Let(l) => l,
        _ => panic!(),
    };
    // Should be: (a + b) |> f
    let Expr::Pipeline(left, right, _) = &let_decl.value else {
        panic!("Expected Pipeline");
    };
    assert!(matches!(left.as_ref(), Expr::Binary(BinOp::Add, _, _, _)));
    assert!(matches!(right.as_ref(), Expr::Ident(name, _) if name == "f"));
}

// ---------- Lambda ----------

#[test]
fn parse_lambda() {
    let source = r#"let f = \x -> x + 1"#;
    let ast = parse(source).unwrap();
    let let_decl = match &ast.declarations[0] {
        Declaration::Let(l) => l,
        _ => panic!(),
    };
    assert!(matches!(&let_decl.value, Expr::Lambda(_, _, _)));
}

#[test]
fn parse_lambda_multi_param() {
    let source = r#"let add = \x y -> x + y"#;
    let ast = parse(source).unwrap();
    let let_decl = match &ast.declarations[0] {
        Declaration::Let(l) => l,
        _ => panic!(),
    };
    let Expr::Lambda(params, _, _) = &let_decl.value else {
        panic!("Expected lambda");
    };
    assert_eq!(params.len(), 2);
}

// ---------- Match ----------

#[test]
fn parse_match_expression() {
    let source = r#"let r = match x with | A -> 1 | B -> 2"#;
    let ast = parse(source).unwrap();
    let let_decl = match &ast.declarations[0] {
        Declaration::Let(l) => l,
        _ => panic!(),
    };
    let Expr::Match(_, arms, _) = &let_decl.value else {
        panic!("Expected match");
    };
    assert_eq!(arms.len(), 2);
}

#[test]
fn parse_match_with_wildcard() {
    let source = r#"let r = match x with | _ -> 0"#;
    let ast = parse(source).unwrap();
    let let_decl = match &ast.declarations[0] {
        Declaration::Let(l) => l,
        _ => panic!(),
    };
    let Expr::Match(_, arms, _) = &let_decl.value else {
        panic!();
    };
    assert!(matches!(&arms[0].pattern, Pattern::Wildcard(_)));
}

// ---------- If ----------

#[test]
fn parse_if_expression() {
    let source = "let r = if true then 1 else 2";
    let ast = parse(source).unwrap();
    let let_decl = match &ast.declarations[0] {
        Declaration::Let(l) => l,
        _ => panic!(),
    };
    assert!(matches!(&let_decl.value, Expr::If(_, _, _, _)));
}

// ---------- Records & Lists ----------

#[test]
fn parse_record_literal() {
    let source = "let r = { x: 1, y: 2 }";
    let ast = parse(source).unwrap();
    let let_decl = match &ast.declarations[0] {
        Declaration::Let(l) => l,
        _ => panic!(),
    };
    let Expr::Record(fields, _) = &let_decl.value else {
        panic!("Expected record");
    };
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].0, "x");
    assert_eq!(fields[1].0, "y");
}

#[test]
fn parse_list_literal() {
    let source = "let xs = [1, 2, 3]";
    let ast = parse(source).unwrap();
    let let_decl = match &ast.declarations[0] {
        Declaration::Let(l) => l,
        _ => panic!(),
    };
    let Expr::List(elems, _) = &let_decl.value else {
        panic!("Expected list");
    };
    assert_eq!(elems.len(), 3);
}

#[test]
fn parse_empty_list() {
    let source = "let xs = []";
    let ast = parse(source).unwrap();
    let let_decl = match &ast.declarations[0] {
        Declaration::Let(l) => l,
        _ => panic!(),
    };
    let Expr::List(elems, _) = &let_decl.value else {
        panic!();
    };
    assert!(elems.is_empty());
}

#[test]
fn parse_tuple_literal() {
    let source = "let t = (1, 2)";
    let ast = parse(source).unwrap();
    let let_decl = match &ast.declarations[0] {
        Declaration::Let(l) => l,
        _ => panic!(),
    };
    let Expr::Tuple(elems, _) = &let_decl.value else {
        panic!("Expected tuple");
    };
    assert_eq!(elems.len(), 2);
}

#[test]
fn parse_unit_literal() {
    let source = "let u = ()";
    let ast = parse(source).unwrap();
    let let_decl = match &ast.declarations[0] {
        Declaration::Let(l) => l,
        _ => panic!(),
    };
    assert!(matches!(&let_decl.value, Expr::Literal(Literal::Unit, _)));
}

// ---------- Patterns ----------

#[test]
fn parse_variant_pattern() {
    let source = r#"let r = match x with | Some(a) -> a | None -> 0"#;
    let ast = parse(source).unwrap();
    let let_decl = match &ast.declarations[0] {
        Declaration::Let(l) => l,
        _ => panic!(),
    };
    let Expr::Match(_, arms, _) = &let_decl.value else {
        panic!()
    };
    assert!(matches!(&arms[0].pattern, Pattern::Variant(name, _, _) if name == "Some"));
    assert!(matches!(&arms[1].pattern, Pattern::Ident(name, _) if name == "None"));
}

#[test]
fn parse_record_pattern() {
    let source = r#"let r = match x with | {x, y} -> x + y"#;
    let ast = parse(source).unwrap();
    let let_decl = match &ast.declarations[0] {
        Declaration::Let(l) => l,
        _ => panic!(),
    };
    let Expr::Match(_, arms, _) = &let_decl.value else {
        panic!()
    };
    assert!(matches!(&arms[0].pattern, Pattern::Record(_, _)));
}

// ---------- Complex / Integration ----------

#[test]
fn parse_complex_program() {
    let source = r#"
        type Note = Rest | Tone { freq: Float, vel: Float }

        let playNote = fn note ->
          match note with
          | Rest -> silence
          | Tone { freq, vel } -> sin freq |> gain vel

        let track = playNote (Tone { freq: 440.0, vel: 0.5 })
        export track "out.wav" 2.0
    "#;
    let ast = parse(source).unwrap();
    assert_eq!(ast.declarations.len(), 3);
}

#[test]
fn parse_nested_application() {
    let source = "let x = f (g 1) 2";
    let ast = parse(source).unwrap();
    let let_decl = match &ast.declarations[0] {
        Declaration::Let(l) => l,
        _ => panic!(),
    };
    let Expr::Apply(func, args, _) = &let_decl.value else {
        panic!("Expected apply");
    };
    assert!(matches!(func.as_ref(), Expr::Ident(name, _) if name == "f"));
    assert_eq!(args.len(), 2);
}

// ---------- Error recovery ----------

#[test]
fn parse_error_recovery_skips_to_next_decl() {
    let source = r#"
        let x =
        let y = 42
    "#;
    // Missing RHS for x should not swallow y
    let ast = parse(source).unwrap();
    assert!(ast.declarations.len() >= 1);
}

// ---------- Spans ----------

#[test]
fn parse_spans_are_present() {
    let source = "let x = 42";
    let ast = parse(source).unwrap();
    let let_decl = match &ast.declarations[0] {
        Declaration::Let(l) => l,
        _ => panic!(),
    };
    assert!(let_decl.span.start < let_decl.span.end);
}
