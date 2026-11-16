use crate::source::SourceSpan;

use super::DurationUnit;

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub bindings: Vec<LetDecl>,
    pub render: RenderDecl,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LetDecl {
    pub name: Identifier,
    pub annotation: Option<TypeAnnotation>,
    pub value: Expr,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderDecl {
    pub negative: bool,
    pub duration: DurationLiteral,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Identifier {
    pub name: String,
    pub span: SourceSpan,
}

impl Identifier {
    pub fn new(name: impl Into<String>, span: SourceSpan) -> Self {
        Self {
            name: name.into(),
            span,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypeAnnotation {
    pub kind: TypeName,
    pub span: SourceSpan,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TypeName {
    Float,
    Sig,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DurationLiteral {
    pub value: f64,
    pub unit: DurationUnit,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub span: SourceSpan,
}

impl Expr {
    pub fn new(kind: ExprKind, span: SourceSpan) -> Self {
        Self { kind, span }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Number(f64),
    Identifier(Identifier),
    Call {
        callee: Identifier,
        arguments: Vec<Expr>,
    },
    Unary {
        operator: UnaryOp,
        operand: Box<Expr>,
    },
    Binary {
        operator: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Group(Box<Expr>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnaryOp {
    Negate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}
