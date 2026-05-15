//! Audio effects for Linen DSP runtime.

/// Simple delay line with circular buffer.
pub struct Delay {
    buffer: Vec<f32>,
    write_pos: usize,
    delay_samples: usize,
    feedback: f32,
    mix: f32,
}

impl Delay {
    pub fn new(delay_seconds: f32, sample_rate: f32, feedback: f32, mix: f32) -> Self {
        let delay_samples = (delay_seconds * sample_rate) as usize;
        Self {
            buffer: vec![0.0; delay_samples + 1],
            write_pos: 0,
            delay_samples,
            feedback,
            mix,
        }
    }

    #[inline]
    pub fn process(&mut self, input: f32) -> f32 {
        let read_pos =
            (self.write_pos + self.buffer.len() - self.delay_samples) % self.buffer.len();
        let delayed = self.buffer[read_pos];
        self.buffer[self.write_pos] = input + delayed * self.feedback;
        self.write_pos = (self.write_pos + 1) % self.buffer.len();
        input * (1.0 - self.mix) + delayed * self.mix
    }
}
