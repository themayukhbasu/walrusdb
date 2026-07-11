/* Add Files folder to the target folder */
use std::io::{self, Seek, SeekFrom, Read, Write};
use std::fs::{File, OpenOptions};
use std::path::Path;

const FILES_DIR: &str = "./target/Files/";
const CHUNK_SIZE : usize = 8;

#[derive(Debug)]
enum DBError {
    IO(std::io::Error),
    FileAlreadyExists
}

fn main() {
    let mut file_name = String::new();
    println!("Enter the file to be created: ");
    io::stdin().read_line(&mut file_name)
        .expect("Failed to read line");
    let file_name = file_name.trim();
    println!("File to be created: {}", file_name);
    match create_file(file_name) {
        Ok(_) => println!("File created successfully"),
        Err(e) => match e {
            DBError::IO(io_err) => println!("I/O error occurred: {}", io_err),
            DBError::FileAlreadyExists => println!("File already exists"),
        },
    }
    let mut input = String::new();
    println!("Enter the data to be written to the file: ");
    io::stdin().read_line(&mut input)
        .expect("Failed to read line");
    write_to_file(file_name, input.trim()).expect("Failed to write to file");
    println!("Enter the Slot to be read from the file: ");
    let mut slot_input = String::new();
    io::stdin().read_line(&mut slot_input)
        .expect("Failed to read line");
    let slot: usize = slot_input.trim().parse().expect("Please enter a valid number");
    read_from_file(file_name, slot).expect("Failed to read from file");
}
// Function to create a file, added error handling to check if the file already exists
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

impl From<std::io::Error> for DBError {
    fn from(e: std::io::Error) -> Self {
        DBError::IO(e)
    }
}
// Function to write data to a file in chunks, added error handling for I/O operations
fn write_to_file(file_path: &str, input: &str) -> Result<(), DBError> {
    let path = Path::new(FILES_DIR).join(file_path);
    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)?;
    let bytes = input.as_bytes();
    for chunk  in bytes.chunks(CHUNK_SIZE) {
        let mut slot = [0u8; CHUNK_SIZE];
        slot[..chunk.len()].copy_from_slice(chunk);
        println!("Slot bytes: {:?}", slot);
        match file.write_all(&slot) {
            Ok(_) => println!("Chunk written successfully"),
            Err(e) => return Err(DBError::IO(e)),
        }

    }
    Ok(())

}
// Function to read data from a file in chunks, added error handling for I/O operations
fn read_from_file(file_path:&str, slot: usize) -> Result<(), DBError> {
    let path = Path::new(FILES_DIR).join(file_path);
    let mut file = OpenOptions::new()
        .read(true)
        .open(&path)?;
    let offset = slot * CHUNK_SIZE;
    file.seek(SeekFrom::Start(offset as u64))?;
    let mut buffer = vec![0u8; CHUNK_SIZE];
    match file.read_exact(&mut buffer) {
        Ok(_) => {
            let data = String::from_utf8_lossy(&buffer);
            println!("Data read from slot {}: {}", slot, data);
            Ok(())
        }
        Err(e) => return Err(DBError::IO(e)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn read_slot_0() {
        let file = "slot0.db";

        let path = Path::new(FILES_DIR).join(file);
        let _ = fs::remove_file(&path);

        assert!(create_file(file).is_ok());

        write_to_file(file, "AAA").unwrap();

        assert!(read_from_file(file, 0).is_ok());
    }

    #[test]
    fn read_slot_1() {
        let file = "slot1.db";

        let path = Path::new(FILES_DIR).join(file);
        let _ = fs::remove_file(&path);

        assert!(create_file(file).is_ok());

        write_to_file(file, "AAAAAAAABBBBBBBBCCCCCCCC").unwrap();

        assert!(read_from_file(file, 1).is_ok());
    }

    #[test]
    fn read_slot_2() {
        let file = "slot2.db";

        let path = Path::new(FILES_DIR).join(file);
        let _ = fs::remove_file(&path);

        assert!(create_file(file).is_ok());

        write_to_file(file, "AAAAAAAABBBBBBBBCCCCCCCC").unwrap();

        assert!(read_from_file(file, 2).is_ok());
    }
}