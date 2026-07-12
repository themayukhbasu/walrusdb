use std::io::{Seek, SeekFrom, Read, Write};
use std::fs::{OpenOptions};

const FILES_DIR: &str = "./target/Files/block_store.bin";
const BLOCK_SIZE: usize = 64;

struct BlockStore {
    file: std::fs::File,
}

#[derive(Debug)]
enum DBError {
    IO(std::io::Error),
    BlockOutOfBounds(u64),
    LengthMismatch(String),
}

impl From<std::io::Error> for DBError {
    fn from(e: std::io::Error) -> Self {
        DBError::IO(e)
    }
}

impl std::fmt::Display for DBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DBError::IO(e) => write!(f, "IO error: {}", e),
            DBError::BlockOutOfBounds(n) => write!(f, "Block out of bounds: {}", n),
            DBError::LengthMismatch(s) => write!(f, "Input Length is more than Block Size: {}", s),
        }
    }
}


impl BlockStore {
    fn open(path: &str) -> Result<Self, DBError>{
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .map_err(|e| DBError::IO(e))?;
        return Ok(BlockStore { file })
    }

    fn num_of_blocks(&mut self) -> Result<u64, DBError>{
        let size = self.file.seek(SeekFrom::End(0))?;
        Ok(size / BLOCK_SIZE as u64)
    }

    fn allocate_block(&mut self) -> Result<u64, DBError>{
        let block_num = self.num_of_blocks()?;
        let zero_block = [0u8; BLOCK_SIZE];
        self.file.write_all(&zero_block)?;
        Ok(block_num)
    }

    fn write_block(&mut self, block_num: u64, data: &[u8; BLOCK_SIZE]) -> Result<(), DBError>{
        if block_num >= self.num_of_blocks()? {
            return Err(DBError::BlockOutOfBounds(block_num));
        }

        let offset = block_num * BLOCK_SIZE as u64;
        self.file.seek(SeekFrom::Start(offset))?;   
        self.file.write_all(data)?;
        Ok(())
    }

    fn read_block(&mut self, block_num: u64) -> Result<[u8; BLOCK_SIZE], DBError> {
        if block_num >= self.num_of_blocks()? {
            return Err(DBError::BlockOutOfBounds(block_num));
        }

        let offset = block_num * BLOCK_SIZE as u64;
        self.file.seek(SeekFrom::Start(offset))?;
        let mut buffer = [0u8; BLOCK_SIZE];
        self.file.read_exact(&mut buffer)?;
        return Ok(buffer);
    }

    
}

fn main() -> Result<(), DBError> {
    println!("Step 1: Open BlockStore");
    let mut block_store =   BlockStore::open(FILES_DIR)?;
    let first_block = block_store.allocate_block()?;
    let second_block = block_store.allocate_block()?;
    let third_block = block_store.allocate_block()?;

    println!("Allocated blocks: {}, {}, {}", first_block, second_block, third_block);

    read_input_from_user_and_write_to_file(&mut block_store, first_block)?;
    read_input_from_user_and_write_to_file(&mut block_store, second_block)?;
    read_input_from_user_and_write_to_file(&mut block_store, third_block)?;

    drop(block_store);

    let mut reopened = BlockStore::open(FILES_DIR)?;

    let _first_block = reopened.read_block(first_block)?;
    let _second_block = reopened.read_block(second_block)?;
    let _third_block = reopened.read_block(third_block)?;   
    println!("Read blocks: {:?}, {:?}, {:?}", String::from_utf8_lossy(&_first_block), String::from_utf8_lossy(&_second_block), String::from_utf8_lossy(&_third_block));
    Ok(())
    
}

fn read_input_from_user_and_write_to_file(block_store: &mut BlockStore, block_number: u64) -> Result<(), DBError> {
    let mut file_input = String::new();
     println!("Enter the file input");
     std::io::stdin().read_line(&mut file_input).expect("Failed to read line");
     let input_bytes = file_input.as_bytes();
     if input_bytes.len() > BLOCK_SIZE {
        println!("Input exceeds block size of {} bytes", BLOCK_SIZE);
        return Err(DBError:: LengthMismatch(format!("Input length: {}, Block size: {}", input_bytes.len(), BLOCK_SIZE)));
    }
    let mut block_data = [0u8; BLOCK_SIZE];
    block_data[..input_bytes.len()].copy_from_slice(input_bytes);
    block_store.write_block(block_number, &block_data)?;

    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn new_store_has_zero_blocks() {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("block_store_new_store_has_zero_blocks_{}.bin", nanos));
        let _ = fs::remove_file(&path);

        let mut store = BlockStore::open(path.to_str().unwrap()).unwrap();
        assert_eq!(store.num_of_blocks().unwrap(), 0);

        drop(store);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn allocate_increases_block_count() {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("block_store_allocate_increases_block_count_{}.bin", nanos));
        let _ = fs::remove_file(&path);

        let mut store = BlockStore::open(path.to_str().unwrap()).unwrap();
        assert_eq!(store.num_of_blocks().unwrap(), 0);

        let block_num = store.allocate_block().unwrap();
        assert_eq!(block_num, 0);
        assert_eq!(store.num_of_blocks().unwrap(), 1);

        drop(store);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn write_and_read_block_roundtrip() {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("block_store_write_and_read_block_roundtrip_{}.bin", nanos));
        let _ = fs::remove_file(&path);

        let mut store = BlockStore::open(path.to_str().unwrap()).unwrap();
        let block_num = store.allocate_block().unwrap();

        let mut data = [0u8; BLOCK_SIZE];
        let msg = b"hello block store";
        data[..msg.len()].copy_from_slice(msg);

        store.write_block(block_num, &data).unwrap();
        let read_back = store.read_block(block_num).unwrap();

        assert_eq!(read_back, data);

        drop(store);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn read_invalid_block_returns_err() {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("block_store_read_invalid_block_returns_err_{}.bin", nanos));
        let _ = fs::remove_file(&path);

        let mut store = BlockStore::open(path.to_str().unwrap()).unwrap();

        let result = store.read_block(0);
        assert!(result.is_err());

        drop(store);
        let _ = fs::remove_file(&path);
    }

    #[test]
    fn data_persists_across_reopen() {
        let mut path = std::env::temp_dir();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("block_store_data_persists_across_reopen_{}.bin", nanos));
        let _ = fs::remove_file(&path);

        {
            let mut store = BlockStore::open(path.to_str().unwrap()).unwrap();
            let block_num = store.allocate_block().unwrap();

            let mut data = [0u8; BLOCK_SIZE];
            let msg = b"persistent data";
            data[..msg.len()].copy_from_slice(msg);

            store.write_block(block_num, &data).unwrap();
            store.file.sync_all().unwrap();
        }

        let mut reopened = BlockStore::open(path.to_str().unwrap()).unwrap();
        let read_back = reopened.read_block(0).unwrap();

        let end = read_back.iter().position(|&b| b == 0).unwrap_or(BLOCK_SIZE);
        assert_eq!(&read_back[..end], b"persistent data");

        drop(reopened);
        let _ = fs::remove_file(&path);
    }
}