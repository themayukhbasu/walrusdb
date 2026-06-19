# Phase 0 Journal — REPL + In-Memory KV Store

**What I built:**
A REPL that accepts `GET`, `PUT`, `DELETE`, and `DUMP` commands against an in-memory key-value store backed by a `HashMap`. I structured the code with a `DB` struct and an `impl` block to keep the store logic separate from the REPL loop and the command parser.

**What surprised me about Rust:**
The compiler's error messages are genuinely expressive — more helpful than most languages I've used. My background with Scala and Cats (particularly `Option` and pattern matching) translated well; `Some(x)` / `None` in Rust feels almost identical conceptually.

**Where I got stuck:**
The borrow checker — I don't fully understand it yet, but I got things working by reading the compiler errors carefully and trying different approaches. The `read_line` return type also tripped me up initially; matching on `Ok(0)` vs `Ok(n)` wasn't obvious until I looked it up in the Rust Book, which is excellent.