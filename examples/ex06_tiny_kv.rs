use std::fmt;
use std::fmt::{Formatter, write};
use std::fs::OpenOptions;
use std::io::{Read, Seek, SeekFrom, Write};

const BLOCK_SIZE: usize = 64;

fn todo() {
    // TODO (resume here):
    // - Decide DecodeError nesting: DBError::Decode(DecodeError) + `impl From<DecodeError> for DBError`.
    //   Currently there's a conflicting `impl From<std::str::Utf8Error> for DBError` stub (todo!()) —
    //   replace it with `From<Utf8Error> for DecodeError` instead, now that DecodeError exists.
    // - Rename DecodeError::BadData -> something like InvalidUtf8 (no need to repeat "Decode",
    //   the enum name already gives that context). Add a second DecodeError variant for
    //   "key_len/value_len read from disk don't fit in the block" (corrupted length fields).
    // - Add `impl fmt::Display for DecodeError`, and a `DBError::Decode(e)` arm in DBError's Display.
    // - Fix the off-by-one in the offset math in BOTH `encode` and `decode` — `..=` is inclusive,
    //   so `start..=(start + len)` covers len+1 bytes, not len. Trace through a concrete example
    //   (key_len = 3) to find the right range, then make decode mirror whatever encode ends up using.
    // - In `decode`, validate key_len/value_len (fits within the block, matches how `new` checks
    //   `key_len + value_len > 59`) BEFORE using them to slice — a corrupted length field must not
    //   be trusted enough to index into `buf` directly (panic risk, not a graceful DBError).
    // - Finish `decode`: slice + convert value bytes the same way as key bytes, then build `Self`.
    //   Revisit whether `Record.key_len`/`value_len` should still be struct fields at all, given
    //   `key.len()`/`value.len()` already recover that info losslessly.
    // - Double check whether `encode(&mut self)` actually needs `&mut` — nothing in its body mutates
    //   `self` anymore now that `RecordStatus` is `Copy`.
    todo!();
}

#[derive(Debug)]
enum DBError {
    Io(std::io::Error),
    BlockOutOfBounds(u64),
    InvalidRecordStatus(u8),
    RecordDataOutOfBounds(u16, u16),
}

enum DecodeError {
    BadData(std::str::Utf8Error),
}

// From trait to convert std::io::Error to DBError
impl From<std::io::Error> for DBError {
    fn from(e: std::io::Error) -> Self {
        DBError::Io(e)
    }
}

impl From<std::str::Utf8Error> for DBError {
    fn from(e: std::str::Utf8Error) -> Self {
        todo!()
    }
}

// Display trait
impl fmt::Display for DBError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            DBError::Io(e) => write!(f, "I/O Error: {}", e),
            DBError::BlockOutOfBounds(n) => write!(f, "Block Out of Bounds: {}", n),
            DBError::InvalidRecordStatus(status) => write!(f, "Invalid Record Status: {}", status),
            DBError::RecordDataOutOfBounds(key_len, value_len) => write!(
                f,
                "Record Data Out of Bounds: key_len {} + value_len {} <= 59",
                key_len, value_len
            ),
        }
    }
}

struct BlockStore {
    file: std::fs::File,
}

impl BlockStore {
    fn open(path: &str) -> Result<Self, DBError> {
        let file = OpenOptions::new()
            .create(true)
            .write(true)
            .read(true)
            .open(path)?;
        Ok(Self { file })
    }

    fn num_blocks(&mut self) -> Result<u64, DBError> {
        let size = self.file.seek(SeekFrom::End(0))?;
        Ok(size / BLOCK_SIZE as u64)
    }

    fn allocate_block(&mut self) -> Result<u64, DBError> {
        let size = self.file.seek(SeekFrom::End(0))?;
        self.file.write_all(&[0u8; BLOCK_SIZE])?;
        Ok((size / BLOCK_SIZE as u64) - 1)
    }

    fn validate_block_idx(&mut self, block_idx: u64) -> Result<(), DBError> {
        if block_idx >= self.num_blocks()? {
            return Err(DBError::BlockOutOfBounds(block_idx));
        }
        Ok(())
    }

    fn write(&mut self, block_idx: u64, data: [u8; BLOCK_SIZE]) -> Result<(), DBError> {
        self.validate_block_idx(block_idx)?;
        self.file
            .seek(SeekFrom::Start(block_idx * BLOCK_SIZE as u64))?;
        self.file.write_all(&data)?;
        Ok(())
    }

    fn read(&mut self, block_idx: u64) -> Result<[u8; BLOCK_SIZE], DBError> {
        self.validate_block_idx(block_idx)?;
        self.file
            .seek(SeekFrom::Start(block_idx * BLOCK_SIZE as u64))?;
        let mut buf = [0u8; BLOCK_SIZE];
        self.file.read(&mut buf)?;
        Ok(buf)
    }
}

#[derive(Copy, Clone)]
enum RecordStatus {
    Empty = 0,
    Live = 1,
    Deleted = 2,
}

impl TryFrom<u8> for RecordStatus {
    type Error = DBError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Empty),
            1 => Ok(Self::Live),
            2 => Ok(Self::Deleted),
            status => Err(DBError::InvalidRecordStatus(status)),
        }
    }
}

struct Record {
    status: RecordStatus,
    key_len: u16,
    value_len: u16,
    key: String,
    value: String,
}

impl Record {
    fn new(status: RecordStatus, key: &str, value: &str) -> Result<Self, DBError> {
        let key_len = key.len() as u16;
        let value_len = value.len() as u16;

        if key_len + value_len > 59 {
            return Err(DBError::RecordDataOutOfBounds(key_len, value_len));
        }

        Ok(Self {
            status,
            key_len,
            value_len,
            key: key.to_string(),
            value: value.to_string(),
        })
    }
    fn encode(&mut self) -> [u8; 64] {
        let mut buf = [0u8; BLOCK_SIZE];

        // fixed size bytes
        buf[0] = self.status as u8;
        buf[1..=2].copy_from_slice(&self.key_len.to_le_bytes());
        buf[3..=4].copy_from_slice(&self.value_len.to_le_bytes());

        // variable size key
        let key_start_offset: usize = 5usize;
        let key_end_offset: usize = key_start_offset + self.key_len as usize;
        buf[key_start_offset..=key_end_offset].copy_from_slice(self.key.as_bytes());

        // variable size value
        let value_start_offset: usize = key_end_offset + 1;
        let value_end_offset: usize = value_start_offset + self.value_len as usize;
        buf[value_start_offset..=value_end_offset].copy_from_slice(self.value.as_bytes());

        buf
    }

    fn decode(buf: [u8; BLOCK_SIZE]) -> Result<Self, DBError> {
        let status = RecordStatus::try_from(buf[0])?;
        let key_len: u16 =
            u16::from_le_bytes(buf[1..=2].try_into().expect("buf[1..=2] is always 2 bytes"));
        let value_len: u16 =
            u16::from_le_bytes(buf[3..=4].try_into().expect("buf[3..=4 is always 2 bytes"));

        let key_bytes = &buf[5..=key_len as usize];
        let key = std::str::from_utf8(key_bytes);
        todo!()
    }
}

fn main() {}
