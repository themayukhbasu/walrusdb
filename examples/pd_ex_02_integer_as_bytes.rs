use core::result::Result::Ok;
use std::io::{self, Write, Read};
use std::path::Path;
use std::fs::File;

const FILES_DIR: &str = "./target/Files/";
#[derive(Debug)]
enum DBError {
    IO(std::io::Error),
    FileAlreadyExists
}

impl From<std::io::Error> for DBError {
    fn from(e: std::io::Error) -> Self {
        DBError::IO(e)
    }
}
fn main(){
    let mut file_name = String::new();
    println!("Enter the file to be created: ");
    io::stdin().read_line(&mut file_name).expect("Failed to read line");
    let _file_name = file_name.trim();
    match create_file(_file_name){
        Ok(_) => println!("File created successfully"),
        Err(e) => match e {
            DBError::IO(io_err) => println!("I/O error occurred: {}", io_err),
            DBError::FileAlreadyExists => println!("File already exists"),
        },
    }

    let mut input = String::new();
    read_input(&mut input, "first".to_string());
    let _first_input = input.trim().parse::<u16>().expect("Please enter a valid integer");
    read_input(&mut input, "second".to_string());
    let _second_input = input.trim().parse::<u32>().expect("Please enter a valid integer");
    read_input(&mut input, "third".to_string());
    let _third_input = input.trim().parse::<u8>().expect("Please enter a valid integer");
    let mut buffer = encode_into_buffer(_first_input, _second_input, _third_input);
    write_to_file(_file_name, &buffer).expect("Failed to write to file");
    buffer = [0u8;8];
    read_to_file(_file_name, &mut buffer).expect("Failed to read from file");
    println!("Read data from file: {:?}", buffer);
    let (_first_input, _second_input, _third_input) = decode_from_buffer(&buffer);
    println!("Read integers from file: {}, {}, {}", _first_input, _second_input, _third_input);
}

fn create_file(file_name: &str) -> Result<(), DBError> {
    let path = Path::new(FILES_DIR).join(file_name);
    if !path.exists() {
        match File::create(&path) {
            Ok(_) => {
                Ok(())
            }
            Err(e) => Err(DBError::IO(e)),
        }
    } else {
        Err(DBError::FileAlreadyExists)
    }
}

fn encode_into_buffer(first: u16, second: u32, third: u8) -> [u8; 8] {
        let mut buffer = [0u8; 8];

        buffer[0..2].copy_from_slice(&first.to_le_bytes());
        buffer[2..6].copy_from_slice(&second.to_le_bytes());
        buffer[6] = third;

        buffer
    }

    fn decode_from_buffer(buffer: &[u8; 8]) -> (u16, u32, u8) {
        let first = u16::from_le_bytes(buffer[0..2].try_into().unwrap());
        let second = u32::from_le_bytes(buffer[2..6].try_into().unwrap());
        let third = buffer[6];

        (first, second, third)
    }

fn read_input(input: &mut String, count: String) {
    input.clear();
    println!("Enter {} integer input to be written to the file: ", count);
    io::stdin().read_line(input).expect("Failed to read line");
}

fn write_to_file(file_path: &str, buffer: &[u8]) -> Result<(), DBError> {
    let path = Path::new(FILES_DIR).join(file_path);
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .open(&path)?;
    file.write_all(buffer)?;
    Ok(())
}

fn read_to_file(file_path: &str, buffer: &mut [u8]) -> Result<(), DBError> {
    let path = Path::new(FILES_DIR).join(file_path);
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .open(&path)?;
    file.read_exact(buffer)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_file_name() -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        format!("ints_{}_{}.bin", std::process::id(), nanos)
    }

    #[test]
    fn encode_decode_roundtrip() {
        fs::create_dir_all(FILES_DIR).unwrap();

        let file_name = unique_file_name();
        let original_first: u16 = 42;
        let original_second: u32 = 100_000;
        let original_third: u8 = 7;

        let buffer = encode_into_buffer(original_first, original_second, original_third);

        create_file(&file_name).unwrap();
        write_to_file(&file_name, &buffer).unwrap();

        let mut read_buffer = [0u8; 8];
        read_to_file(&file_name, &mut read_buffer).unwrap();

        let (first, second, third) = decode_from_buffer(&read_buffer);

        assert_eq!(first, original_first);
        assert_eq!(second, original_second);
        assert_eq!(third, original_third);

        let _ = fs::remove_file(Path::new(FILES_DIR).join(&file_name));
    }

    #[test]
    fn u16_is_little_endian() {
        let mut buffer = [0u8; 8];

        buffer[0..2].copy_from_slice(&42u16.to_le_bytes());

        assert_eq!(buffer[0], 0x2A);
        assert_eq!(buffer[1], 0x00);
        assert_eq!(&buffer[0..2], &[0x2A, 0x00]);
    }

    #[test]
    fn u32_correct_byte_layout() {
        let mut buffer = [0u8; 8];

        buffer[2..6].copy_from_slice(&100_000u32.to_le_bytes());

        assert_eq!(buffer[2], 0xA0);
        assert_eq!(buffer[3], 0x86);
        assert_eq!(buffer[4], 0x01);
        assert_eq!(buffer[5], 0x00);
        assert_eq!(&buffer[2..6], &[0xA0, 0x86, 0x01, 0x00]);
    }
}

