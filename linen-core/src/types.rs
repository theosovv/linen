//! Type system definitions for Linen.
//!
//! Supports Hindley-Milner type inference with let-polymorphism,
//! phantom types for audio units, and FRP primitives.

/// A type variable ID (unique per inference session).
pub type TypeVar = u64;

/// Type constructor names.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TypeCon {
    Int,
    Float,
    Bool,
    String,
    Unit,
    Time,
    // Audio phantom types (runtime: Float)
    Hz,
    Seconds,
    Samples,
    Amplitude,
    Bpm,
    // FRP types
    Signal,
    Event,
    Behavior,
    // Composite
    Arrow,
    Tuple(usize),
    Record(Vec<String>),
    // User-defined
    Named(String),
}

/// Types in the Linen type system.
#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    /// Type variable (unification variable)
    Var(TypeVar),
    /// Concrete type constructor with arguments
    Con(TypeCon, Vec<Type>),
    /// Function arrow: A -> B
    Arrow(Box<Type>, Box<Type>),
    /// Tuple: (A, B, ...)
    Tuple(Vec<Type>),
    /// Record: { field: T, ... }
    Record(Vec<(String, Type)>),
}

/// A type scheme: polymorphic type (forall vars. type).
#[derive(Debug, Clone, PartialEq)]
pub struct Scheme {
    pub vars: Vec<TypeVar>,
    pub ty: Type,
}

impl Scheme {
    pub fn monomorphic(ty: Type) -> Self {
        Self { vars: vec![], ty }
    }
}

/// Type environment mapping names to schemes.
#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    pub bindings: std::collections::HashMap<String, Scheme>,
}

impl TypeEnv {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn extend(&mut self, name: String, scheme: Scheme) {
        self.bindings.insert(name, scheme);
    }

    pub fn lookup(&self, name: &str) -> Option<&Scheme> {
        self.bindings.get(name)
    }
}
