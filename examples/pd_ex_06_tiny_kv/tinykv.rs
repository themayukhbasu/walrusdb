use crate::{error::DBError, otherblockstore::{BLOCK_SIZE, BlockStore}};

pub struct TinyKV{
    store: BlockStore,
}   

impl TinyKV {
    pub fn open(path: &str) -> Result<Self, DBError> {
        let store = BlockStore::open(path)?;
        Ok(Self { store })
    }

    pub fn put(&mut self, key: &str, value: &str) -> Result<(), DBError> {
        let (_, block_number) = self.scan_live_blocks_with_key(key)?;
        if block_number == -1 {
               let new_record =  Self::encode(1, key.len() as u16, value.len() as u16, key, value);
               let block = self.store.allocate_block()?;
               self.store.write_block(block, &new_record)?;
            }

        else {
               let (_, block_number) = self.delete(key)?;
               let new_record =  Self::encode(1, key.len() as u16, value.len() as u16, key, value);
               self.store.write_block(block_number as u64, &new_record)?;              
            }
        

        return Ok(())
    }

    pub fn delete(&mut self, key: &str) -> Result<([u8; BLOCK_SIZE], i8), DBError> {
        let (live_blocks, block_number) = self.scan_live_blocks_with_key(key)?;
        if block_number == -1 {
            return Ok((live_blocks, block_number));
        }
        let existing= Self::encode(2, key.len() as u16, 0, key, "");
        self.store.write_block(block_number as u64, &existing)?;
        Ok((live_blocks, block_number))
    }

    pub fn get(&mut self, key: &str) -> Result<Option<String>, DBError> {
        let (live_blocks, block_number) = self.scan_live_blocks_with_key(key)?;
        if block_number == -1 {
           let (dead_blocks, _) = self.scan_dead_blocks_with_key(key)?;
           let (status, _, _, _, _) = Self::decode(dead_blocks)?;
           println!("The status of the dead block is: {}", status);
           return Ok(None);
        }
        let (_, _, _, _, value) = Self::decode(live_blocks)?;
        Ok(Some(value))
    }

    fn encode(
        status: u8,
        key_len: u16,
        val_len: u16,
        key: &str,
        value: &str,
    ) -> [u8; BLOCK_SIZE] {
        let mut buf = [0u8; BLOCK_SIZE];
        buf[0] = status;
        buf[1..=2].copy_from_slice(&key_len.to_le_bytes());
        buf[3..=4].copy_from_slice(&val_len.to_le_bytes());
        buf[5..(5 + key_len as usize)].copy_from_slice(key.as_bytes());
        buf[(5 + key_len as usize)..(5 + key_len as usize + val_len as usize)]
            .copy_from_slice(value.as_bytes());
        return buf;
    }

    fn decode(buf: [u8; BLOCK_SIZE]) -> Result<(u8, u16, u16, String, String), DBError> {
        let status = buf[0];
        let key_len = u16::from_le_bytes(buf[1..=2].try_into().unwrap());
        let val_len = u16::from_le_bytes(buf[3..=4].try_into().unwrap());
        let key = String::from_utf8(buf[5..(5 + key_len as usize)].to_vec()).unwrap();
        let value = String::from_utf8(
            buf[(5 + key_len as usize)..(5 + key_len as usize + val_len as usize)].to_vec(),
        )
        .unwrap();
        return Ok((status, key_len, val_len, key, value));
    }

   fn scan_live_blocks_with_key(&mut self, key: &str) -> Result<([u8; BLOCK_SIZE], i8), DBError> {
        let mut live_blocks = [0u8; BLOCK_SIZE];
        let mut block_number = -1;
        let num_blocks = self.store.num_blocks()?;
        for block_num in 0..num_blocks {
            let block_data = self.store.read_block(block_num)?;
            let (status, _, _, block_key, _) = Self::decode(block_data)?;
            if status == 1 && block_key == key {
                println!("The status of the block is: {}", status);
                println!("The key of the block is: {}", block_key);
                live_blocks = block_data;
                block_number = block_num as i8;
                break;
            }
        }
        Ok((live_blocks, block_number))
    }

    fn scan_dead_blocks_with_key(&mut self, key: &str) -> Result<([u8; BLOCK_SIZE], i8), DBError> {
        let mut dead_blocks = [0u8; BLOCK_SIZE];
        let mut block_number = -1;
        let num_blocks = self.store.num_blocks()?;
        for block_num in 0..num_blocks {
            let block_data = self.store.read_block(block_num)?;
            let (status, _, _, block_key, _) = Self::decode(block_data)?;
            if status == 2 && block_key == key {
                println!("The status of the block is: {}", status);
                println!("The key of the block is: {}", block_key);
                dead_blocks = block_data;
                block_number = block_num as i8;
                break;
            }
        }
        Ok((dead_blocks, block_number))
    }
}