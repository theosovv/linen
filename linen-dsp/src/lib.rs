//! DSP runtime for the Linen programming language.
//!
//! Provides oscillators, filters, envelopes, effects, and audio I/O.

pub mod effects;
pub mod envelopes;
pub mod ffi;
pub mod filters;
pub mod graph;
pub mod io;
pub mod oscillators;
pub mod traits;

/// Re-export core trait.
pub use traits::SignalNode;
