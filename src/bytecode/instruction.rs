use crate::ir::{NodeOrigin, RandomStreamId};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ScalarRegister(usize);

impl ScalarRegister {
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct AudioRegister(usize);

impl AudioRegister {
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StateId(usize);

impl StateId {
    pub const fn new(index: usize) -> Self {
        Self(index)
    }

    pub const fn index(self) -> usize {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputLayout {
    Mono(AudioRegister),
    Stereo {
        left: AudioRegister,
        right: AudioRegister,
    },
}

impl OutputLayout {
    pub const fn channels(self) -> usize {
        match self {
            Self::Mono(_) => 1,
            Self::Stereo { .. } => 2,
        }
    }

    pub const fn contains(self, register: AudioRegister) -> bool {
        match self {
            Self::Mono(output) => output.index() == register.index(),
            Self::Stereo { left, right } => {
                left.index() == register.index() || right.index() == register.index()
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Operand {
    Scalar(ScalarRegister),
    Audio(AudioRegister),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Instruction {
    ConstScalar {
        dst: ScalarRegister,
        value: f64,
    },
    Lift {
        dst: AudioRegister,
        src: ScalarRegister,
    },
    NegScalar {
        dst: ScalarRegister,
        src: ScalarRegister,
    },
    NegAudio {
        dst: AudioRegister,
        src: AudioRegister,
    },
    BinaryScalar {
        op: BinaryOp,
        dst: ScalarRegister,
        lhs: ScalarRegister,
        rhs: ScalarRegister,
    },
    BinaryAudio {
        op: BinaryOp,
        dst: AudioRegister,
        lhs: Operand,
        rhs: Operand,
    },
    Sine {
        dst: AudioRegister,
        freq: Operand,
        state: StateId,
    },
    Phasor {
        dst: AudioRegister,
        freq: Operand,
        state: StateId,
    },
    Saw {
        dst: AudioRegister,
        freq: Operand,
        state: StateId,
    },
    Square {
        dst: AudioRegister,
        freq: Operand,
        state: StateId,
    },
    Triangle {
        dst: AudioRegister,
        freq: Operand,
        state: StateId,
    },
    MakeStereo {
        dst_left: AudioRegister,
        dst_right: AudioRegister,
        src_left: AudioRegister,
        src_right: AudioRegister,
    },
    Pan {
        dst_left: AudioRegister,
        dst_right: AudioRegister,
        input: Operand,
        position: Operand,
    },
    Noise {
        dst: AudioRegister,
        stream: RandomStreamId,
        state: StateId,
    },
    SetOutput {
        src: AudioRegister,
    },
    SetStereoOutput {
        src_left: AudioRegister,
        src_right: AudioRegister,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bytecode {
    pub instructions: Vec<Instruction>,
    pub scalar_register_count: usize,
    pub audio_register_count: usize,
    pub state_count: usize,
    pub output: OutputLayout,
    pub debug_origins: Option<Vec<NodeOrigin>>,
}

impl Bytecode {
    pub fn strip_debug_info(&mut self) {
        self.debug_origins = None;
    }
}
