use std::{error::Error, f64::consts::PI, fmt};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sine {
    phase: f64,
}

impl Sine {
    pub const fn new() -> Self {
        Self { phase: 0.0 }
    }

    pub const fn phase(&self) -> f64 {
        self.phase
    }

    pub fn reset(&mut self) {
        self.phase = 0.0;
    }

    pub fn next_sample(
        &mut self,
        frequency: f64,
        sample_rate: u32,
    ) -> Result<f32, NonFiniteFrequency> {
        if !frequency.is_finite() {
            return Err(NonFiniteFrequency);
        }

        let sample = f64::sin((2.0 * PI) * self.phase);

        self.phase = (self.phase + frequency / (sample_rate as f64)).rem_euclid(1.0);

        Ok(sample as f32)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NonFiniteFrequency;

impl fmt::Display for NonFiniteFrequency {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("sine frequency must be finite")
    }
}

impl Error for NonFiniteFrequency {}
