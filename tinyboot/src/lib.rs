#![no_std]

pub mod protocol;
pub mod traits;

mod log;

pub use tinyboot_protocol as wire;

use traits::{BootCtl, BootMetaStore, BootState, Platform, Storage, Transport};

#[inline(never)]
fn read_byte<T: embedded_io::Read>(transport: &mut T) -> u8 {
    let mut b = [0u8; 1];
    loop {
        match transport.read(&mut b) {
            Ok(n) if n > 0 => return b[0],
            _ => {}
        }
    }
}

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
    fn app_is_blank(&self) -> bool {
        let data = self.platform.storage.as_slice();
        data.len() < 4 || data[..4] == [0xFF; 4]
    }

    fn enter_bootloader(&mut self) -> ! {
        log_info!("Entering bootloader mode");

        let Platform {
            transport,
            storage,
            boot_meta,
            ctl,
        } = &mut self.platform;
        let mut data_buf = [0u8; 2];

        loop {
            // Sync on frame header [0xAA, 0x55]
            let mut prev = 0u8;
            loop {
                let b = read_byte(transport);
                if prev == wire::HEAD[0] && b == wire::HEAD[1] {
                    break;
                }
                prev = b;
            }

            // Read fixed header: CMD(1) + LEN(1) + ADDR_LO(1) + ADDR_HI(1)
            let cmd_byte = read_byte(transport);
            let len = read_byte(transport);
            let addr_lo = read_byte(transport);
            let addr_hi = read_byte(transport);

            let data_len = len as usize;
            if data_len > data_buf.len() {
                continue;
            }

            // Read data bytes
            for i in 0..data_len {
                data_buf[i] = read_byte(transport);
            }

            // Validate CRC over header + data
            let mut crc = wire::crc::crc16(wire::CRC_INIT, &[cmd_byte, len, addr_lo, addr_hi]);
            if data_len > 0 {
                crc = wire::crc::crc16(crc, &data_buf[..data_len]);
            }
            let crc_lo = read_byte(transport);
            let crc_hi = read_byte(transport);
            if u16::from_le_bytes([crc_lo, crc_hi]) != crc {
                continue;
            }

            // Dispatch command
            let addr = u16::from_le_bytes([addr_lo, addr_hi]) as u32;
            match wire::Cmd::from_u8(cmd_byte) {
                Some(wire::Cmd::Info) => {
                    let _ = protocol::handle_info(transport, storage);
                }
                Some(wire::Cmd::Erase) => {
                    let _ = protocol::handle_erase(transport, storage);
                }
                Some(wire::Cmd::Write) => {
                    let _ = protocol::handle_write(
                        transport, storage, addr, &data_buf[..data_len],
                    );
                }
                Some(wire::Cmd::Verify) => {
                    let _ = protocol::handle_verify(transport, storage);
                }
                Some(wire::Cmd::Reset) => {
                    protocol::handle_reset(transport, boot_meta, ctl);
                }
                None => {}
            }
        }
    }
}
