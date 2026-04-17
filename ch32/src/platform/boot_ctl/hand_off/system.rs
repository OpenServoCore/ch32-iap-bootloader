//! System-flash hand-off: software reset into the factory ROM, which reads
//! the BOOT_MODE register and dispatches to user flash when cleared.

use crate::hal::pfic;

pub struct SystemHandOff;

impl SystemHandOff {
    #[inline(always)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    #[inline(always)]
    pub fn execute(&mut self) -> ! {
        pfic::software_reset()
    }
}
