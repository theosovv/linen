use crate::ir::NodeOrigin;

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
    SetOutput {
        src: AudioRegister,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Bytecode {
    pub instructions: Vec<Instruction>,
    pub scalar_register_count: usize,
    pub audio_register_count: usize,
    pub state_count: usize,
    pub output: AudioRegister,
    pub debug_origins: Option<Vec<NodeOrigin>>,
}

impl Bytecode {
    pub fn strip_debug_info(&mut self) {
        self.debug_origins = None;
    }
}
