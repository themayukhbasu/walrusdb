pub enum DBError {
    Io(std::io::Error),
}

impl From<std::io::Error> for DBError {
    fn from(e: std::io::Error) -> Self {
        DBError::Io(e)
    }
}
