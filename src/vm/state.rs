use crate::dsp::{Noise, Phasor, Saw, SawPolyblep, Sine, Square, SquarePolyblep, Triangle};

#[derive(Debug)]
pub(crate) enum NodeState {
    Sine(Sine),
    Phasor(Phasor),
    Saw(Saw),
    Square(Square),
    SawPolyblep(SawPolyblep),
    SquarePolyblep(SquarePolyblep),
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
            Self::SawPolyblep(state) => state.reset(),
            Self::SquarePolyblep(state) => state.reset(),
            Self::Triangle(state) => state.reset(),
            Self::Noise(state) => state.reset(),
        }
    }
}
