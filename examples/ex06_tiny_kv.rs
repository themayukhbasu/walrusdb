use std::fmt;
use std::fmt::Formatter;
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::macos::raw::stat;

const BLOCK_SIZE: usize = 64;

#[derive(Debug)]
enum DBError {
    Io(std::io::Error),
    BlockOutOfBounds(u64),
    InvalidRecordStatus(u8),
}

// From trait to convert std::io::Error to DBError
impl From<std::io::Error> for DBError {
    fn from(e: std::io::Error) -> Self {
        DBError::Io(e)
    }
}

// Display trait
impl fmt::Display for DBError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            DBError::Io(e) => write!(f, "I/O Error: {}", e),
            DBError::BlockOutOfBounds(n) => write!(f, "Block Out of Bounds: {}", n),
            DBError::InvalidRecordStatus(status) => write!(f, "Invalid Record Status: {}", status),
        }
    }
}

struct BlockStore {
    file: std::fs::File,
}

impl BlockStore {
    fn open(path: &str) -> Result<Self, DBError> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(path)?;
        Ok(Self { file })
    }

    fn num_blocks(&mut self) -> Result<u64, DBError> {
        let size = self.file.seek(SeekFrom::End(0))?;
        Ok(size / BLOCK_SIZE as u64)
    }

    fn allocate_block(&mut self) -> Result<u64, DBError> {
        let size = self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(&[0u8; BLOCK_SIZE])?;
        Ok((size / BLOCK_SIZE as u64) - 1)
    }

    fn validate_block_idx(&mut self, block_idx: u64) -> Result<(), DBError> {
        if block_idx >= self.num_blocks()? {
            return Err(DBError::BlockOutOfBounds(block_idx));
        }
        Ok(())
    }

    fn write(&mut self, block_idx: u64, data: [u8; BLOCK_SIZE]) -> Result<(), DBError> {
        self.validate_block_idx(block_idx)?;
        self.file
            .seek(SeekFrom::Start(block_idx * BLOCK_SIZE as u64))?;
        self.file.write_all(&data)?;
        Ok(())
    }

    fn read(&mut self, block_idx: u64) -> Result<[u8; BLOCK_SIZE], DBError> {
        self.validate_block_idx(block_idx)?;
        self.file
            .seek(SeekFrom::Start(block_idx * BLOCK_SIZE as u64))?;
        let mut buf = [0u8; BLOCK_SIZE];
        self.file.read(&mut buf)?;
        Ok(buf)
    }
}

enum RecordStatus {
    Empty = 0,
    Live = 1,
    Deleted = 2,
}

impl TryFrom<u8> for RecordStatus {
    type Error = DBError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Empty),
            1 => Ok(Self::Live),
            2 => Ok(Self::Deleted),
            status => Err(DBError::InvalidRecordStatus(status)),
        }
    }
}

struct Record {
    status: RecordStatus,
    key_len: u16,
    value_len: u16,
    key: Vec<u8>,
    value: Vec<u8>,
}

impl Record {
    // fn new()
    fn encode(&mut self) {
        todo!()
    }
}

fn main() {}
