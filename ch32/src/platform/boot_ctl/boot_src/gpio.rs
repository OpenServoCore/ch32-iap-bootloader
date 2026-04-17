//! GPIO-driven BOOT0 select (V103 + system-flash).
//!
//! Drives an external RC/flip-flop that latches BOOT0 for the next power-on
//! reset. `reset_delay_cycles` lets the circuit settle before the caller
//! triggers a software reset.

use super::BootSrc;
use crate::hal::{Pin, gpio, rcc};

pub struct GpioBootSrcCtl {
    pin: Pin,
    /// Level to drive for [`BootSrc::SystemFlash`]; the inverse is driven for
    /// [`BootSrc::UserFlash`].
    system_flash_level: gpio::Level,
    reset_delay_cycles: u32,
}

impl GpioBootSrcCtl {
    #[inline(always)]
    pub fn new(pin: Pin, system_flash_level: gpio::Level, reset_delay_cycles: u32) -> Self {
        rcc::enable_gpio(pin.port_index());
        gpio::configure(pin, gpio::PinMode::OUTPUT_PUSH_PULL);
        let s = Self {
            pin,
            system_flash_level,
            reset_delay_cycles,
        };
        s.drive(BootSrc::SystemFlash);
        s
    }

    #[inline(always)]
    pub fn set(&mut self, src: BootSrc) {
        self.drive(src);
        crate::hal::delay_cycles(self.reset_delay_cycles);
    }

    #[inline(always)]
    fn drive(&self, src: BootSrc) {
        let level = match (src, self.system_flash_level) {
            (BootSrc::SystemFlash, l) => l,
            (BootSrc::UserFlash, gpio::Level::High) => gpio::Level::Low,
            (BootSrc::UserFlash, gpio::Level::Low) => gpio::Level::High,
        };
        gpio::set_level(self.pin, level);
    }
}
