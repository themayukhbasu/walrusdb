use crate::blockstore;
use crate::errors;

use crate::record::{Record, RecordStatus};
use blockstore::BlockStore;
use errors::DBError;

pub struct TinyKV {
    store: BlockStore,
}

impl TinyKV {
    pub fn init(path: &str) -> Result<Self, DBError> {
        let store = BlockStore::open(path)?;

        Ok(Self { store })
    }

    fn read(&mut self, block_idx: u64) -> Result<Record, DBError> {
        let record = Record::decode(self.store.read(block_idx)?)?;
        Ok(record)
    }

    fn write(&mut self, block_idx: u64, record: Record) -> Result<(), DBError> {
        let record_bytes = record.encode();
        self.store.write(block_idx, record_bytes)?;
        Ok(())
    }

    fn scan_live_key(&mut self, key: &str) -> Result<(Option<Record>, Option<u64>), DBError> {
        // Returns record and block index if `Ok`

        let db_size = self.store.num_blocks()?;
        for i in 0..db_size {
            let record: Record = self.read(i)?;
            if record.status == RecordStatus::Live && record.key == key {
                return Ok((Some(record), Some(i)));
            }
        }
        return Ok((None, None));
    }

    fn scan_empty(&mut self) -> Result<Option<u64>, DBError> {
        // Returns index of first empty block
        let db_size = self.store.num_blocks()?;
        for i in 0..db_size {
            let record: Record = self.read(i)?;
            if record.status == RecordStatus::Empty {
                return Ok(Some(i));
            }
        }
        return Ok(None);
    }

    fn next_empty(&mut self) -> Result<u64, DBError> {
        let some_block_idx = self.scan_empty()?;

        let block_idx = match some_block_idx {
            Some(idx) => idx,
            None => {
                let idx = self.store.allocate_block()?;
                idx
            }
        };
        Ok(block_idx)
    }

    pub fn put(&mut self, key: &str, value: &str) -> Result<(), DBError> {
        let new_record = Record::new(RecordStatus::Live, key, value)?;

        let (some_existing_record, some_block_idx) = self.scan_live_key(key)?;
        match (some_existing_record, some_block_idx) {
            (None, None) => {
                // no live key

                let block_idx = self.next_empty()?;
                self.write(block_idx, new_record)?;
                Ok(())
            }
            (Some(existing_record), Some(block_idx)) => {
                // live record exist

                if existing_record.size() == new_record.size() {
                    self.write(block_idx, new_record)?;
                } else {
                    self.delete_idx(block_idx, existing_record)?;
                    let idx = self.next_empty()?;
                    self.write(idx, new_record)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    pub fn get(&mut self, key: &str) -> Result<Option<String>, DBError> {
        let (some_record, _) = self.scan_live_key(key)?;

        match some_record {
            None => Ok(None),
            Some(record) => Ok(Some(record.value)),
        }
    }

    fn delete_idx(&mut self, block_idx: u64, mut existing_record: Record) -> Result<(), DBError> {
        existing_record.status = RecordStatus::Deleted;
        self.write(block_idx, existing_record)?;
        return Ok(());
    }
    pub fn delete(&mut self, key: &str) -> Result<(), DBError> {
        let (some_existing_record, some_block_idx) = self.scan_live_key(key)?;
        match (some_existing_record, some_block_idx) {
            (None, None) => Ok(()),
            (Some(existing_record), Some(block_idx)) => self.delete_idx(block_idx, existing_record),
            _ => Ok(()),
        }
    }

    pub fn dump(&mut self) -> Result<Vec<(u64, Record)>, DBError> {
        let db_size = self.store.num_blocks()?;
        println!("DB Size: {}", db_size);
        let mut records = vec![];
        for i in 0..db_size {
            let record = Record::decode(self.store.read(i)?)?;
            records.push((i, record));
        }
        Ok(records)
    }
}
