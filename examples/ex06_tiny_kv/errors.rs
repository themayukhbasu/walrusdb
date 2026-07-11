use std::fmt;
use std::fmt::Formatter;

#[derive(Debug)]
pub enum DBError {
    Io(std::io::Error),
    BlockOutOfBounds(u64),
    RecordDataOutOfBounds(u16, u16),
    Decode(DecodeError),
}

#[derive(Debug)]
pub enum DecodeError {
    InvalidUtf8(std::str::Utf8Error),
    InvalidRecordStatus(u8),
    InvalidRecordLength(u16, u16),
}

// From trait to convert std::io::Error to DBError
impl From<std::io::Error> for DBError {
    fn from(e: std::io::Error) -> Self {
        DBError::Io(e)
    }
}

impl From<std::str::Utf8Error> for DecodeError {
    fn from(e: std::str::Utf8Error) -> Self {
        DecodeError::InvalidUtf8(e)
    }
}

// Display trait
impl fmt::Display for DBError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            DBError::Io(e) => write!(f, "DBError: I/O Error: {}", e),
            DBError::BlockOutOfBounds(n) => write!(f, "DBError: Block Out of Bounds: {}", n),
            DBError::RecordDataOutOfBounds(key_len, val_len) => write!(
                f,
                "DBError: Record Data Out of Bounds: (key_len {} + val_len {}) > 59",
                key_len, val_len
            ),
            DBError::Decode(e) => write!(f, "DBError: {}", e),
        }
    }
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DecodeError::InvalidUtf8(e) => write!(f, "DecodeError: Invalid UTF-8: {}", e),
            DecodeError::InvalidRecordStatus(status) => {
                write!(f, "DecodeError: Invalid Record Status: {}", status)
            }
            DecodeError::InvalidRecordLength(key_len, val_len) => write!(
                f,
                "DecodeError: Invalid record length: (key_len {} + val_len {}) > 59",
                key_len, val_len
            ),
        }
    }
}