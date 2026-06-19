use std::collections::HashMap;
use std::io;

fn main() {
    in_memory_kv_repl()
    // _test2();
}

fn _test() {
    println!("Hello, world!");

    let input = "echo hello world".to_string();
    let parts: Vec<&str> = input.split_whitespace().collect();
    println!("{:?}", parts);
    match parts.as_slice() {
        ["ping"] => println!("pong"),
        ["echo", rest @ ..] => println!("{}", rest.join(" ")),
        _ => println!("unknown command"),
    }

    let mut cache: HashMap<String, String> = HashMap::new();

    println!("Type something");

    let mut count = 0;
    loop {
        count += 1;
        let mut line = String::new();
        match io::stdin().read_line(&mut line) {
            Ok(0) => {
                println!("EOF, good bye");
                break;
            }
            Ok(_) => {
                let op = match line.trim() {
                    s if matches!(s.to_lowercase().as_str(), "quit" | "q" | "exit" | "bye") => {
                        println!("{:?}", cache);
                        println!("Exiting. good bye");
                        break;
                    }
                    s => {
                        cache.insert(count.to_string(), s.to_string());
                        s
                    }
                };
                println!("You entered: {}", op);
            }
            Err(_) => {
                println!("Some Error occurred");
                break;
            }
        };
    }
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

fn _test2() {
    let mut db = DB::init();
    db.put("foo", "bar is a big baz");

    println!("get(foo) = {:?}", db.get("foo"));
    println!("get(baz) = {:?}", db.get("baz"));
    db.put(&*"baz".to_string(), &*"qux".to_string());
    println!("get(baz) = {:?}", db.get("baz"));
    db.delete("foo");
    println!("get(foo) = {:?}", db.get("foo"));
    db.delete("foo");
    println!("tried to delete foo again")
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

fn in_memory_kv_repl() {
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
