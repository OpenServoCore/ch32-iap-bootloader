//! User-flash hand-off: reset APB2 peripherals, then jump to the app's reset
//! vector at the `__tb_app_entry` linker symbol.

use crate::hal::{pfic, rcc};

pub struct UserHandOff {
    app_entry: u32,
}

impl UserHandOff {
    #[inline(always)]
    pub fn new() -> Self {
        unsafe extern "C" {
            static __tb_app_entry: u8;
        }
        Self {
            app_entry: unsafe { &__tb_app_entry as *const u8 as u32 },
        }
    }

    #[inline(always)]
    pub fn execute(&mut self) -> ! {
        rcc::reset_apb2();
        pfic::jump(self.app_entry)
    }
}
