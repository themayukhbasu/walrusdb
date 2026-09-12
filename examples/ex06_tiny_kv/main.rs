mod blockstore;
mod errors;
mod record;
mod tinykv;

use crate::errors::DBError;
use std::io;
use tinykv::TinyKV;

fn main() -> Result<(), DBError> {
    let path_str = "./target/tinykvstore.bin";
    // let path = Path::new(path_str);
    // let _ = std::fs::remove_file(path);

    let db = TinyKV::init(path_str)?;

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

fn compile(db: &mut TinyKV, query: &str) -> Result<(), DBError> {
    let commands: Vec<&str> = query.split_whitespace().collect();

    let _ = match commands.as_slice() {
        ["GET", key] => {
            let value = db.get(key)?;
            match value {
                Some(s) => println!("Value = {}", s),
                None => println!("Sorry, no such key is present in DB!"),
            }
        }
        ["GET"] => println!("Usage: GET <key>"),

        ["PUT", key, rest @ ..] => db.put(key, rest.join(" ").as_str())?,
        ["PUT"] => println!("Usage: PUT <key> <value>"),

        ["DELETE", key] => db.delete(key)?,
        ["DELETE"] => println!("Usage: DELETE <key>"),

        ["DUMP"] => println!("DB dump = {:?}", db.dump()),
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn put_and_get_returns_value() {
        let path_str = "target/test_tinykvstore_1.bin";
        let path = Path::new(path_str);
        let _ = std::fs::remove_file(path); // clean slate

        let mut db = TinyKV::init(path_str).unwrap();
        let (key, value) = ("foo", "bar");
        db.put(key, value).expect("TODO: panic message");
        let val = db.get(key).unwrap().unwrap();
        assert_eq!(value, val);

        let _ = std::fs::remove_file(path); // clean slate
    }

    #[test]
    fn get_missing_key_returns_none() {
        let path_str = "target/test_tinykvstore_2.bin";
        let path = Path::new(path_str);
        let _ = std::fs::remove_file(path); // clean slate

        let mut db = TinyKV::init(path_str).unwrap();
        let val = db.get("foo").unwrap();
        assert_eq!(None, val);

        let _ = std::fs::remove_file(path); // clean slate
    }

    #[test]
    fn delete_removes_key() {
        let path_str = "target/test_tinykvstore_3.bin";
        let path = Path::new(path_str);
        let _ = std::fs::remove_file(path); // clean slate

        let mut db = TinyKV::init(path_str).unwrap();

        let (key, value) = ("foo", "bar");
        db.put(key, value).unwrap();

        db.delete(key).unwrap();

        let val = db.get("foo").unwrap();
        assert_eq!(None, val);

        let _ = std::fs::remove_file(path); // clean slate
    }

    #[test]
    fn overwrite_existing_key() {
        let path_str = "target/test_tinykvstore_4.bin";
        let path = Path::new(path_str);
        let _ = std::fs::remove_file(path); // clean slate

        let mut db = TinyKV::init(path_str).unwrap();

        db.put("foo", "a").unwrap();

        let val = db.get("foo").unwrap();
        assert_eq!(Some("a".to_string()), val);

        db.put("foo", "b").unwrap();

        let val = db.get("foo").unwrap();
        assert_eq!(Some("b".to_string()), val);

        let _ = std::fs::remove_file(path); // clean slate
    }

    #[test]
    fn delete_nonexistent_key_is_noop() {
        let path_str = "target/test_tinykvstore_5.bin";
        let path = Path::new(path_str);
        let _ = std::fs::remove_file(path); // clean slate

        let mut db = TinyKV::init(path_str).unwrap();

        let res = db.delete("foo");

        assert!(res.is_ok());

        let _ = std::fs::remove_file(path); // clean slate
    }

    #[test]
    fn data_persists_across_reopen() {
        let path_str = "target/test_tinykvstore_6.bin";
        let path = Path::new(path_str);
        let _ = std::fs::remove_file(path); // clean slate

        let mut db = TinyKV::init(path_str).unwrap();

        let (key1, val1) = ("foo", "bar");
        let (key2, val2) = ("baz", "qux");

        db.put(key1, val1).unwrap();
        db.put(key2, val2).unwrap();

        drop(db);

        let mut db = TinyKV::init(path_str).unwrap();

        let val = db.get(key1).unwrap().unwrap();
        assert_eq!(val1, val);

        let val = db.get(key2).unwrap().unwrap();
        assert_eq!(val2, val);

        let _ = std::fs::remove_file(path); // clean slate
    }
}
