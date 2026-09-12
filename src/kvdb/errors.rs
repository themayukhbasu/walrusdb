use std::fmt;
use std::fmt::{Formatter, write};

#[derive(Debug)]
pub enum DBError {
    Io(std::io::Error),
    InvalidPageNumber(u16, String),
    InvalidFreeSpace(u16, String),
    AvailabilityListOverflow(usize, usize, String),
}

impl From<std::io::Error> for DBError {
    fn from(e: std::io::Error) -> Self {
        DBError::Io(e)
    }
}

impl fmt::Display for DBError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DBError::Io(e) => write!(f, "DBError: I/O Error: {}", e),
            DBError::InvalidPageNumber(page_num, msg) => {
                write!(f, "DBError: Invalid Page Num: {} | {}", page_num, msg)
            }
            DBError::InvalidFreeSpace(free_space, msg) => {
                write!(f, "DBError: Invalide Free Space: {} | {}", free_space, msg)
            }
            DBError::AvailabilityListOverflow(al_size, total_size, msg) => {
                write!(
                    f,
                    "DBError: Availability List greater than max allowed: {}/{} | {}",
                    al_size, total_size, msg
                )
            }
        }
    }
}
