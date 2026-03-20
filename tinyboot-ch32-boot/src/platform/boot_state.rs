use tinyboot::traits::BootState;
use tinyboot::traits::boot::BootMetaStore as TBBootMetaStore;
use tinyboot_ch32_hal::flash;

#[derive(Debug)]
pub enum BootMetaError {
    InvalidTransition,
    TrialsExhausted,
}

/// CH32 boot metadata backed by option bytes.
#[derive(Default)]
pub struct BootMetaStore;

impl TBBootMetaStore for BootMetaStore {
    type Error = BootMetaError;

    fn boot_state(&self) -> BootState {
        BootState::from_u8(flash::ob_boot_state())
    }

    fn trials_remaining(&self) -> u8 {
        flash::ob_trials().count_ones() as u8
    }

    fn app_checksum(&self) -> u16 {
        flash::ob_checksum()
    }

    fn advance(&mut self) -> Result<BootState, Self::Error> {
        let next = flash::ob_step_down(flash::OB_STATE_ADDR, BootState::Validating as u8)
            .ok_or(BootMetaError::InvalidTransition)?;
        Ok(BootState::from_u8(next))
    }

    fn consume_trial(&mut self) -> Result<(), Self::Error> {
        flash::ob_step_down(flash::OB_TRIALS_ADDR, 0).ok_or(BootMetaError::TrialsExhausted)?;
        Ok(())
    }

    fn refresh(&mut self, checksum: u16, state: BootState) -> Result<(), Self::Error> {
        flash::ob_refresh(state as u8, checksum);
        Ok(())
    }
}
