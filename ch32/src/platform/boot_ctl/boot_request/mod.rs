//! Run-mode request persistence across reset.
//!
//! Exactly one variant is selected per build by `build.rs` cfgs:
//! - [`reg`]: flash BOOT_MODE register (system-flash, chips without `boot_pin`).
//! - [`ram`]: magic word in RAM (user-flash).
//! - [`gpio`]: [`ram`] plus a GPIO-driven BOOT0 circuit (system-flash + `boot_pin`).

#[cfg(boot_req_reg)]
mod reg;
#[cfg(boot_req_ram)]
mod ram;
#[cfg(boot_req_gpio)]
mod gpio;

#[cfg(boot_req_reg)]
pub type Active = reg::RegRequest;
#[cfg(all(boot_req_ram, not(boot_req_gpio)))]
pub type Active = ram::RamRequest;
#[cfg(boot_req_gpio)]
pub type Active = gpio::GpioRequest;
