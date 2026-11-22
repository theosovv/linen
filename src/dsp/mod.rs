mod noise;
mod oscillator;

pub use noise::Noise;
pub use oscillator::{
    NonFiniteFrequency, NonFinitePosition, Phasor, Saw, Sine, Square, Triangle, pan,
};
