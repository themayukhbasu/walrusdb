# Phase 1 Exercises Journal

# Context: 
While working on Exercise 5 of Phase 1, I encountered the following issue and learned an important concept from it.

### 1. Why `read()` requires `&mut self`

**Issue**

Initially, I assumed that since `read_block()` only reads data from the file, it should take `&self` instead of `&mut self`.

```rust
fn read_block(&self, block_idx: u64) -> Result<[u8; BLOCK_SIZE], DBError>
```

However, Rust required the method to take `&mut self`.

**Fix**

The reason is that reading from a file does not only read bytes—it also advances the file cursor.

The cursor is part of the internal state of `std::fs::File`. Operations like `seek()` and `read_exact()` modify that state, even though they don't modify the file contents.

Since the `File` object itself is mutated, Rust requires mutable access.

```rust
fn read_block(&mut self, block_idx: u64) -> Result<[u8; BLOCK_SIZE], DBError>
```

**What I learned**

- The mutability requirement is about the **file handle**, not the **file contents**.
- Reading changes the file cursor.
- Since the cursor is internal state, `File` is being mutated.
- Therefore methods performing `read()` or `seek()` require `&mut self`.

---

### 2. Hidden side effects of changing the file cursor

**Issue**

During code review, I implemented `num_blocks()` by seeking to the end of the file to determine its size.

```rust
self.file.seek(SeekFrom::End(0))?;
```

Although the function correctly returned the number of blocks, it also left the file cursor positioned at the end of the file.

To avoid this hidden side effect, I updated the implementation to:

1. Save the current cursor position.
2. Seek to the end of the file.
3. Compute the number of blocks.
4. Restore the cursor to its original position.

However, another review comment pointed out that `allocate_blocks()` should explicitly seek to the end before writing new blocks.

**Fix**

The important realization was that functions should never depend on another function accidentally leaving the file cursor in the correct position.

Even though `num_blocks()` restores the cursor, `allocate_blocks()` should still perform its own:

```rust
self.file.seek(SeekFrom::End(0))?;
```

before writing.

This makes the function self-contained and prevents accidental overwriting of existing data if the cursor happens to be somewhere else.

**What I learned**

- Helper functions should avoid hidden side effects.
- Restoring the cursor makes `num_blocks()` safer to use.
- Functions that append data should always seek to the end themselves instead of relying on previous functions.
- Every function should establish the state it expects before performing I/O.

---

### 3. Why implementing `Display` matters for custom errors

**Issue**

After implementing my `DBError` enum, I noticed compiler warnings and realized that my custom errors could only be printed using the `Debug` representation.

```rust
#[derive(Debug)]
enum DBError {
    ...
}
```

Initially, I thought deriving `Debug` was sufficient.

**Fix**

By implementing the `Display` trait, I could define how each error should be presented in a user-friendly manner.

```rust
impl fmt::Display for DBError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DBError::Io(e) => write!(f, "I/O Error: {}", e),
            DBError::BlockOutOfBounds(block) => {
                write!(f, "Block {} is out of bounds", block)
            }
        }
    }
}
```

This allows errors to be printed naturally using:

```rust
println!("{}", err);
```

instead of relying only on:

```rust
println!("{:?}", err);
```

**What I learned**

- `Debug` is intended for developers and debugging.
- `Display` is intended for user-friendly output.
- Implementing `Display` makes a custom error behave like standard Rust errors and provides meaningful error messages.