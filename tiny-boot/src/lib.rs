#![no_std]

pub mod traits;

mod log;

use traits::{BootCtl, BootState, BootStateStore, Platform, Storage, Transport};

pub struct Core<T, S, R, C>
where
    T: Transport,
    S: Storage,
    R: BootStateStore,
    C: BootCtl,
{
    platform: Platform<T, S, R, C>,
}

impl<T, S, R, C> Core<T, S, R, C>
where
    T: Transport,
    S: Storage,
    R: BootStateStore,
    C: BootCtl,
{
    pub fn new(platform: Platform<T, S, R, C>) -> Self {
        Core { platform }
    }

    pub fn run(&mut self) -> ! {
        log_info!("Bootloader started");

        let state = self.platform.reg.state().unwrap_or(BootState::Idle);
        log_info!("Boot state: {:?}", state);

        match state {
            BootState::Idle => self.handle_idle(),
            BootState::Updating => self.handle_updating(),
            BootState::Validating => self.handle_validating(),
            BootState::Confirmed => self.handle_confirmed(),
        }
    }

    fn handle_idle(&mut self) -> ! {
        if self.platform.reg.boot_requested().unwrap_or(false) {
            log_info!("Boot requested, entering bootloader mode");
            self.platform.reg.transition().ok();
            self.enter_bootloader();
        }
        log_info!("Jumping to application");
        self.platform.ctl.jump_to_app();
    }

    fn handle_updating(&mut self) -> ! {
        log_info!("Update in progress");
        self.enter_bootloader();
    }

    fn handle_validating(&mut self) -> ! {
        let remaining = self.platform.reg.trials_remaining().unwrap_or(0);
        if remaining == 0 {
            log_info!("Trial boots exhausted, entering bootloader mode");
            self.enter_bootloader();
        }
        log_info!("Trial boot ({} remaining)", remaining);
        self.platform.reg.increment_trial().ok();
        self.platform.ctl.jump_to_app();
    }

    fn handle_confirmed(&mut self) -> ! {
        log_info!("Boot confirmed, resetting state");
        self.platform.reg.transition().ok(); // Confirmed → Idle (erase)
        self.platform.ctl.jump_to_app();
    }

    fn enter_bootloader(&mut self) -> ! {
        log_info!("Entering bootloader mode");
        // TODO: firmware update loop over transport
        loop {}
    }
}
