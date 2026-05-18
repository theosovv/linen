//! Abstract Syntax Tree definitions for Linen.

use crate::span::Span;

/// A complete module (source file).
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub imports: Vec<Import>,
    pub declarations: Vec<Declaration>,
}

/// An import statement: `import "path" as Alias`.
#[derive(Debug, Clone, PartialEq)]
pub struct Import {
    pub path: String,
    pub alias: String,
    pub span: Span,
}

/// Top-level declaration.
#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    Let(LetDecl),
    Fn(FnDecl),
    Type(TypeDecl),
    Export(ExportDecl),
}

/// An export declaration: `export expr "filename" duration`.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportDecl {
    pub expr: Expr,
    pub filename: String,
    pub duration: Expr,
    pub span: Span,
}

/// A let binding: `let name : Type = expr`.
#[derive(Debug, Clone, PartialEq)]
pub struct LetDecl {
    pub name: String,
    pub ty: Option<Type>,
    pub value: Expr,
    pub span: Span,
}

/// A function declaration: `fn name params : Type -> expr`.
#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub ret_ty: Option<Type>,
    pub body: Expr,
    pub span: Span,
}

/// A type declaration: `type Name params = Variants`.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeDecl {
    pub name: String,
    pub params: Vec<String>,
    pub variants: Vec<Variant>,
    pub span: Span,
}

/// Function parameter.
#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: Option<Type>,
}

/// Variant in an algebraic data type.
#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub name: String,
    pub fields: Vec<Type>,
}

/// Type annotation AST node.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Named { name: String, args: Vec<Type> },
    Arrow(Box<Type>, Box<Type>),
    Tuple(Vec<Type>),
    Record(Vec<(String, Type)>),
    Var(String),
}

/// Binary operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    And,
    Or,
}

/// Expression AST node.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(Literal, Span),
    Ident(String, Span),
    Pipeline(Box<Expr>, Box<Expr>, Span),
    Apply(Box<Expr>, Vec<Expr>, Span),
    Lambda(Vec<Param>, Box<Expr>, Span),
    Let(String, Option<Type>, Box<Expr>, Box<Expr>, Span),
    Match(Box<Expr>, Vec<Arm>, Span),
    If(Box<Expr>, Box<Expr>, Box<Expr>, Span),
    Record(Vec<(String, Expr)>, Span),
    List(Vec<Expr>, Span),
    FieldAccess(Box<Expr>, String, Span),
    Tuple(Vec<Expr>, Span),
    Binary(BinOp, Box<Expr>, Box<Expr>, Span),
}

impl Expr {
    pub fn get_span(&self) -> Span {
        match self {
            Expr::Literal(_, span)
            | Expr::Ident(_, span)
            | Expr::Pipeline(_, _, span)
            | Expr::Apply(_, _, span)
            | Expr::Lambda(_, _, span)
            | Expr::Let(_, _, _, _, span)
            | Expr::Match(_, _, span)
            | Expr::If(_, _, _, span)
            | Expr::Record(_, span)
            | Expr::List(_, span)
            | Expr::FieldAccess(_, _, span)
            | Expr::Tuple(_, span)
            | Expr::Binary(_, _, _, span) => *span,
        }
    }
}

/// Pattern match arm: `| Pattern -> Expr`.
#[derive(Debug, Clone, PartialEq)]
pub struct Arm {
    pub pattern: Pattern,
    pub expr: Expr,
}

/// Pattern AST node.
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Wildcard(Span),
    Literal(Literal, Span),
    Ident(String, Span),
    Record(Vec<(String, Pattern)>, Span),
    Variant(String, Vec<Pattern>, Span),
    Tuple(Vec<Pattern>, Span),
}

/// Literal values.
#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    Unit,
}
