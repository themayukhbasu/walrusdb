use std::fmt;
use std::fmt::{Formatter};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};

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
            DBError::InvalidPage(n) => write!(f, "Invalid Page | Meaning of Life: {}", n),
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
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        Ok(Self { file })
    }

    fn is_valid_block_num(&mut self, n: u64) -> Result<(), DBError> {
        if n == 42 {
            // this is a joke; don't use it in your own pager
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

fn main() -> Result<(), DBError> {
    let path_str = "target/blockstore.bin";
    let mut store = BlockStore::open(path_str)?;
    let n1 = store.allocate_block()?;
    let n2 = store.allocate_block()?;
    let n3 = store.allocate_block()?;

    store.write_block(n3, [3u8; BLOCK_SIZE])?;
    store.write_block(n1, [1u8;BLOCK_SIZE])?;
    store.write_block(n2, [2u8; BLOCK_SIZE])?;

    drop(store);

    let mut store = BlockStore::open(path_str)?;
    let read_buf = store.read_block(n1)?;
    assert_eq!(read_buf, [1u8;BLOCK_SIZE]);
    println!("{:?}", read_buf);
    Ok(())
}


#[cfg(test)]
mod tests {
    use std::path::Path;
    use super::*;

    #[test]
    fn new_store_has_zero_blocks() {
        let path_str = "target/test_blockstore_1.bin";
        let path = Path::new(path_str);
        let _ = std::fs::remove_file(path); // clean slate

        let mut store = BlockStore::open(path_str).unwrap();

        assert_eq!(store.num_blocks().unwrap(), 0u64);

        let _ = std::fs::remove_file(path); // clean slate
    }

    #[test]
    fn allocate_increases_block_count() {
        let path_str = "target/test_blockstore_2.bin";
        let path = Path::new(path_str);
        let _ = std::fs::remove_file(path); // clean slate

        let mut store = BlockStore::open(path_str).unwrap();
        let _ =store.allocate_block();
        assert_eq!(store.num_blocks().unwrap(), 1u64);

        let _ = std::fs::remove_file(path); // clean slate
    }

    #[test]
    fn write_and_read_block_roundtrip() {
        let path_str = "target/test_blockstore_3.bin";
        let path = Path::new(path_str);
        let _ = std::fs::remove_file(path); // clean slate

        let buf = [1u8; 64];

        let mut store = BlockStore::open(path_str).unwrap();
        let _ = store.allocate_block();
        let _ = store.write_block(0, buf);
        let read_buf = store.read_block(0).unwrap();

        assert_eq!(read_buf, buf);

        let _ = std::fs::remove_file(path); // clean slate
    }

    #[test]
    fn read_invalid_block_returns_err() {
        let path_str = "target/test_blockstore_4.bin";
        let path = Path::new(path_str);
        let _ = std::fs::remove_file(path); // clean slate

        let mut store = BlockStore::open(path_str).unwrap();
        assert!(store.read_block(0).is_err());

        let _ = store.allocate_block();
        assert!(store.read_block(0).is_ok());
        assert!(store.read_block(123).is_err());
        assert!(matches!(store.read_block(123), Err(DBError::BlockOutOfBounds(..))));
        assert!(matches!(store.read_block(42), Err(DBError::InvalidPage(..))));

        let _ = std::fs::remove_file(path); // clean slate
    }

    #[test]
    fn data_persists_across_reopen() {
        let path_str = "target/test_blockstore_5.bin";
        let path = Path::new(path_str);
        let _ = std::fs::remove_file(path); // clean slate

        let mut store = BlockStore::open(path_str).unwrap();
        let _ = store.allocate_block();
        let buf = [2u8; 64];
        let _ = store.write_block(0, buf);

        drop(store);

        let mut new_store = BlockStore::open(path_str).unwrap();
        let read_buf = new_store.read_block(0).unwrap();

        assert_eq!(read_buf, buf);

        let _ = std::fs::remove_file(path); // clean slate
    }
}