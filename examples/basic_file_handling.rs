use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::path::Path;

fn main() {
    let path = Path::new("target/lorem_ipsum3.txt");
    _create_and_write(path);
    _append(path);
    _read(path);
}

fn _read(path: &Path) {
    let display = path.display();

    // open file in read-only mode
    let mut file = match File::open(&path) {
        Err(why) => panic!("couldn't open file {}: {}", display, why),
        Ok(file) => file,
    };

    // read the file into a string
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => panic!("couldn't read {}: {}", display, why),
        Ok(_) => println!("{} contains:\n {}", display, s),
    }
}

fn _create_and_write(path: &Path) {
    let display = path.display();

    static LOREM_IPSUM: &str =
        "Lorem ipsum dolor sit amet, consectetur adipisicing elit, sed do eiusmod
tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam,
quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo
consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse
cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non
proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
";

    // Open file in write only mode
    let mut file = match File::create(&path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(file) => file,
    };

    // write to file
    match file.write_all(LOREM_IPSUM.as_bytes()) {
        Err(why) => panic!("couldn't write to {}: {}", display, why),
        Ok(_) => println!("successfully wrote to {}", display),
    }
}

fn _append(path: &Path) {
    let display = path.display();

    // open file in append mode
    let mut file = match OpenOptions::new().write(true).append(true).open(path) {
        Err(why) => panic!("couldn't open {}", why),
        Ok(file) => file,
    };

    // data to append
    let foo = "\nbar baz qux quux";

    // append to file
    match file.write_all(foo.as_bytes()) {
        Err(why) => panic!("couldn't append {}", why),
        Ok(_) => println!("successfully appended to {}", display),
    }
}
