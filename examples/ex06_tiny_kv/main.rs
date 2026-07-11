mod blockstore;
mod errors;
mod record;
mod tinykv;

use crate::errors::DBError;
use std::io;
use std::path::Path;
use tinykv::TinyKV;

fn main() -> Result<(), DBError> {
    let path_str = "./target/tinykvstore.bin";
    // let path = Path::new(path_str);
    // let _ = std::fs::remove_file(path);

    let mut db = TinyKV::init(path_str)?;

    // println!("{:?}", db.get_all());
    // println!("==========");
    // db.put("foo", "bar")?;
    // println!("{:?}", db.get_all());
    // println!("==========");
    // db.put("baz", "qux")?;
    // println!("{:?}", db.get_all());
    //
    // println!("==========");
    // db.put("blue", "green")?;
    // println!("{:?}", db.get_all());

    repl(db)?;

    Ok(())
}

fn print_get(db: &mut TinyKV, key: &str) -> Result<Option<String>, DBError> {
    let some_value = db.get(key)?;
    let val = match some_value {
        Some(val) => {
            println!("Value = {}", val);
            Some(val)
        }
        None => {
            println!("Key not present in DB: {}", key);
            None
        }
    };
    Ok(val)
}

fn compile(db: &mut TinyKV, query: &str) -> Result<(), DBError> {
    let commands: Vec<&str> = query.split_whitespace().collect();

    let _ = match commands.as_slice() {
        ["GET", rest @ ..] => {
            let value = db.get(rest[0])?;
            match value {
                Some(s) => println!("Value = {}", s),
                None => println!("Sorry, no such key is present in DB!"),
            }
        }
        ["PUT", rest @ ..] => db.put(rest[0], rest[1..].join(" ").as_str())?,
        ["DELETE", rest @ ..] => db.delete(rest[0])?,
        ["DUMP"] => println!("DB dump = {:?}", db.get_all()),
        _ => println!("Unknown Command"),
    };
    Ok(())
}

fn repl(mut db: TinyKV) -> Result<(), DBError> {
    // Read-Evaluate-Print-Loop for in memory KV store

    loop {
        let mut input = String::new();

        println!(
            "> Enter your query [GET <key> | PUT <key> <value> | DELETE <key>] or [q] to quit:"
        );

        let input = match io::stdin().read_line(&mut input) {
            Ok(0) => {
                println!("EOF Found | Exiting REPL");
                break;
            }
            Ok(_) => match input.trim() {
                "q" | "quit" | "exit" => {
                    println!("Exit Message Found | Exiting REPL");
                    break;
                }
                s => s.trim(),
            },
            Err(e) => return Err(DBError::Io(e)),
        };

        compile(&mut db, input)?
    }
    Ok(())
}
