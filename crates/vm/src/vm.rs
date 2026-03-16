//! Virtual machine execution engine

/// The Linen virtual machine
#[derive(Debug)]
pub struct Vm;

impl Vm {
    /// Create a new VM instance
    pub fn new() -> Self {
        Self
    }
}

impl Default for Vm {
    fn default() -> Self {
        Self::new()
    }
}
