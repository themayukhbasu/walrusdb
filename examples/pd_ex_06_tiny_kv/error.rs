#[derive(Debug)]
pub enum DBError {
    IO(std::io::Error),
    BlockOutOfBounds(u64),
}

impl From<std::io::Error> for DBError {
    fn from(e: std::io::Error) -> Self {
        DBError::IO(e)
    }
}

impl std::fmt::Display for DBError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DBError::IO(e) => write!(f, "IO error: {}", e),
            DBError::BlockOutOfBounds(n) => write!(f, "Block out of bounds: {}", n),
        }
    }
}