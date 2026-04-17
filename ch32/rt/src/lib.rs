#![no_std]

//! Minimal bootloader runtime for tinyboot on CH32.
//!
//! Provides a tiny `_start` + `.init` section and the companion `link.x`
//! script. Used by bootloader binaries that can't afford the full
//! `qingke-rt` runtime (system-flash builds must fit in ~2 KB).
//!
//! Apps should keep using `qingke-rt`; this crate must not be a dependency
//! of application binaries or the `_start` symbols will collide at link time.

core::arch::global_asm!(include_str!("start.S"));
