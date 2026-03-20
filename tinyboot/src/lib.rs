#![no_std]
#![warn(missing_docs)]

//! Platform-agnostic bootloader core.
//!
//! Implements the boot state machine, protocol dispatcher, and app validation.
//! Platform-specific behaviour is injected via the traits in [`traits::boot`].

#[macro_use]
mod log;

/// Boot state machine and entry point.
pub mod core;
/// Protocol frame dispatcher.
pub mod protocol;
/// Platform abstraction traits.
pub mod traits;

pub use crate::core::Core;
