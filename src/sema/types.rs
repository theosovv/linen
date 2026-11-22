use crate::{
    source::SourceSpan,
    syntax::{BinaryOp, DurationLiteral, Identifier, TypeAnnotation, UnaryOp},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Type {
    Float,
    Sig,
    Stereo,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
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
pub struct FunctionId(usize);

impl FunctionId {
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
    pub const PHASOR: Self = Self(1);
    pub const SAW: Self = Self(2);
    pub const SQUARE: Self = Self(3);
    pub const TRIANGLE: Self = Self(4);
    pub const GAIN: Self = Self(5);
    pub const MIX: Self = Self(6);
    pub const STEREO: Self = Self(7);
    pub const PAN: Self = Self(8);
    pub const NOISE: Self = Self(9);

    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FunctionLimits {
    pub max_call_depth: usize,
    pub max_expanded_nodes: usize,
}

impl Default for FunctionLimits {
    fn default() -> Self {
        Self {
            max_call_depth: 128,
            max_expanded_nodes: 10_000,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypedProgram {
    pub declarations: Vec<TypedTopDecl>,
    pub output: BindingId,
    pub render: TypedRender,
    pub span: SourceSpan,
}

impl TypedProgram {
    pub fn bindings(&self) -> Vec<&TypedBinding> {
        self.declarations
            .iter()
            .filter_map(|declaration| {
                let TypedTopDecl::Binding(binding) = declaration else {
                    return None;
                };
                Some(binding)
            })
            .collect()
    }

    pub fn functions(&self) -> impl Iterator<Item = &TypedFunction> {
        self.declarations.iter().filter_map(|declaration| {
            let TypedTopDecl::Function(function) = declaration else {
                return None;
            };
            Some(function)
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypedTopDecl {
    Binding(TypedBinding),
    Function(TypedFunction),
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
pub struct TypedFunction {
    pub id: FunctionId,
    pub name: Identifier,
    pub parameters: Vec<TypedParameter>,
    pub result: TypeAnnotation,
    pub body: TypedExpr,
    pub span: SourceSpan,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TypedParameter {
    pub id: BindingId,
    pub name: Identifier,
    pub annotation: TypeAnnotation,
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
    BuiltinCall {
        builtin: BuiltinId,
        callee: Identifier,
        arguments: Vec<TypedExpr>,
    },
    FunctionCall {
        function: FunctionId,
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
    Block {
        bindings: Vec<TypedBinding>,
        result: Box<TypedExpr>,
    },
    LiftFloatToSig(Box<TypedExpr>),
}
