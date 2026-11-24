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
    Multichannel,
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

const NO_RESOURCES: ResourceModel = ResourceModel {
    scalar_registers: 0,
    audio_registers: 0,
    state_slots: 0,
    state_bytes: 0,
};

const PHASE_RESOURCES: ResourceModel = ResourceModel {
    scalar_registers: 0,
    audio_registers: 1,
    state_slots: 1,
    state_bytes: size_of::<f64>(),
};

const STEREO_RESOURCES: ResourceModel = ResourceModel {
    scalar_registers: 0,
    audio_registers: 2,
    state_slots: 0,
    state_bytes: 0,
};

const fn phase_builtin(id: BuiltinId, name: &'static str) -> BuiltinDescriptor {
    BuiltinDescriptor {
        id,
        name,
        signature: BuiltinSignature {
            parameters: &[Type::Sig],
            result: Type::Sig,
        },
        purity: Purity::NoSideEffects,
        determinism: Determinism::Deterministic,
        statefulness: Statefulness::Stateful,
        capability: RuntimeCapability::CoreAudio,
        resources: PHASE_RESOURCES,
        documentation_name: name,
    }
}

pub const BUILTINS: [BuiltinDescriptor; 12] = [
    phase_builtin(BuiltinId::SINE, "sine"),
    phase_builtin(BuiltinId::PHASOR, "phasor"),
    phase_builtin(BuiltinId::SAW, "saw"),
    phase_builtin(BuiltinId::SQUARE, "square"),
    phase_builtin(BuiltinId::TRIANGLE, "triangle"),
    BuiltinDescriptor {
        id: BuiltinId::GAIN,
        name: "gain",
        signature: BuiltinSignature {
            parameters: &[Type::Sig, Type::Sig],
            result: Type::Sig,
        },
        purity: Purity::NoSideEffects,
        determinism: Determinism::Deterministic,
        statefulness: Statefulness::Stateless,
        capability: RuntimeCapability::CoreAudio,
        resources: NO_RESOURCES,
        documentation_name: "gain",
    },
    BuiltinDescriptor {
        id: BuiltinId::MIX,
        name: "mix",
        signature: BuiltinSignature {
            parameters: &[Type::Sig, Type::Sig, Type::Sig],
            result: Type::Sig,
        },
        purity: Purity::NoSideEffects,
        determinism: Determinism::Deterministic,
        statefulness: Statefulness::Stateless,
        capability: RuntimeCapability::CoreAudio,
        resources: NO_RESOURCES,
        documentation_name: "mix",
    },
    BuiltinDescriptor {
        id: BuiltinId::STEREO,
        name: "stereo",
        signature: BuiltinSignature {
            parameters: &[Type::Sig, Type::Sig],
            result: Type::Stereo,
        },
        purity: Purity::NoSideEffects,
        determinism: Determinism::Deterministic,
        statefulness: Statefulness::Stateless,
        capability: RuntimeCapability::Multichannel,
        resources: STEREO_RESOURCES,
        documentation_name: "stereo",
    },
    BuiltinDescriptor {
        id: BuiltinId::PAN,
        name: "pan",
        signature: BuiltinSignature {
            parameters: &[Type::Sig, Type::Sig],
            result: Type::Stereo,
        },
        purity: Purity::NoSideEffects,
        determinism: Determinism::Deterministic,
        statefulness: Statefulness::Stateless,
        capability: RuntimeCapability::Multichannel,
        resources: STEREO_RESOURCES,
        documentation_name: "pan",
    },
    BuiltinDescriptor {
        id: BuiltinId::NOISE,
        name: "noise",
        signature: BuiltinSignature {
            parameters: &[Type::Float],
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
            state_bytes: 2 * size_of::<u64>(),
        },
        documentation_name: "noise",
    },
    phase_builtin(BuiltinId::SAW_POLYBLEP, "saw_polyblep"),
    phase_builtin(BuiltinId::SQUARE_POLYBLEP, "square_polyblep"),
];

pub fn builtin_descriptor(id: BuiltinId) -> Option<&'static BuiltinDescriptor> {
    BUILTINS.get(id.index())
}

pub fn builtin_by_name(name: &str) -> Option<&'static BuiltinDescriptor> {
    BUILTINS.iter().find(|descriptor| descriptor.name == name)
}
