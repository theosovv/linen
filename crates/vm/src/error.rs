//! VM error types

use thiserror::Error;

/// Result type for VM operations
pub type Result<T> = std::result::Result<T, VmError>;

/// Errors that can occur in the VM
#[derive(Error, Debug, Clone)]
pub enum VmError {
    /// Generic error with message
    #[error("VM error: {0}")]
    Generic(String),
}
