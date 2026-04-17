//! RAM-persisted request with an extra GPIO driving an external BOOT0 circuit.
//!
//! Wraps [`super::ram::RamRequest`] for the intent bit; the GPIO signals the
//! RC/flip-flop circuit that selects boot source on the next power-on.
//! `reset_delay_cycles` lets that circuit settle before the caller resets.

use tinyboot_core::traits::RunMode;

use super::ram::RamRequest;
use crate::hal::{Pin, gpio, rcc};

pub struct GpioRequest {
    ram: RamRequest,
    pin: Pin,
    active_high: bool,
    reset_delay_cycles: u32,
}

impl GpioRequest {
    #[inline(always)]
    pub fn new(pin: Pin, active_high: bool, reset_delay_cycles: u32) -> Self {
        rcc::enable_gpio(pin.port_index());
        gpio::configure(pin, gpio::PinMode::OUTPUT_PUSH_PULL);
        let s = Self {
            ram: RamRequest::new(),
            pin,
            active_high,
            reset_delay_cycles,
        };
        s.drive(true);
        s
    }

    #[inline(always)]
    pub fn read(&self) -> RunMode {
        self.ram.read()
    }

    #[inline(always)]
    pub fn write(&mut self, mode: RunMode) {
        self.ram.write(mode);
        self.drive(mode == RunMode::Service);
        crate::hal::delay_cycles(self.reset_delay_cycles);
    }

    #[inline(always)]
    fn drive(&self, service: bool) {
        let level = if self.active_high == service {
            gpio::Level::High
        } else {
            gpio::Level::Low
        };
        gpio::set_level(self.pin, level);
    }
}
