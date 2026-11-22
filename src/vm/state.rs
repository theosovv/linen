use crate::dsp::{Noise, Phasor, Saw, Sine, Square, Triangle};

#[derive(Debug)]
pub(crate) enum NodeState {
    Sine(Sine),
    Phasor(Phasor),
    Saw(Saw),
    Square(Square),
    Triangle(Triangle),
    Noise(Noise),
}

impl NodeState {
    pub(crate) fn reset(&mut self) {
        match self {
            Self::Sine(state) => state.reset(),
            Self::Phasor(state) => state.reset(),
            Self::Saw(state) => state.reset(),
            Self::Square(state) => state.reset(),
            Self::Triangle(state) => state.reset(),
            Self::Noise(state) => state.reset(),
        }
    }
}
