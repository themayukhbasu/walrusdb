# Exercise 4 — Error Handling with Result and ?

**File to create:** `examples/ex04_error_handling.rs`

## Goal

Replace `panic!` and `match`-or-panic with a real error type and the `?` operator. This is the Rust error handling pattern you'll use throughout Phase 1.

## Why this matters for Phase 1

Your Pager, Page, and Store will each return `Result<T, DbError>`. The `?` operator is what makes this ergonomic — it short-circuits on error and propagates it up the call stack. Without it, every fallible call becomes a `match` block and the signal drowns in noise.

## Concepts introduced

- Defining a custom error `enum`
- `impl std::fmt::Display for MyError`
- `impl std::error::Error for MyError`
- `impl From<std::io::Error> for MyError` — the glue that makes `?` work on `io::Error`
- Changing `fn foo()` to `fn foo() -> Result<(), MyError>`
- `?` vs `.unwrap()` vs `match` — when to use each

## Your task

Start from this broken program (copy it into your file):

```rust
use std::fs::File;
use std::io::{Read, Write};

fn write_number(path: &str, n: u32) {
    let mut file = File::create(path).unwrap();
    file.write_all(&n.to_le_bytes()).unwrap();
}

fn read_number(path: &str) -> u32 {
    let mut file = File::open(path).unwrap();
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf).unwrap();
    u32::from_le_bytes(buf)
}

fn main() {
    write_number("target/num.bin", 12345);
    let n = read_number("target/num.bin");
    println!("read back: {}", n);

    // This will panic — the file doesn't exist:
    let _ = read_number("target/does_not_exist.bin");
}
```

**Refactor it** so that:

1. You define an error type:
   ```rust
   enum DbError {
       Io(std::io::Error),
   }
   ```

2. `write_number` and `read_number` return `Result<_, DbError>` and use `?` instead of `.unwrap()`.

3. `main` returns `Result<(), DbError>` and uses `?` throughout.

4. `DbError` implements `Display` (print something useful) and `std::error::Error`.

5. The missing-file case prints a helpful error message instead of panicking.

## Hints

<details>
<summary>Hint 1 — the From impl is what makes ? work on io::Error</summary>

When you write `file.open(path)?`, Rust needs to know how to turn `std::io::Error` into your `DbError`. That's the `From` impl:

```rust
impl From<std::io::Error> for DbError {
    fn from(e: std::io::Error) -> Self {
        DbError::Io(e)
    }
}
```

Without this, the compiler will tell you it can't convert the error types. The error message is actually quite clear — read it carefully.
</details>

<details>
<summary>Hint 2 — main can return Result</summary>

```rust
fn main() -> Result<(), DbError> {
    // ...
    Ok(())
}
```

If `main` returns `Err(...)`, Rust prints the error and exits with a non-zero code. That's the right behavior.
</details>

<details>
<summary>Hint 3 — Display implementation</summary>

```rust
impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbError::Io(e) => write!(f, "I/O error: {}", e),
        }
    }
}
```

`std::error::Error` can then be implemented as an empty `impl` — it has default methods that rely on `Display`.
</details>

## Tests to write

Add a `#[cfg(test)]` block and write these tests. Run with `cargo test --example ex04_error_handling`.

1. **`write_and_read_succeeds`** — call `write_number` then `read_number` on the same file, assert the value comes back correctly. Assert the result is `Ok`, not `Err`.
2. **`read_nonexistent_file_returns_err`** — call `read_number` on a path that does not exist, assert the result is `Err`. The test should not panic.
3. **`error_display_is_not_empty`** — construct a `DbError::Io(...)` and call `to_string()` on it (which uses your `Display` impl), assert the string is not empty.

Test 2 is the important one — it proves the refactor actually works. If `.unwrap()` were still in place, the test would panic instead of returning `Err`.

## You're done when

- All tests pass with `cargo test --example ex04_error_handling`.
- No `.unwrap()` or `panic!` anywhere in your code.
- You can explain: what does `?` actually do? What two things does it replace?
- You can answer: when would you still use `.unwrap()` in production Rust code? (Hint: it's for cases that are truly impossible, not just unlikely.)
