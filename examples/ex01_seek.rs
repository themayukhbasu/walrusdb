use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

fn main() {
    let path = Path::new("target/slots.bin");
    _create_write(path);
    let mut offset: u64 = 0;
    println!("{:?}", _seek_file(path, offset));
    offset = 32;
    println!("{:?}", _seek_file(path, offset));
    offset = 64;
    println!("{:?}", _seek_file(path, offset));
    /*
    _seek_file(path, offset.clone());

    .clone() is unnecessary on a u64. Remember when I said to pass numbers by value, not reference? The reason is that u64 implements the Copy trait — which means Rust
    automatically copies it when you pass it to a function. No .clone() needed, it just happens.

    _seek_file(path, offset);  // u64 is Copy — it's automatically copied

    Copy is for small, cheap types: all the integer types, bool, f32/f64, char. Anything that fits in a register basically. String and Vec are not Copy — they own heap memory, so
    copying them requires an explicit .clone().

     */
}

fn _seek_file(path: &Path, offset: u64) -> [u8; 32] {
    let display = path.display();

    let mut file = match File::open(path) {
        Err(why) => panic!("error opening {}: {}", display, why),
        Ok(file) => file,
    };

    let buf = match file.seek(SeekFrom::Start(offset)) {
        Err(why) => panic!("error reading {}: {}", display, why),
        Ok(..) => match _read_slot(&mut file) {
            Err(why) => panic!("couldn't read {}: {}", display, why),
            Ok(buf) => buf,
        },
    };
    buf
}

fn _read_slot(file: &mut File) -> Result<[u8; 32], std::io::Error> {
    let mut buf = [0u8; 32];
    file.read_exact(&mut buf)?;
    Ok(buf)
}

fn _create_write(path: &Path) {
    _create_file(path);
    let slot0: [u8; 32] = [b'A'; 32];
    println!("{:?}", slot0);
    let slot1: [u8; 32] = [b'B'; 32];
    println!("{:?}", slot1);
    let slot2: [u8; 32] = [b'C'; 32];
    println!("{:?}", slot2);
    _write_file(&path, &slot0);
    _write_file(&path, &slot1);
    _write_file(&path, &slot2);
}

fn _create_file(path: &Path) {
    match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", path.display(), why),
        Ok(_) => println!("successfully created {}", path.display()),
    }
}

fn _write_file(path: &Path, buf: &[u8]) {
    let display = path.display();

    let mut file = match OpenOptions::new().write(true).append(true).open(path) {
        Err(why) => panic!("error opening {}: {}", display, why),
        Ok(file) => file,
    };

    match file.write_all(buf) {
        Err(why) => panic!("error writing to {}: {}", display, why),
        Ok(..) => println!("successfully written to {}", display),
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_slot_0_returns_all_b_a() {
        let path = Path::new("target/test_slots_0.bin");
        let _ = std::fs::remove_file(path); // clean slate
        _create_write(path);
        let offset: u64 = 0;
        let buf = _seek_file(path, offset);
        assert_eq!(buf, [b'A'; 32]);
        let _ = std::fs::remove_file(path); // clean up after
    }

    #[test]
    fn read_slot_1_returns_all_b_b() {
        let path = Path::new("target/test_slots_1.bin");

        let _ = std::fs::remove_file(path); // clean slate
        _create_write(path);
        let buf = _seek_file(path, 32);
        assert_eq!(buf, [b'B'; 32]);

        let _ = std::fs::remove_file(path); // clean up after
    }

    #[test]
    fn read_slot_2_returns_all_b_c() {
        let path = Path::new("target/test_slots_2.bin");

        let _ = std::fs::remove_file(path); // clean slate
        _create_write(path);
        let buf = _seek_file(path, 64);
        assert_eq!(buf, [b'C'; 32]);

        let _ = std::fs::remove_file(path); // clean up after
    }
}
