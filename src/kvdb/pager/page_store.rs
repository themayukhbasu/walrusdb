use crate::kvdb::errors::DBError;
use crate::kvdb::pager::page::PAGE_SIZE;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};

pub struct PageStore {
    file: std::fs::File,
}

impl PageStore {
    pub fn open(path: &str) -> Result<Self, DBError> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(path)?;
        Ok(Self { file })
    }
    pub fn hello() -> String {
        "Hello from PageStore".to_string()
    }

    pub fn write(&mut self, offset: u64, data: [u8; PAGE_SIZE]) -> Result<(), DBError> {
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(&data)?;
        Ok(())
    }

    pub fn read(&mut self, offset: u64, buf: &mut [u8; PAGE_SIZE]) -> Result<(), DBError> {
        self.file.seek(SeekFrom::Start(offset))?;
        self.file.read(buf)?;
        Ok(())
    }

    pub fn allocate(&mut self) -> Result<u64, DBError> {
        let start_offset = self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(&[0u8; PAGE_SIZE])?;
        Ok(start_offset)
    }

    pub fn size(&mut self) -> Result<u64, DBError> {
        let size = self.file.seek(SeekFrom::End(0))?;
        Ok(size)
    }
}
