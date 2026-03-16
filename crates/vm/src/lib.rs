//! Linen Virtual Machine
//!
//! This crate implements the Linen VM:
//! - Stack-based bytecode execution
//! - Audio thread with real-time guarantees
//! - Memory management (region-based, no GC in audio thread)
//! - JIT compilation support (hooks for Cranelift)

#![warn(missing_docs)]

/// Bytecode format and instructions
pub mod bytecode;

/// VM execution engine
pub mod vm;

/// Audio thread and real-time processing
pub mod audio;

/// Memory management (regions, pools, linear types)
pub mod memory;

/// JIT compilation hooks
pub mod jit;

/// Error types
pub mod error;

pub use error::{Result, VmError};
pub use vm::Vm;

/// VM version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
