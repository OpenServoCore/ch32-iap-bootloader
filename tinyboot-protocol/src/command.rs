use crate::crc::crc16;
use crate::seal;
use crate::{CRC_INIT, Cmd, HEAD, TAIL};

/// Minimum command frame size (no data): HEAD(2) + CMD(1) + LEN(1) + ADDR(2) + CRC(2) + TAIL(2) = 10
pub const MIN_FRAME_SIZE: usize = 10;

/// Maximum data payload per command frame.
pub const MAX_DATA_LEN: usize = 255;

/// Total frame size for a command with `data_len` bytes of payload.
pub fn frame_len(data_len: usize) -> usize {
    MIN_FRAME_SIZE + data_len
}

/// Serialize a command frame (host → device) into `buf`.
/// Returns the number of bytes written.
///
/// ```text
/// [HEAD_0] [HEAD_1] [CMD] [LEN] [ADDR_LO] [ADDR_HI] [DATA × LEN] [CRC_LO] [CRC_HI] [TAIL_0] [TAIL_1]
/// ```
///
/// Panics if `buf` is too small (needs `frame_len(data.len())` bytes).
pub fn build(cmd: Cmd, addr: u16, data: &[u8], buf: &mut [u8]) -> usize {
    let len = data.len();
    assert!(len <= MAX_DATA_LEN);
    assert!(buf.len() >= frame_len(len));

    buf[2] = cmd as u8;
    buf[3] = len as u8;
    buf[4] = addr as u8;
    buf[5] = (addr >> 8) as u8;
    buf[6..6 + len].copy_from_slice(data);

    seal(buf, 6 + len)
}

/// Parsed command frame.
#[derive(Debug, PartialEq)]
pub struct CommandFrame {
    pub cmd: Cmd,
    pub addr: u16,
    pub len: u8,
}

/// Result of feeding a byte into the command parser.
#[derive(Debug, PartialEq)]
pub enum ParseResult {
    /// Need more bytes.
    Need,
    /// A data byte to be consumed by the caller (pushed into ring buffer, etc).
    Data(u8),
    /// Complete valid command frame.
    Frame(CommandFrame),
    /// Frame error (bad CRC, unknown cmd, bad delimiter). Parser resets.
    Error,
}

/// Byte-at-a-time parser for command frames (host → device).
///
/// Feed bytes via `feed()`. Data bytes are returned as `ParseResult::Data`
/// so the caller can push them into a ring buffer without double-buffering.
/// When the frame is complete and CRC-valid, returns `ParseResult::Frame`.
pub struct CommandParser {
    state: CState,
    cmd: u8,
    len: u8,
    addr_lo: u8,
    addr: u16,
    data_remaining: u8,
    crc_lo: u8,
    crc: u16, // running CRC
}

#[derive(Clone, Copy)]
enum CState {
    Head0,
    Head1,
    Cmd,
    Len,
    AddrLo,
    AddrHi,
    Data,
    CrcLo,
    CrcHi,
    Tail0,
    Tail1,
}

impl Default for CommandParser {
    fn default() -> Self {
        Self {
            state: CState::Head0,
            cmd: 0,
            len: 0,
            addr_lo: 0,
            addr: 0,
            data_remaining: 0,
            crc_lo: 0,
            crc: CRC_INIT,
        }
    }
}

impl CommandParser {
    pub fn reset(&mut self) {
        self.state = CState::Head0;
        self.crc = CRC_INIT;
    }

    pub fn feed(&mut self, byte: u8) -> ParseResult {
        match self.state {
            CState::Head0 => {
                if byte == HEAD[0] {
                    self.state = CState::Head1;
                }
                ParseResult::Need
            }
            CState::Head1 => {
                if byte == HEAD[1] {
                    self.state = CState::Cmd;
                    self.crc = CRC_INIT;
                } else if byte == HEAD[0] {
                    // Could be start of a new frame; stay in Head1
                } else {
                    self.reset();
                }
                ParseResult::Need
            }
            CState::Cmd => {
                self.cmd = byte;
                self.crc = crc16(self.crc, &[byte]);
                self.state = CState::Len;
                ParseResult::Need
            }
            CState::Len => {
                self.len = byte;
                self.data_remaining = byte;
                self.crc = crc16(self.crc, &[byte]);
                self.state = CState::AddrLo;
                ParseResult::Need
            }
            CState::AddrLo => {
                self.addr_lo = byte;
                self.crc = crc16(self.crc, &[byte]);
                self.state = CState::AddrHi;
                ParseResult::Need
            }
            CState::AddrHi => {
                self.addr = u16::from_le_bytes([self.addr_lo, byte]);
                self.crc = crc16(self.crc, &[byte]);
                if self.data_remaining > 0 {
                    self.state = CState::Data;
                } else {
                    self.state = CState::CrcLo;
                }
                ParseResult::Need
            }
            CState::Data => {
                self.crc = crc16(self.crc, &[byte]);
                self.data_remaining -= 1;
                if self.data_remaining == 0 {
                    self.state = CState::CrcLo;
                }
                ParseResult::Data(byte)
            }
            CState::CrcLo => {
                self.crc_lo = byte;
                self.state = CState::CrcHi;
                ParseResult::Need
            }
            CState::CrcHi => {
                let received = u16::from_le_bytes([self.crc_lo, byte]);
                if received != self.crc {
                    self.reset();
                    return ParseResult::Error;
                }
                self.state = CState::Tail0;
                ParseResult::Need
            }
            CState::Tail0 => {
                if byte != TAIL[0] {
                    self.reset();
                    return ParseResult::Error;
                }
                self.state = CState::Tail1;
                ParseResult::Need
            }
            CState::Tail1 => {
                self.reset();
                if byte != TAIL[1] {
                    return ParseResult::Error;
                }
                match Cmd::from_u8(self.cmd) {
                    Some(cmd) => ParseResult::Frame(CommandFrame {
                        cmd,
                        addr: self.addr,
                        len: self.len,
                    }),
                    None => ParseResult::Error,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_no_data() {
        let mut buf = [0u8; MIN_FRAME_SIZE];
        let n = build(Cmd::Info, 0, &[], &mut buf);
        assert_eq!(n, MIN_FRAME_SIZE);
        assert_eq!(buf[0], HEAD[0]);
        assert_eq!(buf[1], HEAD[1]);
        assert_eq!(buf[2], 0x01); // Info
        assert_eq!(buf[3], 0x00); // LEN = 0
    }

    #[test]
    fn build_with_data() {
        let data = [0xDE, 0xAD, 0xBE, 0xEF];
        let mut buf = [0u8; MIN_FRAME_SIZE + 4];
        let n = build(Cmd::Write, 0x0400, &data, &mut buf);
        assert_eq!(n, 14);
        assert_eq!(buf[3], 4); // LEN
        assert_eq!(buf[4], 0x00); // ADDR_LO
        assert_eq!(buf[5], 0x04); // ADDR_HI
        assert_eq!(&buf[6..10], &data);
    }

    #[test]
    fn parser_round_trip_no_data() {
        let mut buf = [0u8; MIN_FRAME_SIZE];
        build(Cmd::Erase, 0, &[], &mut buf);

        let mut parser = CommandParser::default();
        let mut result = ParseResult::Need;
        for &b in &buf {
            result = parser.feed(b);
        }
        assert_eq!(
            result,
            ParseResult::Frame(CommandFrame {
                cmd: Cmd::Erase,
                addr: 0,
                len: 0,
            })
        );
    }

    #[test]
    fn parser_round_trip_with_data() {
        let data = [0x01, 0x02, 0x03, 0x04];
        let mut buf = [0u8; MIN_FRAME_SIZE + 4];
        build(Cmd::Write, 0x0800, &data, &mut buf);

        let mut parser = CommandParser::default();
        let mut received_data = [0u8; 4];
        let mut data_idx = 0;
        let mut result = ParseResult::Need;

        for &b in &buf {
            result = parser.feed(b);
            match result {
                ParseResult::Data(d) => {
                    received_data[data_idx] = d;
                    data_idx += 1;
                }
                _ => {}
            }
        }

        assert_eq!(received_data, data);
        assert_eq!(
            result,
            ParseResult::Frame(CommandFrame {
                cmd: Cmd::Write,
                addr: 0x0800,
                len: 4,
            })
        );
    }

    #[test]
    fn parser_bad_crc() {
        let mut buf = [0u8; MIN_FRAME_SIZE];
        build(Cmd::Info, 0, &[], &mut buf);
        buf[2] ^= 0xFF; // corrupt CMD byte (CRC won't match)

        let mut parser = CommandParser::default();
        let mut saw_error = false;
        for &b in &buf {
            if parser.feed(b) == ParseResult::Error {
                saw_error = true;
            }
        }
        assert!(saw_error);
    }

    #[test]
    fn parser_resyncs_after_garbage() {
        let mut buf = [0u8; MIN_FRAME_SIZE];
        build(Cmd::Verify, 0, &[], &mut buf);

        let mut parser = CommandParser::default();
        for &b in &[0xFF, 0x00, 0xAA, 0x42] {
            parser.feed(b);
        }
        let mut result = ParseResult::Need;
        for &b in &buf {
            result = parser.feed(b);
        }
        assert_eq!(
            result,
            ParseResult::Frame(CommandFrame {
                cmd: Cmd::Verify,
                addr: 0,
                len: 0,
            })
        );
    }
}
