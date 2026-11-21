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
pub enum RuntimeCapability {
    CoreAudio,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ResourceModel {
    pub scalar_registers: usize,
    pub audio_registers: usize,
    pub state_slots: usize,
    pub state_bytes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BuiltinDescriptor {
    pub id: BuiltinId,
    pub name: &'static str,
    pub signature: BuiltinSignature,
    pub purity: Purity,
    pub determinism: Determinism,
    pub statefulness: Statefulness,
    pub capability: RuntimeCapability,
    pub resources: ResourceModel,
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
    capability: RuntimeCapability::CoreAudio,
    resources: ResourceModel {
        scalar_registers: 0,
        audio_registers: 1,
        state_slots: 1,
        state_bytes: size_of::<f64>(),
    },
    documentation_name: "sine",
}];

pub fn builtin_descriptor(id: BuiltinId) -> Option<&'static BuiltinDescriptor> {
    BUILTINS.get(id.index())
}

pub fn builtin_by_name(name: &str) -> Option<&'static BuiltinDescriptor> {
    BUILTINS.iter().find(|descriptor| descriptor.name == name)
}
