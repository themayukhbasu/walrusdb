use std::collections::HashMap;
use std::io;

fn main() {
    println!("Hello, world!");

    let input = "echo hello world";
    let parts: Vec<&str> = input.split_whitespace().collect();
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
            Ok(1_usize..) => {
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
