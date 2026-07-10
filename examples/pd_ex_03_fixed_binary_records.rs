use std::io::{self, Seek, Read, Write};
use std::path::Path;
use std::fs::{File,remove_file};
use std::fs::OpenOptions;

const FILES_DIR: &str = "./target/Files/records.bin";

#[derive(Debug)]
enum DBError {
    IO(std::io::Error)
}

impl From<std::io::Error> for DBError {
    fn from(e: std::io::Error) -> Self {
        DBError::IO(e)
    }
}

struct PlayerRecord {
    score:u32,
    level:u16,
    active:u8
}

fn main() {
    let _first_player = PlayerRecord {
        score: 1000,
        level: 5,
        active: 1
    };

    let _second_player = PlayerRecord {
        score: 2000,
        level: 10,
        active: 0
    };

    let _third_player = PlayerRecord {
        score: 1500,
        level: 7,
        active: 1
    };

    let _fourth_player = PlayerRecord {
        score: 3000,
        level: 15,
        active: 1
    };
   match create_file(FILES_DIR) {
        Ok(_) => println!("File created successfully"),
        Err(e) => match e {
            DBError::IO(io_err) => println!("I/O error occurred: {}", io_err)
        },
    }
    let mut _buffer = encode(&_first_player);
    match write_to_file(FILES_DIR, &_buffer) {
        Ok(_) => println!("First player record written successfully"),
        Err(e) => match e {
            DBError::IO(io_err) => println!("I/O error occurred: {}", io_err)
        },
    }
    _buffer = encode(&_second_player);
    match write_to_file(FILES_DIR, &_buffer) {  
        Ok(_) => println!("Second player record written successfully"),
        Err(e) => match e {
            DBError::IO(io_err) => println!("I/O error occurred: {}", io_err)
        },
    }
    _buffer = encode(&_third_player);
    match write_to_file(FILES_DIR, &_buffer) {
        Ok(_) => println!("Third player record written successfully"),
        Err(e) => match e {
            DBError::IO(io_err) => println!("I/O error occurred: {}", io_err)
        },
    }
    _buffer = encode(&_fourth_player);
    match write_to_file(FILES_DIR, &_buffer) {      
        Ok(_) => println!("Fourth player record written successfully"),
        Err(e) => match e {
            DBError::IO(io_err) => println!("I/O error occurred: {}", io_err)
        },
    }
    match read_from_file(FILES_DIR, 1) {
        Ok(record) => {
            println!("Read record: score={}, level={}, active={}", record.score, record.level, record.active);
        },
        Err(e) => {
            match e {
                DBError::IO(io_err) => println!("I/O error occurred: {}", io_err),
            }
            return;
        }
    };


}

fn encode(record: &PlayerRecord) -> [u8;8] {
    let mut buffer = [0u8; 8];
    buffer[0..4].copy_from_slice(&record.score.to_le_bytes());
    buffer[4..6].copy_from_slice(&record.level.to_le_bytes());
    buffer[6] = record.active;
    return buffer;
}

fn decode(buffer: &[u8;8]) -> PlayerRecord {
    let score = u32::from_le_bytes(buffer[0..4].try_into().unwrap());
    let level = u16::from_le_bytes(buffer[4..6].try_into().unwrap());
    let active = buffer[6];
    PlayerRecord {
        score,
        level,
        active
    }
}

fn create_file(file_name: &str) -> Result<(), DBError> {
    let path = Path::new(file_name);

    if path.exists() {
        remove_file(path)?;
    }

    File::create(path)?;
    Ok(())
}

fn write_to_file(file_path: &str, buffer: &[u8;8]) -> Result<(), DBError> {
    let path = Path::new(file_path);
    let mut file = OpenOptions::new()
        .append(true)
        .open(&path)?;
    file.write_all(buffer)?;
    Ok(())
}

fn read_from_file(file_path: &str, index: usize) -> Result<PlayerRecord, DBError> {
    let path = Path::new(file_path);
    let mut file = OpenOptions::new()
        .read(true)
        .open(&path)?;
    let mut buffer = [0u8; 8];
    let offset = index * 8;
    file.seek(io::SeekFrom::Start(offset as u64))?;
    file.read_exact(&mut buffer)?;
    return Ok(decode(&buffer));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::remove_file;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_path(name: &str) -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        format!("./target/Files/{}_{}.bin", name, nanos)
    }

    fn sample_records() -> [PlayerRecord; 4] {
        [
            PlayerRecord {
                score: 1000,
                level: 5,
                active: 1,
            },
            PlayerRecord {
                score: 2000,
                level: 10,
                active: 0,
            },
            PlayerRecord {
                score: 1500,
                level: 7,
                active: 1,
            },
            PlayerRecord {
                score: 3000,
                level: 15,
                active: 1,
            },
        ]
    }

    #[test]
    fn encode_decode_roundtrip() {
        let record = PlayerRecord {
            score: 1000,
            level: 5,
            active: 1,
        };

        let encoded = encode(&record);
        let decoded = decode(&encoded);

        assert_eq!(decoded.score, 1000);
        assert_eq!(decoded.level, 5);
        assert_eq!(decoded.active, 1);
    }

    #[test]
    fn decode_encode_roundtrip() {
        let raw: [u8; 8] = [232, 3, 0, 0, 5, 0, 1, 0];

        let decoded = decode(&raw);
        let reencoded = encode(&decoded);

        assert_eq!(decoded.score, 1000);
        assert_eq!(decoded.level, 5);
        assert_eq!(decoded.active, 1);
        assert_eq!(reencoded, raw);
    }

    #[test]
    fn seek_to_record_2() {
        let path = unique_test_path("seek_to_record_2");
        let records = sample_records();

        let _ = remove_file(&path);
        create_file(&path).unwrap();

        for record in &records {
            let buffer = encode(record);
            write_to_file(&path, &buffer).unwrap();
        }

        let record = read_from_file(&path, 2).unwrap();
        assert_eq!(record.score, 1500);
        assert_eq!(record.level, 7);
        assert_eq!(record.active, 1);

        let _ = remove_file(&path);
    }

    #[test]
    fn seek_to_record_0() {
        let path = unique_test_path("seek_to_record_0");
        let records = sample_records();

        let _ = remove_file(&path);
        create_file(&path).unwrap();

        for record in &records {
            let buffer = encode(record);
            write_to_file(&path, &buffer).unwrap();
        }

        let record = read_from_file(&path, 0).unwrap();
        assert_eq!(record.score, 1000);
        assert_eq!(record.level, 5);
        assert_eq!(record.active, 1);

        let _ = remove_file(&path);
    }
}