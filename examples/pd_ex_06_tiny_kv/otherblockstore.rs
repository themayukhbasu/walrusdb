use std::fs::OpenOptions;
use crate::error::DBError;
use std::io::{Seek, SeekFrom, Read, Write};

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
        let current = self.file.seek(SeekFrom::Current(0))?;
        let size = self.file.seek(SeekFrom::End(0))?;
        self.file.seek(SeekFrom::Start(current))?;  
        Ok(size / BLOCK_SIZE as u64)
    }

    pub fn allocate_block(&mut self) -> Result<u64, DBError> {
        let num_blocks = self.num_blocks()?;
        let _size = self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(&[0u8; BLOCK_SIZE])?;
        Ok(num_blocks)
    }

    pub fn write_block(&mut self, block_num: u64, data: &[u8; BLOCK_SIZE]) -> Result<(), DBError> {
        if block_num >= self.num_blocks()? {
            return Err(DBError::BlockOutOfBounds(block_num));
        }
        let offset = block_num * BLOCK_SIZE as u64;
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(data)?;
        Ok(())
    }

    pub fn read_block(&mut self, block_num: u64) -> Result<[u8; BLOCK_SIZE], DBError> {
        if block_num >= self.num_blocks()? {
            return Err(DBError::BlockOutOfBounds(block_num));
        }
        let offset = block_num * BLOCK_SIZE as u64;
        self.file.seek(SeekFrom::Start(offset))?;
        let mut buffer = [0u8; BLOCK_SIZE];
        self.file.read_exact(&mut buffer)?;
        Ok(buffer)
    }
}