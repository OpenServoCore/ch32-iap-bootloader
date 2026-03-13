use tinyboot_protocol::crc::crc16;
use tinyboot_protocol::{Cmd, CRC_INIT, HEAD, Status};

use crate::traits::{BootCtl, BootMetaStore, Storage, Transport};

/// Send a response frame, with optional extra payload after the status byte.
///
/// Layout: `[HEAD] [CMD] [STATUS] [extra...] [CRC]`
#[inline(never)]
fn send_response<T: embedded_io::Write>(
    transport: &mut T,
    cmd: Cmd,
    status: Status,
    extra: &[u8],
) -> Result<(), T::Error> {
    let cmd_status = [cmd as u8, status as u8];
    let mut crc = crc16(CRC_INIT, &cmd_status);
    crc = crc16(crc, extra);
    transport.write_all(&HEAD)?;
    transport.write_all(&cmd_status)?;
    transport.write_all(extra)?;
    transport.write_all(&crc.to_le_bytes())
}

/// Handle INFO command: report device geometry.
///
/// Extended response payload: `[write_size: u16 LE] [app_size: u16 LE] [payload_size: u16 LE]`
#[inline(never)]
pub fn handle_info<T: Transport, S: Storage>(
    transport: &mut T,
    storage: &S,
) -> Result<(), T::Error> {
    let ws = (S::WRITE_SIZE as u16).to_le_bytes();
    let cap = (storage.capacity() as u16).to_le_bytes();
    send_response(
        transport,
        Cmd::Info,
        Status::Ok,
        &[ws[0], ws[1], cap[0], cap[1], ws[0], ws[1]],
    )
}

/// Handle ERASE command: erase entire app region.
#[inline(never)]
pub fn handle_erase<T: Transport, S: Storage>(
    transport: &mut T,
    storage: &mut S,
) -> Result<(), T::Error> {
    let capacity = storage.capacity() as u32;
    let status = match storage.erase(0, capacity) {
        Ok(()) => Status::Ok,
        Err(_) => Status::Error,
    };
    send_response(transport, Cmd::Erase, status, &[])
}

/// Handle WRITE command: write data at given address.
#[inline(never)]
pub fn handle_write<T: Transport, S: Storage>(
    transport: &mut T,
    storage: &mut S,
    addr: u32,
    data: &[u8],
) -> Result<(), T::Error> {
    let capacity = storage.capacity() as u32;

    if addr >= capacity || addr + data.len() as u32 > capacity {
        return send_response(transport, Cmd::Write, Status::AddrOutOfBounds, &[]);
    }

    if addr as usize % S::WRITE_SIZE != 0 {
        return send_response(transport, Cmd::Write, Status::AddrOutOfBounds, &[]);
    }

    let status = match storage.write(addr, data) {
        Ok(()) => Status::Ok,
        Err(_) => Status::Error,
    };
    send_response(transport, Cmd::Write, status, &[])
}

/// Handle VERIFY command: compute CRC16 over entire app region.
#[inline(never)]
pub fn handle_verify<T: Transport, S: Storage>(
    transport: &mut T,
    storage: &S,
) -> Result<(), T::Error> {
    let crc = crc16(CRC_INIT, storage.as_slice());
    send_response(transport, Cmd::Verify, Status::Ok, &crc.to_le_bytes())
}

/// Handle RESET command: advance boot state and reset system.
#[inline(never)]
pub fn handle_reset<T: Transport, B: BootMetaStore, C: BootCtl>(
    transport: &mut T,
    boot_meta: &mut B,
    ctl: &mut C,
) -> ! {
    let _ = boot_meta.advance();
    let _ = send_response(transport, Cmd::Reset, Status::Ok, &[]);
    ctl.system_reset()
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_storage::nor_flash;
    use tinyboot_protocol::crc::crc16;
    use tinyboot_protocol::CRC_INIT;

    // -- Mock transport --

    struct Sink {
        buf: [u8; 64],
        pos: usize,
    }

    impl Sink {
        fn new() -> Self {
            Self {
                buf: [0; 64],
                pos: 0,
            }
        }
        fn written(&self) -> &[u8] {
            &self.buf[..self.pos]
        }
    }

    impl embedded_io::ErrorType for Sink {
        type Error = core::convert::Infallible;
    }

    impl embedded_io::Read for Sink {
        fn read(&mut self, _buf: &mut [u8]) -> Result<usize, Self::Error> {
            Ok(0)
        }
    }

    impl embedded_io::Write for Sink {
        fn write(&mut self, buf: &[u8]) -> Result<usize, Self::Error> {
            let n = buf.len().min(self.buf.len() - self.pos);
            self.buf[self.pos..self.pos + n].copy_from_slice(&buf[..n]);
            self.pos += n;
            Ok(n)
        }
        fn flush(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    // -- Mock storage --

    struct MockStorage {
        data: [u8; 256],
    }

    impl MockStorage {
        fn new() -> Self {
            Self { data: [0xFF; 256] }
        }
    }

    impl crate::traits::Storage for MockStorage {
        fn as_slice(&self) -> &[u8] {
            &self.data
        }
    }

    impl nor_flash::ErrorType for MockStorage {
        type Error = nor_flash::NorFlashErrorKind;
    }

    impl nor_flash::ReadNorFlash for MockStorage {
        const READ_SIZE: usize = 1;

        fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
            let start = offset as usize;
            let end = start + bytes.len();
            if end > self.data.len() {
                return Err(nor_flash::NorFlashErrorKind::OutOfBounds);
            }
            bytes.copy_from_slice(&self.data[start..end]);
            Ok(())
        }

        fn capacity(&self) -> usize {
            self.data.len()
        }
    }

    impl nor_flash::NorFlash for MockStorage {
        const WRITE_SIZE: usize = 4;
        const ERASE_SIZE: usize = 256;

        fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
            self.data[from as usize..to as usize].fill(0xFF);
            Ok(())
        }

        fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
            let start = offset as usize;
            let end = start + bytes.len();
            if end > self.data.len() {
                return Err(nor_flash::NorFlashErrorKind::OutOfBounds);
            }
            self.data[start..end].copy_from_slice(bytes);
            Ok(())
        }
    }

    // -- Tests --

    #[test]
    fn response_frame_format() {
        let mut sink = Sink::new();
        send_response(&mut sink, Cmd::Info, Status::Ok, &[]).unwrap();
        let buf = sink.written();

        assert_eq!(buf.len(), 6);
        assert_eq!(buf[0], 0xAA);
        assert_eq!(buf[1], 0x55);
        assert_eq!(buf[2], 0x01);
        assert_eq!(buf[3], 0x00);
        let expected_crc = crc16(CRC_INIT, &[0x01, 0x00]);
        assert_eq!(buf[4], expected_crc as u8);
        assert_eq!(buf[5], (expected_crc >> 8) as u8);
    }

    #[test]
    fn info_reports_geometry() {
        let mut sink = Sink::new();
        let storage = MockStorage::new();
        handle_info(&mut sink, &storage).unwrap();
        let buf = sink.written();

        // HEAD(2) + CMD(1) + STATUS(1) + 3×u16(6) + CRC(2) = 12
        assert_eq!(buf.len(), 12);
        assert_eq!(buf[2], Cmd::Info as u8);
        assert_eq!(buf[3], Status::Ok as u8);

        let write_size = u16::from_le_bytes([buf[4], buf[5]]);
        let app_size = u16::from_le_bytes([buf[6], buf[7]]);
        let payload_size = u16::from_le_bytes([buf[8], buf[9]]);

        assert_eq!(write_size, 4);
        assert_eq!(app_size, 256);
        assert_eq!(payload_size, 4);

        // Verify frame CRC
        let expected_crc = crc16(CRC_INIT, &buf[2..10]);
        assert_eq!(buf[10], expected_crc as u8);
        assert_eq!(buf[11], (expected_crc >> 8) as u8);
    }

    #[test]
    fn erase_clears_storage() {
        let mut sink = Sink::new();
        let mut storage = MockStorage::new();
        storage.data[0] = 0x42;

        handle_erase(&mut sink, &mut storage).unwrap();

        assert_eq!(storage.data[0], 0xFF);
        let buf = sink.written();
        assert_eq!(buf[2], Cmd::Erase as u8);
        assert_eq!(buf[3], Status::Ok as u8);
    }

    #[test]
    fn write_stores_data() {
        let mut sink = Sink::new();
        let mut storage = MockStorage::new();
        let data = [0xDE, 0xAD, 0xBE, 0xEF];

        handle_write(&mut sink, &mut storage, 0, &data).unwrap();

        assert_eq!(&storage.data[..4], &data);
        let buf = sink.written();
        assert_eq!(buf[2], Cmd::Write as u8);
        assert_eq!(buf[3], Status::Ok as u8);
    }

    #[test]
    fn write_at_offset() {
        let mut sink = Sink::new();
        let mut storage = MockStorage::new();
        let data = [0x01, 0x02, 0x03, 0x04];

        handle_write(&mut sink, &mut storage, 8, &data).unwrap();

        assert_eq!(&storage.data[8..12], &data);
        assert_eq!(sink.written()[3], Status::Ok as u8);
    }

    #[test]
    fn write_out_of_bounds() {
        let mut sink = Sink::new();
        let mut storage = MockStorage::new();

        handle_write(&mut sink, &mut storage, 256, &[0x01, 0x02, 0x03, 0x04]).unwrap();

        assert_eq!(sink.written()[3], Status::AddrOutOfBounds as u8);
    }

    #[test]
    fn write_unaligned_addr() {
        let mut sink = Sink::new();
        let mut storage = MockStorage::new();

        handle_write(&mut sink, &mut storage, 1, &[0x01, 0x02, 0x03, 0x04]).unwrap();

        assert_eq!(sink.written()[3], Status::AddrOutOfBounds as u8);
    }

    #[test]
    fn verify_computes_crc() {
        let mut sink = Sink::new();
        let mut storage = MockStorage::new();
        storage.data[..4].copy_from_slice(&[0x01, 0x02, 0x03, 0x04]);

        handle_verify(&mut sink, &mut storage).unwrap();
        let buf = sink.written();

        // HEAD(2) + CMD(1) + STATUS(1) + app_crc(2) + CRC(2) = 8
        assert_eq!(buf.len(), 8);
        assert_eq!(buf[2], Cmd::Verify as u8);
        assert_eq!(buf[3], Status::Ok as u8);

        let expected_app_crc = crc16(CRC_INIT, &storage.data);
        assert_eq!(buf[4], expected_app_crc as u8);
        assert_eq!(buf[5], (expected_app_crc >> 8) as u8);
    }

    #[test]
    fn verify_erased_flash() {
        let mut sink = Sink::new();
        let mut storage = MockStorage::new();

        handle_verify(&mut sink, &mut storage).unwrap();
        let buf = sink.written();

        let expected_crc = crc16(CRC_INIT, &[0xFF; 256]);
        assert_eq!(buf[4], expected_crc as u8);
        assert_eq!(buf[5], (expected_crc >> 8) as u8);
    }
}
