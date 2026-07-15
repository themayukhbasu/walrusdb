mod otherblockstore;
mod error;
mod tinykv;
use crate::error::DBError;
use std::io;

const FILES_DIR: &str = "./target/Files/block_store.bin";

fn main() -> Result<(), DBError> {

    let db = tinykv::TinyKV::open(FILES_DIR)?;
    repl(db)?;

    return Ok(());  
        
}

fn compile(db: &mut tinykv::TinyKV, query: &str) -> Result<(), DBError> {
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

        ["DELETE", key] => { db.delete(key)?; },
        ["DELETE"] => println!("Usage: DELETE <key>"),
        _ => println!("Unknown Command"),
    };
    Ok(())
}

fn repl(mut db: tinykv::TinyKV) -> Result<(), DBError> {

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
            Err(e) => return Err(DBError::IO(e)),
        };

        compile(&mut db, input)?
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};
    use super::tinykv::TinyKV;

    fn test_db_path(test_name: &str) -> String {
        let mut path = std::env::temp_dir();
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();

        path.push(format!(
            "tinykv_{}_{}_{}.bin",
            test_name,
            std::process::id(),
            unique
        ));

        path.to_string_lossy().into_owned()
    }

    #[test]
    fn put_and_get_returns_value() -> Result<(), DBError> {
        let path = test_db_path("put_and_get");
        let mut db = TinyKV::open(&path)?;

        db.put("name", "alice")?;
        let value = db.get("name")?;

        assert_eq!(value, Some("alice".to_string()));

        let _ = fs::remove_file(&path);
        Ok(())
    }

    #[test]
    fn get_missing_key_returns_none() -> Result<(), DBError> {
        let path = test_db_path("get_missing");
        let mut db = TinyKV::open(&path)?;

        let value = db.get("missing_key")?;

        assert_eq!(value, None);

        let _ = fs::remove_file(&path);
        Ok(())
    }

    #[test]
    fn delete_removes_key() -> Result<(), DBError> {
        let path = test_db_path("delete_removes");
        let mut db = TinyKV::open(&path)?;

        db.put("name", "alice")?;
        db.delete("name")?;

        assert_eq!(db.get("name")?, None);

        let _ = fs::remove_file(&path);
        Ok(())
    }

    #[test]
    fn overwrite_existing_key() -> Result<(), DBError> {
        let path = test_db_path("overwrite");
        let mut db = TinyKV::open(&path)?;

        db.put("name", "a")?;
        db.put("name", "b")?;

        assert_eq!(db.get("name")?, Some("b".to_string()));

        let _ = fs::remove_file(&path);
        Ok(())
    }

    #[test]
    fn delete_nonexistent_key_is_noop() -> Result<(), DBError> {
        let path = test_db_path("delete_missing");
        let mut db = TinyKV::open(&path)?;

        assert!(db.delete("ghost_key").is_ok());

        let _ = fs::remove_file(&path);
        Ok(())
    }

    #[test]
    fn data_persists_across_reopen() -> Result<(), DBError> {
        let path = test_db_path("reopen");

        {
            let mut db = TinyKV::open(&path)?;
            db.put("k1", "v1")?;
            db.put("k2", "v2")?;
        }

        let mut db = TinyKV::open(&path)?;
        assert_eq!(db.get("k1")?, Some("v1".to_string()));
        assert_eq!(db.get("k2")?, Some("v2".to_string()));

        let _ = fs::remove_file(&path);
        Ok(())
    }
}