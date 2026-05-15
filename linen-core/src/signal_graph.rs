//! Signal Graph representation (DAG) for FRP lowering.

use crate::types::Type;

/// A node identifier in the signal graph.
pub type NodeId = usize;

/// A directed acyclic graph of signal processing nodes.
#[derive(Debug, Clone, PartialEq)]
pub struct SignalGraph {
    pub nodes: Vec<Node>,
    pub outputs: Vec<NodeId>,
}

/// A node in the signal graph.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Constant(f64),
    Oscillator(OscType, Vec<NodeId>),
    Filter(FilterType, Vec<NodeId>),
    Effect(EffectType, Vec<NodeId>),
    EventSource(EventType),
    Behavior(BehaviorType, Vec<NodeId>),
    Mix(Vec<NodeId>),
    Gain(Vec<NodeId>),
    Pan(Vec<NodeId>),
}

/// Oscillator types.
#[derive(Debug, Clone, PartialEq)]
pub enum OscType {
    Sine,
    Saw,
    Square,
    Triangle,
    NoiseWhite,
    NoisePink,
}

/// Filter types.
#[derive(Debug, Clone, PartialEq)]
pub enum FilterType {
    Lowpass,
    Highpass,
    Bandpass,
    Notch,
    Resonant,
}

/// Effect types.
#[derive(Debug, Clone, PartialEq)]
pub enum EffectType {
    Delay,
    Reverb,
    Chorus,
    Distortion,
    Compressor,
}

/// Event source types.
#[derive(Debug, Clone, PartialEq)]
pub enum EventType {
    Every,
    Cycle,
    Random,
    Midi,
}

/// Behavior types.
#[derive(Debug, Clone, PartialEq)]
pub enum BehaviorType {
    Hold,
    Switch,
}

impl SignalGraph {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            outputs: Vec::new(),
        }
    }

    pub fn add_node(&mut self, node: Node) -> NodeId {
        let id = self.nodes.len();
        self.nodes.push(node);
        id
    }

    pub fn set_outputs(&mut self, outputs: Vec<NodeId>) {
        self.outputs = outputs;
    }

    /// Check for cycles in the graph (excluding valid feedback through delay nodes).
    pub fn has_cycles(&self) -> bool {
        // TODO: implement DFS-based cycle detection
        false
    }
}

impl Default for SignalGraph {
    fn default() -> Self {
        Self::new()
    }
}
