//! Envelope generators for Linen DSP runtime.

/// ADSR envelope state machine.
pub enum AdsrState {
    Idle,
    Attack,
    Decay,
    Sustain,
    Release,
}

/// ADSR envelope generator.
pub struct Adsr {
    attack: f32,
    decay: f32,
    sustain: f32,
    release: f32,
    state: AdsrState,
    level: f32,
    sample_rate: f32,
}

impl Adsr {
    pub fn new(attack: f32, decay: f32, sustain: f32, release: f32, sr: f32) -> Self {
        Self {
            attack,
            decay,
            sustain,
            release,
            state: AdsrState::Idle,
            level: 0.0,
            sample_rate: sr,
        }
    }

    pub fn trigger(&mut self) {
        self.state = AdsrState::Attack;
        self.level = 0.0;
    }

    pub fn release(&mut self) {
        self.state = AdsrState::Release;
    }

    /// Generate next envelope sample.
    #[inline]
    pub fn next_sample(&mut self) -> f32 {
        match self.state {
            AdsrState::Idle => 0.0,
            AdsrState::Attack => {
                self.level += 1.0 / (self.attack * self.sample_rate);
                if self.level >= 1.0 {
                    self.level = 1.0;
                    self.state = AdsrState::Decay;
                }
                self.level
            }
            AdsrState::Decay => {
                self.level -= (1.0 - self.sustain) / (self.decay * self.sample_rate);
                if self.level <= self.sustain {
                    self.level = self.sustain;
                    self.state = AdsrState::Sustain;
                }
                self.level
            }
            AdsrState::Sustain => self.sustain,
            AdsrState::Release => {
                self.level -= self.sustain / (self.release * self.sample_rate);
                if self.level <= 0.0 {
                    self.level = 0.0;
                    self.state = AdsrState::Idle;
                }
                self.level
            }
        }
    }
}
