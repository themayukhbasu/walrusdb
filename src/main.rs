use std::collections::HashMap;
use std::io;

fn main() {
    repl()
}

struct DB {
    cache: HashMap<String, String>,
}

impl DB {
    fn init() -> Self {
        let map: HashMap<String, String> = HashMap::new();
        Self { cache: map }
    }
    fn put(&mut self, key: &str, value: &str) {
        self.cache.insert(key.to_string(), value.to_string());
    }

    fn get(&mut self, key: &str) -> Option<&String> {
        self.cache.get(key)
    }

    fn delete(&mut self, key: &str) {
        self.cache.remove(key);
    }
}

fn compile(db: &mut DB, query: &str) {
    let commands: Vec<&str> = query.split_whitespace().collect();

    match commands.as_slice() {
        ["GET", rest @ ..] => {
            let value = db.get(rest[0]);
            match value {
                Some(s) => println!("Value = {}", s),
                None => println!("Sorry, no such key is present in DB!"),
            }
        }
        ["PUT", rest @ ..] => db.put(rest[0], rest[1..].join(" ").as_str()),
        ["DELETE", rest @ ..] => db.delete(rest[0]),
        ["DUMP"] => println!("DB dump = {:?}", db.cache),
        _ => println!("Unknown Command"),
    }
}

fn repl() {
    // Read-Evaluate-Print-Loop for in memory KV store

    let mut db = DB::init();

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
            Err(e) => {
                panic!("Error occurred: {e:?}")
            }
        };

        compile(&mut db, input)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_and_get_works() {
        let mut db = DB::init();
        db.put("foo", "bar is a big baz");
        assert_eq!(db.get("foo").unwrap(), "bar is a big baz");
    }

    #[test]
    fn missing_key_returns_none() {
        let mut db = DB::init();
        assert_eq!(db.get("foo"), None);
    }

    #[test]
    fn delete_works() {
        let mut db = DB::init();
        db.put("foo", "bar is a big baz");
        assert_eq!(db.get("foo").unwrap(), "bar is a big baz");
        db.delete("foo");
        assert_eq!(db.get("foo"), None);
    }

    #[test]
    fn update_works() {
        let mut db = DB::init();
        db.put("foo", "bar");
        assert_eq!(db.get("foo").unwrap(), "bar");
        db.put("foo", "baz");
        assert_eq!(db.get("foo").unwrap(), "baz");
    }
}
