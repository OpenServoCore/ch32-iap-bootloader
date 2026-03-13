use crate::crc::crc16;
use crate::seal;
use crate::{CRC_INIT, Cmd, HEAD, Status};

/// Response frame size (fixed): HEAD(2) + CMD(1) + STATUS(1) + CRC(2) = 6
pub const FRAME_SIZE: usize = 6;

/// Serialize a response frame (device → host) into `buf`.
///
/// ```text
/// [HEAD_0] [HEAD_1] [CMD] [STATUS] [CRC_LO] [CRC_HI]
/// ```
pub fn build(cmd: Cmd, status: Status, buf: &mut [u8; FRAME_SIZE]) {
    buf[2] = cmd as u8;
    buf[3] = status as u8;
    seal(buf, 4);
}

/// Result of feeding a byte into the response parser.
#[derive(Debug, PartialEq)]
pub enum ParseResult {
    /// Need more bytes.
    Need,
    /// Complete valid response frame.
    Frame(Cmd, Status),
    /// Frame error (bad CRC, unknown cmd, bad delimiter). Parser resets.
    Error,
}

/// Byte-at-a-time parser for response frames (device → host).
///
/// Feed bytes via `feed()`. When a complete valid frame arrives,
/// returns `ParseResult::Frame`. On error, returns `ParseResult::Error`
/// and automatically resets for the next frame.
pub struct ResponseParser {
    state: RState,
    cmd: u8,
    status: u8,
    crc_lo: u8,
}

#[derive(Clone, Copy)]
enum RState {
    Head0,
    Head1,
    Cmd,
    Status,
    CrcLo,
    CrcHi,
}

impl Default for ResponseParser {
    fn default() -> Self {
        Self {
            state: RState::Head0,
            cmd: 0,
            status: 0,
            crc_lo: 0,
        }
    }
}

impl ResponseParser {

    pub fn reset(&mut self) {
        self.state = RState::Head0;
    }

    pub fn feed(&mut self, byte: u8) -> ParseResult {
        match self.state {
            RState::Head0 => {
                if byte == HEAD[0] {
                    self.state = RState::Head1;
                }
                ParseResult::Need
            }
            RState::Head1 => {
                if byte == HEAD[1] {
                    self.state = RState::Cmd;
                } else if byte == HEAD[0] {
                    // Could be start of a new frame; stay in Head1
                } else {
                    self.reset();
                }
                ParseResult::Need
            }
            RState::Cmd => {
                self.cmd = byte;
                self.state = RState::Status;
                ParseResult::Need
            }
            RState::Status => {
                self.status = byte;
                self.state = RState::CrcLo;
                ParseResult::Need
            }
            RState::CrcLo => {
                self.crc_lo = byte;
                self.state = RState::CrcHi;
                ParseResult::Need
            }
            RState::CrcHi => {
                self.reset();
                let received = u16::from_le_bytes([self.crc_lo, byte]);
                let expected = crc16(CRC_INIT, &[self.cmd, self.status]);
                if received != expected {
                    return ParseResult::Error;
                }
                match (Cmd::from_u8(self.cmd), Status::from_u8(self.status)) {
                    (Some(cmd), Some(status)) => ParseResult::Frame(cmd, status),
                    _ => ParseResult::Error,
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_round_trip() {
        let mut buf = [0u8; FRAME_SIZE];
        build(Cmd::Info, Status::Ok, &mut buf);

        let mut parser = ResponseParser::default();
        let mut result = ParseResult::Need;
        for &b in &buf {
            result = parser.feed(b);
        }
        assert_eq!(result, ParseResult::Frame(Cmd::Info, Status::Ok));
    }

    #[test]
    fn parser_bad_crc() {
        let mut buf = [0u8; FRAME_SIZE];
        build(Cmd::Erase, Status::Ok, &mut buf);
        buf[4] ^= 0xFF; // corrupt CRC_LO

        let mut parser = ResponseParser::default();
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
        let mut buf = [0u8; FRAME_SIZE];
        build(Cmd::Reset, Status::Error, &mut buf);

        let mut parser = ResponseParser::default();
        for &b in &[0xFF, 0x00, 0x42] {
            parser.feed(b);
        }
        let mut result = ParseResult::Need;
        for &b in &buf {
            result = parser.feed(b);
        }
        assert_eq!(result, ParseResult::Frame(Cmd::Reset, Status::Error));
    }

    #[test]
    fn parser_resyncs_after_error() {
        let mut bad = [0u8; FRAME_SIZE];
        build(Cmd::Erase, Status::Ok, &mut bad);
        bad[3] ^= 0xFF; // corrupt status (CRC won't match)

        let mut good = [0u8; FRAME_SIZE];
        build(Cmd::Write, Status::AddrOutOfBounds, &mut good);

        // Concatenate bad + good and feed all bytes — parser should
        // recover and eventually deliver the good frame.
        let mut parser = ResponseParser::default();
        let mut got_frame = false;
        for &b in bad.iter().chain(good.iter()) {
            if let ParseResult::Frame(cmd, status) = parser.feed(b) {
                assert_eq!(cmd, Cmd::Write);
                assert_eq!(status, Status::AddrOutOfBounds);
                got_frame = true;
            }
        }
        assert!(got_frame);
    }
}
