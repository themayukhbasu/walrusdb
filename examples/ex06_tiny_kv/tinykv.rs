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
                return Ok((Some(i)));
            }
        }
        return Ok((None));
    }

    pub fn put(&mut self, key: &str, value: &str) -> Result<(), DBError> {
        let (some_record, some_block_idx) = self.scan_live_key(key)?;
        match (some_record, some_block_idx) {
            (None, None) => {
                // no live key

                // scan empty
                //      if no empty
                //      allocate
                // write to empty
            }
            (Some(record), Some(block_idx)) => {
                // if record size is same
                //      overwrite record
                // else if record size not same
                //      delete
                //      scan empty
                //          if no empty
                //          allocate
                //      write to empty

                todo!()
            }
            (None, Some(_)) | (Some(_), None) => todo!(),
        }

        todo!()
    }

    pub fn get(&mut self, key: &str) -> Result<Option<String>, DBError> {
        let (some_record, _) = self.scan_live_key(key)?;

        match some_record {
            None => Ok(None),
            Some(record) => Ok(Some(record.value)),
        }
    }
    pub fn delete(&mut self, key: &str) -> Result<(), DBError> {
        todo!()
    }
}
