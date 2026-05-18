//! Oscillator implementations for Linen DSP runtime.

use crate::traits::{ConstNode, SignalNode};
use std::f32::consts::PI;

/// Sine wave oscillator with phase accumulator.
///
/// Frequency can be a constant or driven by another signal node (FM synthesis).
pub struct SinOsc {
    freq: Box<dyn SignalNode>,
    phase: f32,
}

impl SinOsc {
    /// Create a sine oscillator with a constant frequency.
    pub fn new(freq: f32) -> Self {
        Self {
            freq: Box::new(ConstNode(freq)),
            phase: 0.0,
        }
    }

    /// Create a sine oscillator with a modulated frequency.
    pub fn with_modulator(freq: Box<dyn SignalNode>) -> Self {
        Self { freq, phase: 0.0 }
    }
}

impl SignalNode for SinOsc {
    #[inline]
    fn next_sample(&mut self, t: u64, sr: f32) -> f32 {
        let sample = self.phase.sin();
        let freq_sample = self.freq.next_sample(t, sr);
        self.phase += freq_sample * 2.0 * PI / sr;
        // Keep phase bounded to avoid floating-point drift
        self.phase %= 2.0 * PI;
        sample
    }
}

/// Saw wave oscillator (naive, non-band-limited).
pub struct SawOsc {
    freq: f32,
    phase: f32,
}

impl SawOsc {
    pub fn new(freq: f32) -> Self {
        Self { freq, phase: 0.0 }
    }
}

impl SignalNode for SawOsc {
    #[inline]
    fn next_sample(&mut self, _t: u64, sr: f32) -> f32 {
        self.phase += self.freq / sr;
        self.phase -= self.phase.floor();
        2.0 * self.phase - 1.0
    }
}

/// Square wave oscillator (naive, non-band-limited).
pub struct SquareOsc {
    freq: f32,
    phase: f32,
}

impl SquareOsc {
    pub fn new(freq: f32) -> Self {
        Self { freq, phase: 0.0 }
    }
}

impl SignalNode for SquareOsc {
    #[inline]
    fn next_sample(&mut self, _t: u64, sr: f32) -> f32 {
        self.phase += self.freq / sr;
        self.phase -= self.phase.floor();
        if self.phase < 0.5 {
            1.0
        } else {
            -1.0
        }
    }
}

/// Triangle wave oscillator (naive, non-band-limited).
pub struct TriangleOsc {
    freq: f32,
    phase: f32,
}

impl TriangleOsc {
    pub fn new(freq: f32) -> Self {
        Self { freq, phase: 0.0 }
    }
}

impl SignalNode for TriangleOsc {
    #[inline]
    fn next_sample(&mut self, _t: u64, sr: f32) -> f32 {
        self.phase += self.freq / sr;
        self.phase -= self.phase.floor();
        let x = self.phase;
        if x < 0.5 {
            4.0 * x - 1.0
        } else {
            3.0 - 4.0 * x
        }
    }
}
