use super::{BuiltinId, Type};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuiltinSignature {
    pub parameters: &'static [Type],
    pub result: Type,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Purity {
    NoSideEffects,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Determinism {
    Deterministic,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Statefulness {
    Stateless,
    Stateful,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuiltinDescriptor {
    pub id: BuiltinId,
    pub name: &'static str,
    pub signature: BuiltinSignature,
    pub purity: Purity,
    pub determinism: Determinism,
    pub statefulness: Statefulness,
    pub documentation_name: &'static str,
}

pub const BUILTINS: [BuiltinDescriptor; 1] = [BuiltinDescriptor {
    id: BuiltinId::SINE,
    name: "sine",
    signature: BuiltinSignature {
        parameters: &[Type::Sig],
        result: Type::Sig,
    },
    purity: Purity::NoSideEffects,
    determinism: Determinism::Deterministic,
    statefulness: Statefulness::Stateful,
    documentation_name: "sine",
}];

pub fn builtin_descriptor(id: BuiltinId) -> Option<&'static BuiltinDescriptor> {
    BUILTINS.get(id.index())
}

pub fn builtin_by_name(name: &str) -> Option<&'static BuiltinDescriptor> {
    BUILTINS.iter().find(|descriptor| descriptor.name == name)
}
