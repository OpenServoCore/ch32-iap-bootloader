#![no_std]

pub mod command;
pub mod crc;
pub mod response;

/// Frame header.
pub const HEAD: [u8; 2] = [0xAA, 0x55];

/// CRC16 initial value.
pub const CRC_INIT: u16 = 0xFFFF;

/// Write the frame envelope: HEAD at `buf[0..2]`, CRC over `buf[2..payload_end]`,
/// then CRC. Returns total frame length.
pub(crate) fn seal(buf: &mut [u8], payload_end: usize) -> usize {
    buf[0] = HEAD[0];
    buf[1] = HEAD[1];
    let crc = crc::crc16(CRC_INIT, &buf[2..payload_end]);
    buf[payload_end] = crc as u8;
    buf[payload_end + 1] = (crc >> 8) as u8;
    payload_end + 2
}

/// Commands (host → device).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Cmd {
    Info = 0x01,
    Erase = 0x02,
    Write = 0x03,
    Verify = 0x04,
    Reset = 0x05,
}

impl Cmd {
    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            0x01 => Some(Cmd::Info),
            0x02 => Some(Cmd::Erase),
            0x03 => Some(Cmd::Write),
            0x04 => Some(Cmd::Verify),
            0x05 => Some(Cmd::Reset),
            _ => None,
        }
    }
}

/// Response status codes (device → host).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub enum Status {
    Ok = 0x00,
    Error = 0x01,
    CrcMismatch = 0x02,
    AddrOutOfBounds = 0x03,
    NotReady = 0x04,
}

impl Status {
    pub fn from_u8(b: u8) -> Option<Self> {
        match b {
            0x00 => Some(Status::Ok),
            0x01 => Some(Status::Error),
            0x02 => Some(Status::CrcMismatch),
            0x03 => Some(Status::AddrOutOfBounds),
            0x04 => Some(Status::NotReady),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cmd_from_u8_valid() {
        assert_eq!(Cmd::from_u8(0x01), Some(Cmd::Info));
        assert_eq!(Cmd::from_u8(0x05), Some(Cmd::Reset));
    }

    #[test]
    fn cmd_from_u8_invalid() {
        assert_eq!(Cmd::from_u8(0x00), None);
        assert_eq!(Cmd::from_u8(0x06), None);
        assert_eq!(Cmd::from_u8(0xFF), None);
    }
}
