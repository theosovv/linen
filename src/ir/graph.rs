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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ValueType {
    Float,
    Sig,
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
        let resources = match self {
            Self::ConstFloat(_) => scalar,
            Self::LiftFloatToSig(_) => audio,
            Self::Neg(_) | Self::Add(..) | Self::Sub(..) | Self::Mul(..) | Self::Div(..) => {
                match result_type {
                    ValueType::Float => scalar,
                    ValueType::Sig => audio,
                }
            }
            Self::Sine(_) => ResourceModel {
                scalar_registers: 0,
                audio_registers: 1,
                state_slots: 1,
                state_bytes: size_of::<f64>(),
            },
        };

        NodeDescriptor {
            capability: RuntimeCapability::CoreAudio,
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
