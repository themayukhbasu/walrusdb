use crate::blockstore;
use crate::errors;

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

    fn scan_live(&mut self, key: &str) -> Result<u64, DBError> {
        let db_size = self.store.num_blocks()?;
        
        todo!()
    }

    pub fn put(&mut self, key: &str, value: &str) -> Result<(), DBError> {
        todo!()
    }

    pub fn get(&self, key: &str) -> Result<&str, DBError> {
        todo!()
    }
    pub fn delete(&mut self, key: &str) -> Result<(), DBError> {
        todo!()
    }
}
