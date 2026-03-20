#![no_std]

use tinyboot::traits::BootState;
use tinyboot::traits::app::BootClient as TBBootClient;
use tinyboot_ch32_hal::{flash, iwdg, pfic};

// Re-exports so apps only need this one crate.
pub use tinyboot::app::{App, AppConfig};
pub use tinyboot::traits::app as traits;
pub use tinyboot_protocol::pkg_version;

/// Define the `.tinyboot_version` static using the calling crate's version.
/// Place this at module scope in your application binary.
#[macro_export]
macro_rules! app_version {
    () => {
        #[unsafe(link_section = ".tinyboot_version")]
        #[used]
        static _APP_VERSION: u16 = $crate::pkg_version!();
    };
}

/// CH32 boot client implementation.
pub struct Ch32BootClient;

impl TBBootClient for Ch32BootClient {
    fn confirm(&mut self) {
        critical_section::with(|_| {
            let ob = flash::META_OB_BASE;
            let state = unsafe { core::ptr::read_volatile(ob as *const u8) };
            if BootState::from_u8(state) != BootState::Validating {
                return;
            }
            // Read current meta, set state=Idle and reset trials
            let mut meta = [0xFFu8; 8];
            for (i, slot) in meta.iter_mut().enumerate() {
                *slot = unsafe { core::ptr::read_volatile((ob + i as u32 * 2) as *const u8) };
            }
            meta[0] = BootState::Idle as u8;
            meta[1] = 0xFF;
            // Read chip config, erase OB, rewrite
            let mut buf = [0xFFu8; 16];
            for (i, slot) in buf[..8].iter_mut().enumerate() {
                *slot = unsafe {
                    core::ptr::read_volatile((flash::OB_BASE + i as u32 * 2) as *const u8)
                };
            }
            buf[8..16].copy_from_slice(&meta);
            flash::unlock();
            iwdg::feed();
            let w = flash::FlashWriter::ob();
            w.erase_start();
            w.erase(flash::OB_BASE);
            w.operation_end();
            w.write_start();
            for (i, &byte) in buf.iter().enumerate() {
                if byte != 0xFF {
                    w.write(flash::OB_BASE + (i as u32 * 2), byte as u16);
                }
            }
            w.operation_end();
            flash::lock();
        });
    }

    fn request_update(&mut self) {
        critical_section::with(|_| {
            #[cfg(feature = "system-flash")]
            flash::set_boot_mode(true);
            #[cfg(not(feature = "system-flash"))]
            tinyboot_ch32_hal::boot_request::set_boot_request(true);
        });
    }

    fn system_reset(&mut self) -> ! {
        pfic::system_reset()
    }
}

/// Create an [`App`] configured for CH32 hardware.
///
/// Reads boot version from flash at `boot_base + boot_size - 2`.
pub fn new_app(
    boot_base: u32,
    boot_size: u32,
    app_size: u32,
    erase_size: u16,
) -> App<Ch32BootClient> {
    let boot_ver_addr = (boot_base + boot_size - 2) as *const u16;
    App::new(
        AppConfig {
            capacity: app_size,
            erase_size,
            boot_version: unsafe { boot_ver_addr.read_volatile() },
            app_version: pkg_version!(),
        },
        Ch32BootClient,
    )
}
