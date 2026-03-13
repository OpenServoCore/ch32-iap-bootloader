#![no_std]

pub mod crc;
pub mod traits;

mod log;

use traits::{BootCtl, BootMetaStore, BootState, Platform, Storage, Transport};

pub struct Core<T, S, B, C>
where
    T: Transport,
    S: Storage,
    B: BootMetaStore,
    C: BootCtl,
{
    platform: Platform<T, S, B, C>,
}

impl<T, S, B, C> Core<T, S, B, C>
where
    T: Transport,
    S: Storage,
    B: BootMetaStore,
    C: BootCtl,
{
    pub fn new(platform: Platform<T, S, B, C>) -> Self {
        Core { platform }
    }

    pub fn run(&mut self) -> ! {
        log_info!("Bootloader started");

        let mut enter = self.platform.ctl.take_boot_request();

        if enter {
            log_info!("Boot requested");
            self.platform.boot_meta.advance().unwrap();
        } else {
            let meta = self.platform.boot_meta.read();
            match meta.boot_state() {
                BootState::Idle | BootState::Confirmed => {}
                BootState::Updating | BootState::Corrupt => enter = true,
                BootState::Validating => {
                    if meta.trials_remaining() == 0 {
                        enter = true;
                    } else {
                        self.platform.boot_meta.consume_trial().unwrap();
                    }
                }
            }
        }

        if enter || self.app_is_blank() {
            self.enter_bootloader();
        }
        self.platform.ctl.jump_to_app();
    }

    /// Check if the app region contains valid code by reading the first word.
    /// Erased flash reads as 0xFFFFFFFF.
    fn app_is_blank(&mut self) -> bool {
        let mut buf = [0u8; 4];
        if self.platform.storage.read(0, &mut buf).is_err() {
            return true;
        }
        buf == [0xFF; 4]
    }

    fn enter_bootloader(&mut self) -> ! {
        log_info!("Entering bootloader mode");
        // TODO: firmware update loop over transport
        loop {
            core::hint::spin_loop();
        }
    }
}
