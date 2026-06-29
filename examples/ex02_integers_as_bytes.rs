use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

fn main() {
    let path = Path::new("target/ints.bin");
    println!("size(u16) = {}", std::mem::size_of::<u16>());
    println!("size(u32) = {}", std::mem::size_of::<u32>());
    println!("size(u8) = {}", std::mem::size_of::<u8>());

    let v1: u16 = 42;
    let v2: u32 = 100_000;
    let v3: u8 = 7;
    let buf = _encode(v1, v2, v3);
    // _create_file(path);
    _write_file(path, &buf);
    let buf = _read_file(path);
    println!("{:?}", buf);
    let (v1, v2, v3) = _decode(buf);
    println!("v1= {} | v2= {} | v3= {}", v1, v2, v3);

    let _ = std::fs::remove_file(path); // clean up after
}

fn _encode(v1: u16, v2: u32, v3: u8) -> [u8; 8] {
    let mut buf: [u8; 8] = [0u8; 8];

    // println!("{:?}", buf);
    let bytes = v1.to_le_bytes();
    buf[0..=1].copy_from_slice(&bytes);
    // println!("{:?}", buf);

    let bytes = v2.to_le_bytes();
    buf[2..=5].copy_from_slice(&bytes);
    // println!("{:?}", buf);

    // let bytes = v3.to_le_bytes();
    // buf[6..7].copy_from_slice(&bytes);
    // buf[6] = bytes[0];

    buf[6] = v3;
    println!("{:?}", buf);
    buf
}

fn _decode(buf: [u8; 8]) -> (u16, u32, u8) {
    let v1 = u16::from_le_bytes(buf[0..=1].try_into().unwrap());
    let v2: u32 = u32::from_le_bytes(buf[2..=5].try_into().unwrap());
    let v3: u8 = buf[6];
    (v1, v2, v3)
}

fn _write_file(path: &Path, buf: &[u8]) {
    let mut file = match File::create(path) {
        Err(why) => panic!("Error: {}", why),
        Ok(file) => file,
    };
    match file.write_all(buf) {
        Err(why) => panic!("couldn't write to file: {}", why),
        Ok(_) => println!("successfully wrote to {}", path.display()),
    }
}

fn _read_file(path: &Path) -> [u8; 8] {
    let mut file = match File::open(path) {
        Err(why) => panic!("Error: {}", why),
        Ok(file) => file,
    };
    let mut buf = [0u8; 8];

    match file.read_exact(&mut buf) {
        Err(why) => panic!("Error: {}", why),
        Ok(_) => {}
    }
    buf
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_works() {
        let v1: u16 = 42;
        let v2: u32 = 100_000;
        let v3: u8 = 7;

        assert_eq!(_encode(v1, v2, v3), [42, 0, 160, 134, 1, 0, 7, 0])
    }

    #[test]
    fn encode_and_decode_works() {
        let v1: u16 = 42;
        let v2: u32 = 100_000;
        let v3: u8 = 7;
        assert_eq!((v1, v2, v3), _decode(_encode(v1, v2, v3)));
    }

    #[test]
    fn encode_decode_roundtrip() {
        let path = Path::new("target/test_ints.bin");
        let _ = std::fs::remove_file(path); // clean slate

        let v1: u16 = 42;
        let v2: u32 = 100_000;
        let v3: u8 = 7;
        let buf = _encode(v1, v2, v3);

        _write_file(path, &buf);
        let buf = _read_file(path);

        assert_eq!((v1, v2, v3), _decode(buf));

        let _ = std::fs::remove_file(path); // clean up after
    }

    #[test]
    fn u16_is_little_endian() {
        let v1: u16 = 42;
        let v2: u32 = 100_000;
        let v3: u8 = 7;
        let buf = _encode(v1, v2, v3);

        assert_eq!(buf[0], 0x2A);
    }

    #[test]
    fn u32_correct_byte_layout() {
        let v1: u16 = 42;
        let v2: u32 = 100_000;
        let v3: u8 = 7;
        let buf = _encode(v1, v2, v3);

        assert_eq!(buf[2..=5], [0xA0, 0x86, 0x01, 0x00]);
    }
}
