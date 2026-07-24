use crate::kvdb::pager::ByteRange;

pub const PAGE_SIZE: usize = 4096;
const HEADER_SIZE: usize = 32;
const CELL_POINTER_SIZE: usize = 8;

#[derive(Debug, PartialEq)]
pub struct Page {
    header: Header,
    ptr_array: CellPtrArray,
    bytes: Vec<u8>,
}

#[derive(PartialEq, Debug)]
pub struct Header {
    // 8 bytes
    page_id: u16,             // 2 bytes
    live_count: u16,          // 2 bytes
    ptr_array_loc: ByteRange, // 4 bytes
}

#[derive(PartialEq, Debug)]
pub struct CellPtrArray {
    pointers: Vec<CellPtr>,
}

#[derive(PartialEq, Debug)]
pub struct CellPtr {
    // 8 bytes
    cell_loc: ByteRange, // 4 bytes
    key_loc: ByteRange,  // 4 bytes
}

impl Page {
    pub fn id(&self) -> u16 {
        self.header.page_id
    }

    pub fn encode(&self) -> [u8; PAGE_SIZE] {
        let mut buf = [0u8; PAGE_SIZE];

        let header_bytes = self.header.encode();
        buf[0..HEADER_SIZE].copy_from_slice(&header_bytes);

        let ptr_array_bytes = self.ptr_array.encode();
        let ptr_array_start = self.header.ptr_array_loc.offset as usize;
        let ptr_array_end = ptr_array_start + self.header.ptr_array_loc.len as usize;
        buf[ptr_array_start..ptr_array_end].copy_from_slice(&ptr_array_bytes);

        buf[ptr_array_end..PAGE_SIZE].copy_from_slice(self.bytes.as_slice());
        buf
    }

    pub fn decode(buf: [u8; PAGE_SIZE]) -> Self {
        // header
        let header_bytes = buf[0..HEADER_SIZE].try_into().expect("fixed size");
        let header = Header::decode(header_bytes);

        // pointer array
        let ptr_array_offset = header.ptr_array_loc.offset as usize;
        let ptr_array_len = header.ptr_array_loc.len as usize;
        let ptr_array_bytes: Vec<u8> =
            buf[ptr_array_offset..ptr_array_offset + ptr_array_len].to_vec();
        let ptr_array = CellPtrArray::decode(ptr_array_bytes);

        // cell bytes
        let bytes: Vec<u8> = buf[ptr_array_offset + ptr_array_len..PAGE_SIZE].to_vec();

        Self {
            header,
            ptr_array,
            bytes,
        }
    }
}

impl Header {
    fn encode(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];

        buf[0..2].copy_from_slice(&self.page_id.to_le_bytes());
        buf[2..4].copy_from_slice(&self.live_count.to_le_bytes());
        buf[4..8].copy_from_slice(&self.ptr_array_loc.encode());

        buf
    }

    fn decode(buf: &[u8; HEADER_SIZE]) -> Self {
        let page_id = u16::from_le_bytes(buf[0..2].try_into().expect("fixed size"));
        let live_count = u16::from_le_bytes(buf[2..4].try_into().expect("fixed size"));
        let ptr_array_loc = ByteRange::decode(buf[4..8].try_into().expect("fixed size"));
        Self {
            page_id,
            live_count,
            ptr_array_loc,
        }
    }
}

impl CellPtrArray {
    fn encode(&self) -> Vec<u8> {
        let mut buf: Vec<u8> = Vec::new();

        for cell_ptr in &self.pointers {
            let cell_ptr_bytes = cell_ptr.encode();

            buf.extend(cell_ptr_bytes);
        }

        buf
    }

    fn decode(ptr_array_bytes: Vec<u8>) -> Self {
        let mut pointers: Vec<CellPtr> = Vec::new();
        let num_ptrs = (ptr_array_bytes.len() / CELL_POINTER_SIZE); // todo(raise error if invalid)
        for i in 0..num_ptrs {
            let start = i * CELL_POINTER_SIZE;
            let end = start + CELL_POINTER_SIZE;
            let cell_ptr_bytes = &ptr_array_bytes[start..end];
            let cell_ptr = CellPtr::decode(cell_ptr_bytes.try_into().expect("fixed size"));
            pointers.push(cell_ptr);
        }
        Self { pointers }
    }
}

impl CellPtr {
    fn encode(&self) -> [u8; CELL_POINTER_SIZE] {
        let mut buf = [0u8; CELL_POINTER_SIZE];

        let cell_loc_bytes = self.cell_loc.encode();
        let key_loc_bytes = self.key_loc.encode();

        buf[0..4].copy_from_slice(&cell_loc_bytes);
        buf[4..8].copy_from_slice(&key_loc_bytes);

        buf
    }
    fn decode(buf: &[u8; CELL_POINTER_SIZE]) -> Self {
        let cell_loc = ByteRange::decode(buf[0..4].try_into().expect("fixed size"));
        let key_loc = ByteRange::decode(buf[4..8].try_into().expect("fixed size"));
        Self { cell_loc, key_loc }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_page_encode_decode_round_trip_matches() {
        let header = Header {
            page_id: 1,
            live_count: 0,
            ptr_array_loc: ByteRange {
                offset: HEADER_SIZE as u16,
                len: 0,
            },
        };

        let cell_ptr_array = CellPtrArray { pointers: vec![] };

        let bytes = [0u8; PAGE_SIZE - HEADER_SIZE].to_vec();

        let page = Page {
            header,
            ptr_array: cell_ptr_array,
            bytes,
        };

        assert_eq!(page, Page::decode(page.encode()));
    }

    #[test]
    fn one_live_cell_page_encode_decode_round_trip_matches() {
        let header = Header {
            page_id: 1,
            live_count: 1,
            ptr_array_loc: ByteRange {
                offset: HEADER_SIZE as u16,
                len: CELL_POINTER_SIZE as u16,
            },
        };

        let cell_ptr = CellPtr {
            cell_loc: ByteRange {
                offset: 400,
                len: 200,
            },
            key_loc: ByteRange {
                offset: 400,
                len: 20,
            },
        };

        let cell_ptr_array = CellPtrArray {
            pointers: vec![cell_ptr],
        };

        let bytes =
            vec![
                0u8;
                PAGE_SIZE - HEADER_SIZE - (cell_ptr_array.pointers.len() * CELL_POINTER_SIZE)
            ];
        let page = Page {
            header,
            ptr_array: cell_ptr_array,
            bytes,
        };

        assert_eq!(page, Page::decode(page.encode()));
    }

    #[test]
    fn two_live_cell_page_encode_decode_round_trip_matches() {
        let header = Header {
            page_id: 2,
            live_count: 2,
            ptr_array_loc: ByteRange {
                offset: HEADER_SIZE as u16,
                len: 2 * CELL_POINTER_SIZE as u16,
            },
        };

        let cell_ptr1 = CellPtr {
            cell_loc: ByteRange {
                offset: 400,
                len: 200,
            },
            key_loc: ByteRange {
                offset: 400,
                len: 20,
            },
        };

        let cell_ptr2 = CellPtr {
            cell_loc: ByteRange {
                offset: 600,
                len: 150,
            },
            key_loc: ByteRange {
                offset: 600,
                len: 25,
            },
        };

        let cell_ptr_array = CellPtrArray {
            pointers: vec![cell_ptr1, cell_ptr2],
        };

        let bytes =
            vec![
                0u8;
                PAGE_SIZE - HEADER_SIZE - (cell_ptr_array.pointers.len() * CELL_POINTER_SIZE)
            ];
        let page = Page {
            header,
            ptr_array: cell_ptr_array,
            bytes,
        };

        assert_eq!(page, Page::decode(page.encode()));
    }
}
