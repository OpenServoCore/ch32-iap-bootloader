use crate::traits::BootState;
use crate::traits::boot::{BootCtl, BootMetaStore, Platform, Storage, Transport};
use tinyboot_protocol::crc::{CRC_INIT, crc16};
use tinyboot_protocol::frame::{Frame, InfoData, VerifyData};
use tinyboot_protocol::{Cmd, ReadError, Status};

/// Protocol dispatcher. Borrows the platform, owns the frame.
pub struct Dispatcher<'a, T: Transport, S: Storage, B: BootMetaStore, C: BootCtl> {
    /// Mutable reference to the platform peripherals.
    pub platform: &'a mut Platform<T, S, B, C>,
    /// Reusable frame buffer.
    pub frame: Frame,
}

impl<'a, T: Transport, S: Storage, B: BootMetaStore, C: BootCtl> Dispatcher<'a, T, S, B, C> {
    /// Create a new dispatcher for the given platform.
    pub fn new(platform: &'a mut Platform<T, S, B, C>) -> Self {
        Self {
            platform,
            frame: Frame::default(),
        }
    }

    /// Read a frame, dispatch the command, and send the response.
    /// Returns Err only for transport IO errors. Frame-level errors
    /// (bad CRC, invalid frame) are silently skipped.
    pub fn dispatch(&mut self) -> Result<(), ReadError> {
        let status = self.frame.read(&mut self.platform.transport)?;

        if status != Status::Ok {
            self.frame.len = 0;
            self.frame.status = status;
            return self
                .frame
                .send(&mut self.platform.transport)
                .map_err(|_| ReadError);
        }

        let data_len = self.frame.len as usize;
        let capacity = self.platform.storage.capacity() as u32;
        let erase_size = S::ERASE_SIZE as u32;
        let write_size = S::WRITE_SIZE as u32;
        let state = self.platform.boot_meta.boot_state();
        self.frame.len = 0;
        self.frame.status = Status::Ok;

        match self.frame.cmd {
            Cmd::Info => {
                self.frame.len = 12;
                let boot_ver =
                    self.platform.storage.boot_base() + self.platform.storage.boot_size() - 2;
                let app_ver =
                    self.platform.storage.as_slice().as_ptr() as usize + capacity as usize - 2;
                self.frame.data.info = InfoData {
                    capacity,
                    erase_size: erase_size as u16,
                    boot_version: unsafe { (boot_ver as *const u16).read_volatile() },
                    app_version: unsafe { (app_ver as *const u16).read_volatile() },
                    mode: 0,
                };
            }
            Cmd::Erase => {
                let addr = self.frame.addr;
                let byte_count = unsafe { self.frame.data.erase }.byte_count as u32;
                if !addr.is_multiple_of(erase_size)
                    || !byte_count.is_multiple_of(erase_size)
                    || byte_count == 0
                    || addr + byte_count > capacity
                {
                    self.frame.status = Status::AddrOutOfBounds;
                } else {
                    // State transitions for erase
                    match state {
                        // Idle → Updating: step down state byte
                        BootState::Idle => {
                            if self.platform.boot_meta.advance().is_err() {
                                self.frame.status = Status::WriteError;
                            }
                        }
                        // Validating → Updating: app failed, reflashing
                        BootState::Validating => {
                            if self
                                .platform
                                .boot_meta
                                .refresh(0xFFFF, BootState::Updating)
                                .is_err()
                            {
                                self.frame.status = Status::WriteError;
                            }
                        }
                        // Updating → Updating: no state change
                        BootState::Updating => {}
                    }
                    if self.frame.status == Status::Ok
                        && self
                            .platform
                            .storage
                            .erase(addr, addr + byte_count)
                            .is_err()
                    {
                        self.frame.status = Status::WriteError;
                    }
                }
            }
            Cmd::Write => {
                if state != BootState::Updating {
                    self.frame.status = Status::Unsupported;
                } else {
                    let addr = self.frame.addr;
                    if addr >= capacity
                        || addr + data_len as u32 > capacity
                        || !addr.is_multiple_of(write_size)
                    {
                        self.frame.status = Status::AddrOutOfBounds;
                    } else if self
                        .platform
                        .storage
                        // SAFETY: data_len <= MAX_PAYLOAD validated by frame.read() overflow check
                        .write(addr, unsafe {
                            self.frame.data.raw.get_unchecked(..data_len)
                        })
                        .is_err()
                    {
                        self.frame.status = Status::WriteError;
                    }
                }
            }
            Cmd::Verify => {
                if state != BootState::Updating {
                    self.frame.status = Status::Unsupported;
                } else {
                    let crc = crc16(CRC_INIT, self.platform.storage.as_slice());
                    self.frame.len = 2;
                    self.frame.data.verify = VerifyData { crc };
                    if self
                        .platform
                        .boot_meta
                        .refresh(crc, BootState::Validating)
                        .is_err()
                    {
                        self.frame.status = Status::WriteError;
                    }
                }
            }
            Cmd::Reset => {
                let _ = self.frame.send(&mut self.platform.transport);
                self.platform.ctl.system_reset(self.frame.addr == 1);
            }
        }

        self.frame
            .send(&mut self.platform.transport)
            .map_err(|_| ReadError)
    }
}
