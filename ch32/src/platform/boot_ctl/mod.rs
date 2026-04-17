//! CH32 boot control: run-mode intent, reset, hand-off to app.
//!
//! Composes two orthogonal concerns:
//! - [`boot_request`]: how run-mode intent survives a reset (reg/ram/ram+gpio).
//! - [`hand_off`]: how control transfers to the app (system-flash reset vs user-flash jump).

use tinyboot_core::traits::{BootCtl as TBBootCtl, RunMode};

use crate::hal::pfic;

mod boot_request;
mod hand_off;

#[cfg(boot_req_gpio)]
use crate::hal::Pin;

pub struct BootCtl {
    req: boot_request::Active,
    hand_off: hand_off::Active,
}

impl BootCtl {
    #[cfg(boot_req_gpio)]
    #[inline(always)]
    pub fn new(pin: Pin, active_high: bool, reset_delay_cycles: u32) -> Self {
        Self {
            req: boot_request::Active::new(pin, active_high, reset_delay_cycles),
            hand_off: hand_off::Active::new(),
        }
    }

    #[cfg(not(boot_req_gpio))]
    #[inline(always)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            req: boot_request::Active::new(),
            hand_off: hand_off::Active::new(),
        }
    }
}

impl TBBootCtl for BootCtl {
    #[inline(always)]
    fn run_mode(&self) -> RunMode {
        self.req.read()
    }

    #[inline(always)]
    fn set_run_mode(&mut self, mode: RunMode) {
        self.req.write(mode);
    }

    #[inline(always)]
    fn reset(&mut self) -> ! {
        pfic::software_reset()
    }

    #[inline(always)]
    fn hand_off(&mut self) -> ! {
        // Clearing the request before hand-off keeps the next reset in HandOff
        // unless the app explicitly re-requests Service.
        self.req.write(RunMode::HandOff);
        self.hand_off.execute()
    }
}
