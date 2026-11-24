use crate::{
    SourceSpan,
    sema::{FunctionId, ResourceModel, RuntimeCapability},
};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct NodeId(usize);

impl NodeId {
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct RandomStreamId(u64);

impl RandomStreamId {
    pub const fn new(seed: u64) -> Self {
        Self(seed)
    }

    pub const fn seed(self) -> u64 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueType {
    Float,
    Sig,
    Stereo,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CallOrigin {
    pub function: FunctionId,
    pub call_span: SourceSpan,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NodeOrigin {
    pub primary: SourceSpan,
    pub inlined_from: Vec<CallOrigin>,
}

impl NodeOrigin {
    pub const fn new(primary: SourceSpan) -> Self {
        Self {
            primary,
            inlined_from: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub id: NodeId,
    pub op: NodeOp,
    pub result_type: ValueType,
    pub origin: NodeOrigin,
}

#[derive(Clone, Debug, PartialEq)]
pub enum NodeOp {
    ConstFloat(f64),
    LiftFloatToSig(NodeId),
    Neg(NodeId),
    Add(NodeId, NodeId),
    Sub(NodeId, NodeId),
    Mul(NodeId, NodeId),
    Div(NodeId, NodeId),
    Sine(NodeId),
    Phasor(NodeId),
    Saw(NodeId),
    Square(NodeId),
    SawPolyblep(NodeId),
    SquarePolyblep(NodeId),
    Triangle(NodeId),
    Stereo(NodeId, NodeId),
    Pan(NodeId, NodeId),
    Noise {
        seed: NodeId,
        stream: RandomStreamId,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NodeDescriptor {
    pub capability: RuntimeCapability,
    pub resources: ResourceModel,
}

impl NodeOp {
    pub fn descriptor(&self, result_type: ValueType) -> NodeDescriptor {
        let scalar = ResourceModel {
            scalar_registers: 1,
            audio_registers: 0,
            state_slots: 0,
            state_bytes: 0,
        };
        let audio = ResourceModel {
            scalar_registers: 0,
            audio_registers: 1,
            state_slots: 0,
            state_bytes: 0,
        };
        let stereo = ResourceModel {
            scalar_registers: 0,
            audio_registers: 2,
            state_slots: 0,
            state_bytes: 0,
        };
        let phase = ResourceModel {
            scalar_registers: 0,
            audio_registers: 1,
            state_slots: 1,
            state_bytes: size_of::<f64>(),
        };
        let resources = match self {
            Self::ConstFloat(_) => scalar,
            Self::LiftFloatToSig(_) => audio,
            Self::Neg(_) | Self::Add(..) | Self::Sub(..) | Self::Mul(..) | Self::Div(..) => {
                match result_type {
                    ValueType::Float => scalar,
                    ValueType::Sig => audio,
                    ValueType::Stereo => stereo,
                }
            }
            Self::Sine(_)
            | Self::Phasor(_)
            | Self::Saw(_)
            | Self::Square(_)
            | Self::SawPolyblep(_)
            | Self::SquarePolyblep(_)
            | Self::Triangle(_) => phase,
            Self::Stereo(..) | Self::Pan(..) => stereo,
            Self::Noise { .. } => ResourceModel {
                scalar_registers: 0,
                audio_registers: 1,
                state_slots: 1,
                state_bytes: 2 * size_of::<u64>(),
            },
        };
        let capability = match self {
            Self::Stereo(..) | Self::Pan(..) => RuntimeCapability::Multichannel,
            _ => RuntimeCapability::CoreAudio,
        };

        NodeDescriptor {
            capability,
            resources,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Graph {
    pub nodes: Vec<Node>,
    pub roots: Vec<NodeId>,
    pub output: NodeId,
    pub render_samples: u64,
}

pub(crate) fn const_float_value(nodes: &[Node], id: NodeId) -> Option<f64> {
    let node = nodes.get(id.index())?;
    if node.result_type != ValueType::Float {
        return None;
    }

    match &node.op {
        NodeOp::ConstFloat(value) => Some(*value),
        NodeOp::Neg(input) => Some(-const_float_value(nodes, *input)?),
        NodeOp::Add(left, right) => {
            Some(const_float_value(nodes, *left)? + const_float_value(nodes, *right)?)
        }
        NodeOp::Sub(left, right) => {
            Some(const_float_value(nodes, *left)? - const_float_value(nodes, *right)?)
        }
        NodeOp::Mul(left, right) => {
            Some(const_float_value(nodes, *left)? * const_float_value(nodes, *right)?)
        }
        NodeOp::Div(left, right) => {
            Some(const_float_value(nodes, *left)? / const_float_value(nodes, *right)?)
        }
        _ => None,
    }
}
