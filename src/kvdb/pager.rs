use crate::kvdb::pager::page::{CellPtr, CellPtrArray, Header};

pub mod page;
mod page_store;

#[derive(PartialEq, Debug)]
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
