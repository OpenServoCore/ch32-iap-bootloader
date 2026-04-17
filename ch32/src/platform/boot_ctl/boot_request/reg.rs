//! Run-mode persisted in the flash BOOT_MODE register.

use tinyboot_core::traits::RunMode;

use crate::hal::flash;

pub struct RegRequest;

impl RegRequest {
    #[inline(always)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    #[inline(always)]
    pub fn read(&self) -> RunMode {
        if flash::boot_mode() {
            RunMode::Service
        } else {
            RunMode::HandOff
        }
    }

    #[inline(always)]
    pub fn write(&mut self, mode: RunMode) {
        flash::set_boot_mode(mode == RunMode::Service);
    }
}
