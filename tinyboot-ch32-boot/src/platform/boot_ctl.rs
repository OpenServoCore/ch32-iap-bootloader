use tinyboot::traits::BootCtl as TBBootCtl;

use tinyboot_ch32_hal::pfic;

pub struct BootCtl;

impl TBBootCtl for BootCtl {
    fn is_boot_requested(&self) -> bool {
        #[cfg(feature = "system-flash")]
        {
            tinyboot_ch32_hal::flash::is_boot_mode()
        }
        #[cfg(not(feature = "system-flash"))]
        {
            tinyboot_ch32_hal::boot_request::is_boot_requested()
        }
    }

    fn clear_boot_request(&mut self) {
        #[cfg(feature = "system-flash")]
        tinyboot_ch32_hal::flash::set_boot_mode(false);
        #[cfg(not(feature = "system-flash"))]
        tinyboot_ch32_hal::boot_request::set_boot_request(false);
    }

    fn system_reset(&mut self) -> ! {
        pfic::system_reset();
    }
}
