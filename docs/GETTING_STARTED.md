# Getting Started — WalRusDB

> **WalRusDB** — **WAL** (write-ahead log) + **Rus**(t). Pronounced "walrus." Mascot status: mandatory.

This is the on-ramp *before* Phase 0, written for someone who knows essentially no Rust.

## The boundary for this guide

The Rust **language** — toolchain, syntax, standard-library APIs — I teach you directly here. That's the on-ramp, not the learning target. But the Phase 0 **program** (a REPL backed by an in-memory store) is *yours to assemble*. Below, each Rust mechanism appears in isolation; combining them into WalRusDB's first shell is your first exercise. Resist asking for the assembled version — wiring it up is exactly where you'll meet ownership for real.

---

## Step 1 — Install the toolchain

Install **rustup** (the version manager; it brings `rustc` the compiler and `cargo` the build tool):

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

(Cross-platform; on Windows use the installer from rustup.rs.) Then verify:

```
rustc --version
cargo --version
```

Editor: **RustRover** is the natural pick if you already live in the JetBrains stack — it's free for non-commercial use (learning and hobby projects explicitly qualify), full-featured, and gives you integrated debugging and refactoring with essentially zero setup. **VS Code + the `rust-analyzer` extension** is the equally good alternative if you prefer it. Either way, the thing that matters for a beginner is the same: **inline inferred types and errors explained as you type**, which shortens the borrow-checker feedback loop enormously.

Two notes on RustRover's free tier: keep it pointed at **personal projects only** — commercial or work code needs a paid license — and if you'd rather it not collect detailed code data, turn that off under *Settings → Appearance & Behavior → System Settings → Data Sharing*.

---

## Step 2 — Learn *just enough* Rust

Don't try to learn all of Rust first. Get a floor, then learn the rest as phases demand it.

- *The Rust Programming Language* ("the Book", free online), **chapters 1–6**.
- **`rustlings`** for hands-on reps (`cargo install rustlings`).

**Ownership in three sentences:** every value has exactly one owner; when the owner goes out of scope, the value is freed; you can either *move* ownership or *borrow* it (`&` shared, `&mut` exclusive), but never alias-and-mutate at the same time. That last rule is the whole game, and it's the same rule that keeps a database from corrupting a shared page.

### The mechanisms you'll need for Phase 0 (in isolation)

**Move vs. borrow:**
```rust
let a = String::from("hello");
let b = a;            // value MOVED into b
// println!("{a}");   // ERROR: a no longer owns it
println!("{b}");      // ok

fn length(s: &String) -> usize { s.len() }   // borrows, doesn't take
let s = String::from("hi");
let n = length(&s);   // lend s
println!("{s} is {n} chars");  // s still usable
```

**`Option<T>` — Rust's "maybe":**
```rust
let maybe: Option<i32> = Some(5);
match maybe {
    Some(n) => println!("got {n}"),
    None    => println!("nothing"),
}
```

**`Result<T, E>` and the `?` operator:**
```rust
fn double(s: &str) -> Result<i32, std::num::ParseIntError> {
    let n: i32 = s.parse()?;   // `?` returns early if parse fails
    Ok(n * 2)
}
```

**`HashMap` — the store substrate:**
```rust
use std::collections::HashMap;
let mut m: HashMap<String, String> = HashMap::new();
m.insert("k".to_string(), "v".to_string());
match m.get("k") {               // get returns Option<&String>
    Some(v) => println!("{v}"),
    None    => println!("missing"),
}
m.remove("k");
```

**Reading a line from stdin:**
```rust
use std::io;
let mut line = String::new();
io::stdin().read_line(&mut line).unwrap();
println!("you typed: {}", line.trim());
```

**Splitting input and matching on it** (note: deliberately *not* WalRusDB's commands — these are illustrative):
```rust
let input = "ping hello world";
let parts: Vec<&str> = input.split_whitespace().collect();
match parts.as_slice() {
["ping"]            => println!("pong"),
["echo", rest @ ..] => println!("{}", rest.join(" ")),
_                   => println!("unknown command"),
}
```

That's the whole toolkit. Everything in Phase 0 is some combination of the above.

---

## Step 3 — Phase 0, guided (you write the program)

**Scaffolding (fine to run as-is — it teaches nothing to protect):**
```
cargo new walrusdb
cd walrusdb
cargo run        # prints "Hello, world!"
```
Your code lives in `src/main.rs`.

Now build the shell in micro-steps. Each is small; close them one at a time.

- **A — The loop.** Read a line from stdin repeatedly until the user types `quit` or hits EOF. *Question to settle first:* where does your `String` buffer live — inside or outside the loop — and why does it matter that `read_line` *appends*?
- **B — The store.** Create a `HashMap<String, String>` that lives *across* loop iterations. *Question:* where must it be declared so it survives each turn, and will the borrow checker let you read from it and write to it in the same iteration?
- **C — Parse + dispatch.** Split the line into words and `match` on them to handle `PUT key value`, `GET key`, `DELETE key`. *Question:* `m.get("k")` hands you an `Option<&String>` — how do you turn "found / not found" into the two messages you print?
- **D — Wire it together** into the exit demo from `PLAN.md`: `PUT foo bar`, `GET foo` → `bar`, `DELETE foo`, `GET foo` → not found.

You now have every piece from Step 2. Assembling A–D *is* the exercise — including the ownership puzzles that fall out of it. If you get genuinely stuck after trying, that's when the tiered hints in `CLAUDE.md` kick in (a question first, code only as a last resort).

---

## Step 4 — Your first journal entry and first test

**Journal:** create `docs/journal/0001-phase0-setup.md` with: *what I built, what surprised me about Rust, where I got stuck.* Write it yourself — three honest sentences beat a polished page.

**Test:** here's the *syntax* of a Rust test (run with `cargo test`):
```rust
#[test]
fn arithmetic_works() {
    assert_eq!(2 + 2, 4);
}
```
Your task: write a real test that inserts a key into your store and asserts you can read it back. (To make the store testable, you'll probably want the store logic in a function or struct, not buried in `main` — a useful nudge toward Phase 1's structure.)

---

## What "done with Phase 0" feels like

The exit demo passes, you've got a journal entry and a passing test, and the Rust syntax has stopped looking alien. If Phase 0 felt *easy* — good, that's by design. Phase 1 is where you start owning actual bytes on disk, and the difficulty (and the payoff) climbs from there.

**One caution to re-read in a week:** the urge to have an agent write "just this one thing to move faster" is strongest on the easy phases, where it feels harmless. It isn't — the muscle you're building is *doing it yourself*, and it has to start here while the stakes are low.