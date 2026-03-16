//! Linen Standard Library
//!
//! This crate provides the standard library for Linen:
//! - Audio primitives (oscillators, filters, envelopes)
//! - FRP combinators (Behavior, Event, Signal Function)
//! - Effects (reverb, delay, compression)
//! - I/O (MIDI, audio files)

#![warn(missing_docs)]

/// Audio primitives (oscillators, filters)
pub mod primitives;

/// FRP abstractions
pub mod frp;

/// Effects (reverb, delay, dynamics)
pub mod effects;

/// Time utilities (BPM, transport)
pub mod time;

/// I/O operations
pub mod io;

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
