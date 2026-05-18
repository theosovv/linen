//! Filter implementations for Linen DSP runtime.

/// Biquad filter structure (transposed direct form II).
pub struct Biquad {
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
    z1: f32,
    z2: f32,
}

impl Biquad {
    pub fn new(b0: f32, b1: f32, b2: f32, a1: f32, a2: f32) -> Self {
        Self {
            b0,
            b1,
            b2,
            a1,
            a2,
            z1: 0.0,
            z2: 0.0,
        }
    }

    /// Create a lowpass biquad filter.
    pub fn lowpass(cutoff: f32, q: f32, sr: f32) -> Self {
        let omega = 2.0 * std::f32::consts::PI * cutoff / sr;
        let sin_omega = omega.sin();
        let cos_omega = omega.cos();
        let alpha = sin_omega / (2.0 * q);

        let b0 = (1.0 - cos_omega) / 2.0;
        let b1 = 1.0 - cos_omega;
        let b2 = b0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * cos_omega;
        let a2 = 1.0 - alpha;

        Self::new(b0 / a0, b1 / a0, b2 / a0, a1 / a0, a2 / a0)
    }

    /// Process a single sample.
    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let output = input * self.b0 + self.z1;
        self.z1 = input * self.b1 + self.z2 - self.a1 * output;
        self.z2 = input * self.b2 - self.a2 * output;
        output
    }
}
