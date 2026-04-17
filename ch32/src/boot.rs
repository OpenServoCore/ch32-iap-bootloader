//! CH32 bootloader entry point.
//!
//! Wires the CH32 [`crate::platform`] implementations into
//! [`tinyboot::Core`] and exposes a minimal [`run`] helper.

use crate::platform::{BootCtl, BootMetaStore, Storage};

pub use crate::platform::{BaudRate, BootCtlConfig, Duplex, TxEnConfig, Usart, UsartConfig};

// Re-exports so boot examples only need this one module.
pub use crate::hal::gpio::Pull;
pub use crate::hal::{Pin, UsartMapping};
pub use tinyboot::Platform;
pub use tinyboot::{boot_version, pkg_version};

/// Common imports for bootloader binaries.
pub mod prelude {
    pub use super::{
        BaudRate, BootCtlConfig, Duplex, Pin, Pull, TxEnConfig, Usart, UsartConfig, UsartMapping,
    };
}

/// Protocol write buffer size (2 × page size).
pub const PAGE_SIZE: usize = crate::hal::flash::PAGE_SIZE;

/// Run the bootloader with the given transport and boot control config.
///
/// Sets up storage, boot metadata, and boot control from linker symbols,
/// then enters the boot state machine. Does not return.
#[inline(always)]
pub fn run(transport: impl tinyboot::traits::Transport, config: BootCtlConfig) -> ! {
    let platform = Platform::new(
        transport,
        Storage::default(),
        BootMetaStore::default(),
        BootCtl::new(config),
    );
    tinyboot::Core::<_, _, _, _, { 2 * PAGE_SIZE }>::new(platform).run()
}
