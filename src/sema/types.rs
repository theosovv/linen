use crate::{
    source::SourceSpan,
    syntax::{BinaryOp, DurationLiteral, Identifier, TypeAnnotation, UnaryOp},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Type {
    Float,
    Sig,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BindingId(usize);

impl BindingId {
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct BuiltinId(usize);

impl BuiltinId {
    pub const SINE: Self = Self(0);

    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypedProgram {
    pub bindings: Vec<TypedBinding>,
    pub output: BindingId,
    pub render: TypedRender,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypedBinding {
    pub id: BindingId,
    pub name: Identifier,
    pub annotation: Option<TypeAnnotation>,
    pub value: TypedExpr,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypedRender {
    pub duration: DurationLiteral,
    pub duration_seconds: f64,
    pub render_samples: u64,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypedExpr {
    pub kind: TypedExprKind,
    pub ty: Type,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypedExprKind {
    Number(f64),
    Identifier {
        id: BindingId,
        identifier: Identifier,
    },
    Call {
        builtin: BuiltinId,
        callee: Identifier,
        arguments: Vec<TypedExpr>,
    },
    Unary {
        operator: UnaryOp,
        operand: Box<TypedExpr>,
    },
    Binary {
        operator: BinaryOp,
        left: Box<TypedExpr>,
        right: Box<TypedExpr>,
    },
    Group(Box<TypedExpr>),
    LiftFloatToSig(Box<TypedExpr>),
}
