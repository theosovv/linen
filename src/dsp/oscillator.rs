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

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Phasor {
    phase: f64,
}

impl Phasor {
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

        let sample = self.phase as f32;

        self.phase = (self.phase + frequency / f64::from(sample_rate)).rem_euclid(1.0);

        Ok(sample)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Saw {
    phase: f64,
}

impl Saw {
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

        let sample = (2.0 * self.phase - 1.0) as f32;

        self.phase = (self.phase + frequency / f64::from(sample_rate)).rem_euclid(1.0);

        Ok(sample)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Square {
    phase: f64,
}

impl Square {
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

        let sample = if self.phase < 0.5 { 1.0_f32 } else { -1.0_f32 };

        self.phase = (self.phase + frequency / f64::from(sample_rate)).rem_euclid(1.0);

        Ok(sample)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SawPolyblep {
    phase: f64,
}

impl SawPolyblep {
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

        let phase_step = frequency / sample_rate as f64;
        let dt = phase_step.abs();
        let t = self.phase;
        let saw_naive = 2.0 * t - 1.0;
        let sample = saw_naive - get_b(t, dt);

        self.phase = (self.phase + phase_step).rem_euclid(1.0);

        Ok(sample as f32)
    }
}

fn get_b(t: f64, dt: f64) -> f64 {
    if dt == 0.0 || dt > 0.5 {
        0.0
    } else if t < dt {
        let x = t / dt;

        2.0 * x - x * x - 1.0
    } else if t > 1.0 - dt {
        let x = (t - 1.0) / dt;

        x * x + 2.0 * x + 1.0
    } else {
        0.0
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SquarePolyblep {
    phase: f64,
}

impl SquarePolyblep {
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

        let phase_step = frequency / sample_rate as f64;
        let dt = phase_step.abs();
        let t = self.phase;
        let square_naive = if t < 0.5 { 1.0 } else { -1.0 };
        let square_shifted = if t < 0.5 { t + 0.5 } else { t - 0.5 };
        let sample = square_naive + get_b(t, dt) - get_b(square_shifted, dt);

        self.phase = (self.phase + phase_step).rem_euclid(1.0);

        Ok(sample as f32)
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Triangle {
    phase: f64,
}

impl Triangle {
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

        let sample = if self.phase < 0.5 {
            (4.0 * self.phase - 1.0) as f32
        } else {
            (3.0 - 4.0 * self.phase) as f32
        };

        self.phase = (self.phase + frequency / f64::from(sample_rate)).rem_euclid(1.0);

        Ok(sample)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NonFiniteFrequency;

impl fmt::Display for NonFiniteFrequency {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("oscillator frequency must be finite")
    }
}

impl Error for NonFiniteFrequency {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NonFinitePosition;

impl fmt::Display for NonFinitePosition {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("pan position must be finite")
    }
}

impl Error for NonFinitePosition {}

pub fn pan(input: f32, position: f32) -> Result<[f32; 2], NonFinitePosition> {
    if !position.is_finite() {
        return Err(NonFinitePosition);
    }

    let p = position.clamp(-1.0, 1.0);
    let angle = (p + 1.0) * PI as f32 / 4.0;
    let left = input * f32::cos(angle);
    let right = input * f32::sin(angle);

    Ok([left, right])
}
