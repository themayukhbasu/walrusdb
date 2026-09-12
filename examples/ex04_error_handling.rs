use std::fmt;
use std::fs::File;
use std::io::{Read, Write};

fn main() -> Result<(), DBError> {
    write_number("target/num.bin", 420)?;
    let n = read_number("target/num.bin")?;

    println!("read back: {}", n);

    read_number("target/does_not_exist.bin")?;
    Ok(())
}

#[derive(Debug)]
enum DBError {
    Io(std::io::Error),
    MeaningOfLife, // 42 is the meaning of life
}

impl fmt::Display for DBError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DBError::Io(e) => write!(f, "I/O error: {}", e),
            DBError::MeaningOfLife => write!(f, "42 is the meaning of life"),
        }
    }
}

impl From<std::io::Error> for DBError {
    fn from(e: std::io::Error) -> Self {
        DBError::Io(e)
    }
}

fn write_number(path: &str, n: u32) -> Result<(), DBError> {
    let mut file = File::create(path)?;
    file.write_all(&n.to_le_bytes())?;
    Ok(())
}

fn read_number(path: &str) -> Result<u32, DBError> {
    let mut file = File::open(path)?;
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)?;
    let num = u32::from_le_bytes(buf);
    if num == 42 {
        return Err(DBError::MeaningOfLife);
    }
    Ok(num)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error, ErrorKind};
    use std::path::Path;

    #[test]
    fn write_and_read_succeeds() {
        let path_str = "target/test_num_1.bin";
        let path = Path::new(path_str);
        let _ = std::fs::remove_file(path); // clean slate
        let num = 123;

        assert!(write_number(path_str, num).is_ok());

        let res = read_number(path_str);
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), num);

        let _ = std::fs::remove_file(path); // clean slate
    }

    #[test]
    fn read_nonexistent_file_returns_err() {
        assert!(read_number("target/test_num_non_exists.bin").is_err())
    }

    #[test]
    fn error_display_is_not_empty() {
        let custom_error = Error::new(ErrorKind::Other, "oh no!");
        let err = DBError::Io(custom_error);
        assert_ne!(err.to_string(), "");
    }

    #[test]
    fn read_42_returns_meaning_of_life_error() {
        let path_str = "target/test_num_2.bin";
        let path = Path::new(path_str);
        let _ = std::fs::remove_file(path); // clean slate
        let num = 42;

        assert!(write_number(path_str, num).is_ok());

        let res = read_number(path_str);
        assert!(matches!(res, Err(DBError::MeaningOfLife)));

        let _ = std::fs::remove_file(path); // clean slate
    }
}
