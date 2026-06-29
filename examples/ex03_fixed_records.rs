use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

const RECORD_SIZE: u32 = 8;
#[derive(PartialEq, Debug)]
struct PlayerRecord {
    score: u32,
    level: u16,
    active: u8,
}

fn main() {
    let path = Path::new("target/records.bin");
    let record1 = PlayerRecord {
        score: 66780,
        level: 28,
        active: 0,
    };
    let record2 = PlayerRecord {
        score: 230,
        level: 2,
        active: 1,
    };
    let record3 = PlayerRecord {
        score: 98675,
        level: 199,
        active: 0,
    };
    let record4 = PlayerRecord {
        score: 10_000,
        level: 12,
        active: 1,
    };

    println!("{:?}", encode(&record1));
    println!("{:?}", decode(encode(&record1)));

    write_buf_to_file(path, &encode(&record1));
    write_buf_to_file(path, &encode(&record2));
    write_buf_to_file(path, &encode(&record3));
    write_buf_to_file(path, &encode(&record4));

    let rec = decode(read_buf_from_offset_file(path, (RECORD_SIZE * 2) as u64));
    println!("{:?}", rec);
    let rec = decode(read_buf_from_offset_file(path, (RECORD_SIZE * 0) as u64));
    println!("{:?}", rec);
}

fn encode(record: &PlayerRecord) -> [u8; 8] {
    let mut buf = [0u8; 8];

    buf[0..=3].copy_from_slice(&record.score.to_le_bytes());

    buf[4..=5].copy_from_slice(&record.level.to_le_bytes());
    buf[6] = record.active;

    buf
}

fn decode(buf: [u8; 8]) -> PlayerRecord {
    let record = PlayerRecord {
        score: u32::from_le_bytes(buf[0..=3].try_into().unwrap()),
        level: u16::from_le_bytes(buf[4..=5].try_into().unwrap()),
        active: buf[6],
    };

    record
}

fn write_buf_to_file(path: &Path, buf: &[u8; 8]) {
    let mut file = match OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(path)
    {
        Err(why) => panic!("error: {}", why),
        Ok(file) => file,
    };
    match file.write_all(buf) {
        Err(why) => panic!("error: {}", why),
        Ok(_) => println!("write successful"),
    }
}

fn read_buf_from_offset_file(path: &Path, offset: u64) -> [u8; 8] {
    let mut file = match File::open(path) {
        Err(why) => panic!("error: {}", why),
        Ok(file) => file,
    };

    let mut buf = [0u8; 8];
    match file.seek(SeekFrom::Start(offset)) {
        Err(why) => panic!("error: {}", why),
        Ok(_) => match file.read_exact(&mut buf) {
            Err(why) => panic!("error: {}", why),
            Ok(_) => {}
        },
    }

    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip() {
        let record = PlayerRecord {
            score: 10_000,
            level: 5,
            active: 1,
        };

        assert_eq!(record, decode(encode(&record)));
    }

    #[test]
    fn decode_encode_roundtrip() {
        let buf = [220, 4, 1, 0, 28, 0, 0, 0];
        let record1 = PlayerRecord {
            score: 66780,
            level: 28,
            active: 0,
        };

        assert_eq!(decode(buf), decode(encode(&record1)));
    }

    #[test]
    fn seek_to_record_2() {
        let path = Path::new("target/test_records_1.bin");
        let _ = std::fs::remove_file(path); // clean slate

        let record1 = PlayerRecord {
            score: 66780,
            level: 28,
            active: 0,
        };
        let record2 = PlayerRecord {
            score: 230,
            level: 2,
            active: 1,
        };
        let record3 = PlayerRecord {
            score: 98675,
            level: 199,
            active: 0,
        };
        let record4 = PlayerRecord {
            score: 10_000,
            level: 12,
            active: 1,
        };

        write_buf_to_file(path, &encode(&record1));
        write_buf_to_file(path, &encode(&record2));
        write_buf_to_file(path, &encode(&record3));
        write_buf_to_file(path, &encode(&record4));

        let rec = decode(read_buf_from_offset_file(path, (RECORD_SIZE * 2) as u64));
        assert_eq!(record3, rec);

        let _ = std::fs::remove_file(path); // clean up after
    }

    #[test]
    fn seek_to_record_0() {
        let path = Path::new("target/test_records_2.bin");
        let _ = std::fs::remove_file(path); // clean slate

        let record1 = PlayerRecord {
            score: 66780,
            level: 28,
            active: 0,
        };
        let record2 = PlayerRecord {
            score: 230,
            level: 2,
            active: 1,
        };
        let record3 = PlayerRecord {
            score: 98675,
            level: 199,
            active: 0,
        };
        let record4 = PlayerRecord {
            score: 10_000,
            level: 12,
            active: 1,
        };

        write_buf_to_file(path, &encode(&record1));
        write_buf_to_file(path, &encode(&record2));
        write_buf_to_file(path, &encode(&record3));
        write_buf_to_file(path, &encode(&record4));

        let rec = decode(read_buf_from_offset_file(path, (RECORD_SIZE * 0) as u64));
        assert_eq!(record1, rec);

        let _ = std::fs::remove_file(path); // clean up after
    }
}
