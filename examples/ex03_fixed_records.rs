#[derive(PartialEq, Debug)]
struct PlayerRecord {
    score: u32,
    level: u16,
    active: u8,
}

fn main() {
    let record1 = PlayerRecord {
        score: 10_000,
        level: 12,
        active: 1,
    };

    println!("{:?}", encode(&record1));
    println!("{:?}", decode(encode(&record1)));
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


       assert_eq!(record, decode(encode(&record))) ;
    }
}