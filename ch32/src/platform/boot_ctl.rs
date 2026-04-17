use tinyboot_core::traits::BootCtl as TBBootCtl;
use tinyboot_core::traits::BootMode;

use crate::hal::{boot_request, pfic};

/// CH32 boot control (reset, boot mode selection).
///
/// In user-flash mode, caches the app entry point from `__tb_app_entry` linker symbol.
pub struct BootCtl {
    config: boot_request::Config,
    #[cfg(not(feature = "system-flash"))]
    app_entry: u32,
}

impl BootCtl {
    /// Create boot control with the given boot request configuration.
    #[inline(always)]
    pub fn new(config: boot_request::Config) -> Self {
        boot_request::init(&config);
        Self {
            config,
            #[cfg(not(feature = "system-flash"))]
            app_entry: {
                unsafe extern "C" {
                    static __tb_app_entry: u8;
                }
                unsafe { &__tb_app_entry as *const u8 as u32 }
            },
        }
    }
}

impl TBBootCtl for BootCtl {
    fn is_boot_requested(&self) -> bool {
        boot_request::is_boot_requested()
    }

    fn system_reset(&mut self, mode: BootMode) -> ! {
        let bootloader = mode == BootMode::Bootloader;
        boot_request::set_boot_request(&self.config, bootloader);
        // Allow time for external boot mode circuit (RC) to settle
        // before triggering reset. ~1ms at 8MHz = 8000 iterations.
        for _ in 0..8000u16 {
            core::hint::spin_loop();
        }
        #[cfg(not(feature = "system-flash"))]
        if !bootloader {
            crate::hal::flash::lock();
            crate::hal::rcc::reset_apb2();
            pfic::jump(self.app_entry)
        }
        pfic::system_reset()
    }
}
