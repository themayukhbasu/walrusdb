use std::fmt;
use std::fmt::{Formatter, write};
use std::fs::OpenOptions;
use std::io::{Error, Read, Seek, SeekFrom, Write};

const BLOCK_SIZE: usize = 64;

#[derive(Debug)]
enum DBError {
    Io(std::io::Error),
    InvalidPage(u64),
    BlockOutOfBounds(u64),
}

impl fmt::Display for DBError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DBError::Io(e) => write!(f, "I/O Error: {}", e),
            DBError::InvalidPage(n) => write!(f, "Invalid Page: {}", n),
            DBError::BlockOutOfBounds(n) => write!(f, "Block Out of Bound: {}", n),
        }
    }
}

impl From<std::io::Error> for DBError {
    fn from(e: std::io::Error) -> Self {
        DBError::Io(e)
    }
}

struct BlockStore {
    file: std::fs::File,
}

impl BlockStore {
    fn open(path: &str) -> Result<Self, DBError> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        Ok(Self { file })
    }

    fn is_valid_block_num(&mut self, n: u64) -> Result<(), DBError> {
        if n < 0 {
            return Err(DBError::InvalidPage(n));
        }
        if n >= self.num_blocks()? {
            return Err(DBError::BlockOutOfBounds(n));
        }

        Ok(())
    }

    fn read_block(&mut self, n: u64) -> Result<[u8; BLOCK_SIZE], DBError> {
        self.is_valid_block_num(n)?;
        let offset = n * BLOCK_SIZE as u64;
        self.file.seek(SeekFrom::Start(offset))?;
        let mut buf = [0u8; BLOCK_SIZE];
        self.file.read(&mut buf)?;
        Ok(buf)
    }

    fn write_block(&mut self, n: u64, data: [u8; BLOCK_SIZE]) -> Result<(), DBError> {
        self.is_valid_block_num(n)?;

        let offset = n * BLOCK_SIZE as u64;

        self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(&data)?;

        Ok(())
    }

    fn allocate_block(&mut self) -> Result<u64, DBError> {
        let empty_buf = [0u8; BLOCK_SIZE];

        let size = self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(&empty_buf)?;

        Ok(size / BLOCK_SIZE as u64)
    }

    fn num_blocks(&mut self) -> Result<u64, DBError> {
        let size = self.file.seek(SeekFrom::End(0))?;
        Ok(size / BLOCK_SIZE as u64)
    }
}

fn main() {}
