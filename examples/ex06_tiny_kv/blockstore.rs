use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use crate::errors::DBError;

pub const BLOCK_SIZE: usize = 64;

pub struct BlockStore {
    file: std::fs::File,
}

impl BlockStore {
    pub fn open(path: &str) -> Result<Self, DBError> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(path)?;
        Ok(Self { file })
    }

    pub fn num_blocks(&mut self) -> Result<u64, DBError> {
        let size = self.file.seek(SeekFrom::End(0))?;
        Ok(size / BLOCK_SIZE as u64)
    }

    pub fn allocate_block(&mut self) -> Result<u64, DBError> {
        let size = self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(&[0u8; BLOCK_SIZE])?;
        Ok(size / BLOCK_SIZE as u64)
    }

    fn validate_block_idx(&mut self, block_idx: u64) -> Result<(), DBError> {
        if block_idx >= self.num_blocks()? {
            return Err(DBError::BlockOutOfBounds(block_idx));
        }
        Ok(())
    }

    pub fn write(&mut self, block_idx: u64, data: [u8; BLOCK_SIZE]) -> Result<(), DBError> {
        self.validate_block_idx(block_idx)?;
        self.file
            .seek(SeekFrom::Start(block_idx * BLOCK_SIZE as u64))?;
        self.file.write_all(&data)?;
        Ok(())
    }

    pub fn read(&mut self, block_idx: u64) -> Result<[u8; BLOCK_SIZE], DBError> {
        self.validate_block_idx(block_idx)?;
        self.file
            .seek(SeekFrom::Start(block_idx * BLOCK_SIZE as u64))?;
        let mut buf = [0u8; BLOCK_SIZE];
        self.file.read(&mut buf)?;
        Ok(buf)
    }
}