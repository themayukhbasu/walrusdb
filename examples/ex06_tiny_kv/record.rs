use crate::blockstore::BLOCK_SIZE;
use crate::errors::{DBError, DecodeError};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum RecordStatus {
    Empty = 0,
    Live = 1,
    Deleted = 2,
}

impl TryFrom<u8> for RecordStatus {
    type Error = DecodeError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Empty),
            1 => Ok(Self::Live),
            2 => Ok(Self::Deleted),
            status => Err(DecodeError::InvalidRecordStatus(status)),
        }
    }
}

#[derive(Debug)]
pub struct Record {
    pub status: RecordStatus,
    key_len: u16,
    val_len: u16,
    pub key: String,
    pub value: String,
}

impl Record {
    pub fn new(status: RecordStatus, key: &str, value: &str) -> Result<Self, DBError> {
        let key_len = key.len() as u16;
        let val_len = value.len() as u16;

        if key_len + val_len > 59 {
            return Err(DBError::RecordDataOutOfBounds(key_len, val_len));
        }

        Ok(Self {
            status,
            key_len,
            val_len,
            key: key.to_string(),
            value: value.to_string(),
        })
    }

    pub fn size(&self) -> u16 {
        5 + self.key_len + self.val_len as u16
    }
    pub fn encode(&self) -> [u8; 64] {
        let mut buf = [0u8; BLOCK_SIZE];

        // fixed size bytes
        buf[0] = self.status as u8;
        buf[1..=2].copy_from_slice(&self.key_len.to_le_bytes());
        buf[3..=4].copy_from_slice(&self.val_len.to_le_bytes());

        // variable size key
        let key_start_offset: usize = 5usize;
        let key_end_offset: usize = key_start_offset + self.key_len as usize;
        buf[key_start_offset..key_end_offset].copy_from_slice(self.key.as_bytes());

        // variable size value
        let value_start_offset: usize = key_end_offset;
        let value_end_offset: usize = value_start_offset + self.val_len as usize;
        buf[value_start_offset..value_end_offset].copy_from_slice(self.value.as_bytes());

        buf
    }

    pub fn decode(buf: [u8; BLOCK_SIZE]) -> Result<Self, DecodeError> {
        let status = RecordStatus::try_from(buf[0])?;
        let key_len: u16 =
            u16::from_le_bytes(buf[1..=2].try_into().expect("buf[1..=2] is always 2 bytes"));
        let val_len: u16 =
            u16::from_le_bytes(buf[3..=4].try_into().expect("buf[3..=4 is always 2 bytes"));

        if key_len + val_len > 59 {
            return Err(DecodeError::InvalidRecordLength(key_len, val_len));
        }

        // variable size key
        let key_start_offset: usize = 5usize;
        let key_end_offset: usize = key_start_offset + key_len as usize;
        let key_bytes = &buf[key_start_offset..key_end_offset as usize];
        let key = std::str::from_utf8(key_bytes)?;

        // variable size value
        let value_start_offset: usize = key_end_offset;
        let value_end_offset: usize = value_start_offset + val_len as usize;
        let value_bytes = &buf[value_start_offset..value_end_offset];
        let value = std::str::from_utf8(value_bytes)?;

        Ok(Self {
            status,
            key_len,
            val_len,
            key: key.to_string(),
            value: value.to_string(),
        })
    }
}
