use crate::kvdb::errors::DBError;
use crate::kvdb::pager::page::{CellPtr, CellPtrArray, Header, PAGE_SIZE, Page};
use crate::kvdb::pager::page_store::PageStore;

pub mod freelist_manager;
pub mod page;
mod page_store;

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct ByteRange {
    // 4 bytes
    offset: u16, // 2 bytes
    len: u16,    // 2 bytes
}

impl ByteRange {
    fn encode(&self) -> [u8; 4] {
        let mut buf = [0u8; 4];
        buf[0..2].copy_from_slice(&self.offset.to_le_bytes());
        buf[2..4].copy_from_slice(&self.len.to_le_bytes());
        buf
    }
    fn decode(buf: &[u8; 4]) -> Self {
        Self {
            offset: u16::from_le_bytes(buf[0..2].try_into().expect("fixed size")),
            len: u16::from_le_bytes(buf[2..4].try_into().expect("fixed size")),
        }
    }
}
pub fn test() -> String {
    println!("size of PtrByteRange: {}", std::mem::size_of::<ByteRange>());
    println!("size of CellPtr: {}", std::mem::size_of::<CellPtr>());
    println!(
        "size of CellPtrArray: {}",
        std::mem::size_of::<CellPtrArray>()
    );
    println!("size of Header: {}", std::mem::size_of::<Header>());
    page_store::PageStore::hello()
}

struct Pager {
    store: PageStore,
}

impl Pager {
    fn new_page(&mut self) -> Result<(), DBError> {
        let page_offset = self.store.allocate()?;
        let page_num = (page_offset / PAGE_SIZE as u64) as u16;
        let page = Page::new(page_num);
        todo!("update free list");
        self.store.write(page_offset, page.encode())?;
        Ok(())
    }

    fn read_page(&mut self, page_num: u16) -> Result<Page, DBError> {
        if page_num == 0 {
            return Err(DBError::InvalidPageNumber(
                page_num,
                String::from("page_num=0 is special page"),
            ));
        }
        let page_offset = (page_num * PAGE_SIZE as u16) as u64;
        let mut page_buffer = [0u8; PAGE_SIZE];
        self.store.read(page_offset, &mut page_buffer)?;
        let page = Page::decode(page_buffer);
        Ok(page)
    }

    fn write_page(&mut self, page: Page) -> Result<(), DBError> {
        let page_offset = (page.page_num() * PAGE_SIZE as u16) as u64;
        let page_buffer = page.encode();

        self.store.write(page_offset, page_buffer)?;
        Ok(())
    }

    fn get_record_bytes(page: &Page, key_bytes: &[u8]) -> Option<Vec<u8>> {
        let some_key_cell_ptr = page.lookup_key_cell_pointer(key_bytes);
        match some_key_cell_ptr {
            None => None,
            Some(key_cell_ptr) => {
                let cell_bytes = page.get_cell_bytes(&key_cell_ptr);
                Some(cell_bytes)
            }
        }
    }
}
