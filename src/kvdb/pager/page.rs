use crate::kvdb::pager::ByteRange;

pub const PAGE_SIZE: usize = 4096;
const HEADER_SIZE: usize = 32;
const CELL_POINTER_SIZE: usize = 8;

pub struct Page {
    header: Header,
    ptr_array: CellPtrArray,
    bytes: Vec<u8>,
}

pub struct Header {
    // 8 bytes
    page_id: u16,             // 2 bytes
    live_count: u16,          // 2 bytes
    ptr_array_loc: ByteRange, // 4 bytes
}

pub struct CellPtrArray {
    pointers: Vec<CellPtr>,
}

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
        let header_bytes = self.header.encode();
        todo!()
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
        todo!()
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
    fn encode(&self) {
        todo!()
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
    fn encode(&self) {
        todo!()
    }
    fn decode(buf: &[u8; CELL_POINTER_SIZE]) -> Self {
        let cell_loc = ByteRange::decode(buf[0..4].try_into().expect("fixed size"));
        let key_loc = ByteRange::decode(buf[4..8].try_into().expect("fixed size"));
        Self { cell_loc, key_loc }
    }
}
