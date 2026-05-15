//! Signal graph execution engine.

use crate::traits::SignalNode;

/// An executable signal graph.
pub struct SignalGraphExecutor {
    // TODO: store nodes and execute them in topological order
}

impl SignalGraphExecutor {
    pub fn new() -> Self {
        Self {}
    }

    /// Execute the graph for `num_samples` and write to the output buffer.
    pub fn execute(&mut self, _num_samples: usize, _output: &mut [f32]) {
        // TODO: implement graph execution
    }
}

impl Default for SignalGraphExecutor {
    fn default() -> Self {
        Self::new()
    }
}
