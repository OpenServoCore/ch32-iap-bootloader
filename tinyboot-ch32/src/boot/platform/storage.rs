use embedded_storage::nor_flash::{
    ErrorType, NorFlash, NorFlashError, NorFlashErrorKind, ReadNorFlash,
};
use tinyboot::traits::Storage as StorageTrait;

use crate::common::{APP_BASE, APP_PTR, APP_SIZE, FLASH_ERASE_SIZE, FLASH_WRITE_SIZE};
use crate::hal::flash::FlashWriter;

#[derive(Debug)]
pub enum StorageError {
    NotAligned,
    OutOfBounds,
    Protected,
}

impl NorFlashError for StorageError {
    fn kind(&self) -> NorFlashErrorKind {
        match self {
            StorageError::NotAligned => NorFlashErrorKind::NotAligned,
            StorageError::OutOfBounds => NorFlashErrorKind::OutOfBounds,
            StorageError::Protected => NorFlashErrorKind::Other,
        }
    }
}

pub(crate) struct Storage {
    regs: ch32_metapac::flash::Flash,
}

impl Storage {
    pub fn new(regs: ch32_metapac::flash::Flash) -> Self {
        Storage { regs }
    }
}

impl ErrorType for Storage {
    type Error = StorageError;
}

impl NorFlash for Storage {
    const WRITE_SIZE: usize = FLASH_WRITE_SIZE;
    const ERASE_SIZE: usize = FLASH_ERASE_SIZE;

    fn erase(&mut self, from: u32, to: u32) -> Result<(), Self::Error> {
        if from as usize % FLASH_ERASE_SIZE != 0 || to as usize % FLASH_ERASE_SIZE != 0 {
            return Err(StorageError::NotAligned);
        }
        if to as usize > APP_SIZE {
            return Err(StorageError::OutOfBounds);
        }
        let writer = FlashWriter::standard(&self.regs);
        let mut addr = APP_BASE + from;
        let end = APP_BASE + to;
        while addr < end {
            writer.erase_page(addr);
            addr += FLASH_ERASE_SIZE as u32;
        }
        if writer.check_wrprterr() {
            return Err(StorageError::Protected);
        }
        Ok(())
    }

    fn write(&mut self, offset: u32, bytes: &[u8]) -> Result<(), Self::Error> {
        if offset as usize % FLASH_WRITE_SIZE != 0 || bytes.len() % FLASH_WRITE_SIZE != 0 {
            return Err(StorageError::NotAligned);
        }
        if offset as usize + bytes.len() > APP_SIZE {
            return Err(StorageError::OutOfBounds);
        }
        let writer = FlashWriter::standard(&self.regs);
        let mut addr = APP_BASE + offset;
        for pair in bytes.chunks_exact(2) {
            let halfword = u16::from_le_bytes([pair[0], pair[1]]);
            writer.write_halfword(addr, halfword);
            addr += 2;
        }
        if writer.check_wrprterr() {
            return Err(StorageError::Protected);
        }
        Ok(())
    }
}

impl StorageTrait for Storage {
    fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(APP_PTR as *const u8, APP_SIZE) }
    }
}

impl ReadNorFlash for Storage {
    const READ_SIZE: usize = 1;

    fn read(&mut self, offset: u32, bytes: &mut [u8]) -> Result<(), Self::Error> {
        if offset as usize + bytes.len() > APP_SIZE {
            return Err(StorageError::OutOfBounds);
        }
        let src = unsafe { core::slice::from_raw_parts(APP_PTR as *const u8, APP_SIZE) };
        let offset = offset as usize;
        bytes.copy_from_slice(&src[offset..offset + bytes.len()]);
        Ok(())
    }

    fn capacity(&self) -> usize {
        APP_SIZE
    }
}
