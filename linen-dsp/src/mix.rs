//! Mixing and gain utilities for the Linen DSP runtime.

use crate::traits::SignalNode;

/// Amplitude scaling node.
pub struct Gain {
    input: Box<dyn SignalNode>,
    gain: f32,
}

impl Gain {
    pub fn new(input: Box<dyn SignalNode>, gain: f32) -> Self {
        Self { input, gain }
    }
}

impl SignalNode for Gain {
    #[inline]
    fn next_sample(&mut self, t: u64, sr: f32) -> f32 {
        self.input.next_sample(t, sr) * self.gain
    }
}

/// Sum multiple signals into one.
pub struct Mix {
    inputs: Vec<Box<dyn SignalNode>>,
}

impl Mix {
    pub fn new(inputs: Vec<Box<dyn SignalNode>>) -> Self {
        Self { inputs }
    }
}

impl SignalNode for Mix {
    #[inline]
    fn next_sample(&mut self, t: u64, sr: f32) -> f32 {
        let mut sum = 0.0;
        for input in &mut self.inputs {
            sum += input.next_sample(t, sr);
        }
        sum
    }
}
