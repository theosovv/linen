//! Core trait for DSP signal nodes.

/// A node in the signal graph that can produce audio samples.
pub trait SignalNode {
    /// Generate the next audio sample.
    ///
    /// # Arguments
    /// * `t` - Current sample time (absolute, monotonically increasing)
    /// * `sr` - Sample rate in Hz
    ///
    /// # Returns
    /// The sample value for time `t`, typically in range [-1.0, 1.0]
    fn next_sample(&mut self, t: u64, sr: f32) -> f32;
}

/// A constant signal node (for fixed frequencies, DC offsets, etc.).
pub struct ConstNode(pub f32);

impl SignalNode for ConstNode {
    #[inline]
    fn next_sample(&mut self, _t: u64, _sr: f32) -> f32 {
        self.0
    }
}

/// Sample rate constants.
pub const SAMPLE_RATE_44K: f32 = 44100.0;
pub const SAMPLE_RATE_48K: f32 = 48000.0;
pub const SAMPLE_RATE_96K: f32 = 96000.0;
