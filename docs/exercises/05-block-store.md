# Exercise 5 — Block Store (Mini Pager)

**File to create:** `examples/ex05_block_store.rs`

## Goal

Build the Pager in miniature: a struct that treats a file as an array of fixed-size blocks and exposes `read_block(n)` / `write_block(n, data)`. This is structurally identical to what you'll build in `src/pager.rs`.

## Why this matters for Phase 1

This exercise IS the Pager, just with a smaller block size (64 bytes instead of 4096). Once you've done this, `src/pager.rs` is the same code with `PAGE_SIZE = 4096` and a few extra methods.

## Concepts introduced

- Encapsulating file I/O in a struct (not just free functions)
- `impl Drop` — or explicitly flushing on close
- How to know how many blocks exist without storing that count separately (derive it from file size)
- `allocate_block()` — extending the file by one block
- Using all four previous exercises together

## Your task

Implement this struct:

```rust
const BLOCK_SIZE: usize = 64;

struct BlockStore {
    file: std::fs::File,
}

impl BlockStore {
    fn open(path: &str) -> Result<Self, DbError>
    fn read_block(&mut self, n: u64) -> Result<[u8; BLOCK_SIZE], DbError>
    fn write_block(&mut self, n: u64, data: &[u8; BLOCK_SIZE]) -> Result<(), DbError>
    fn allocate_block(&mut self) -> Result<u64, DbError>
    fn num_blocks(&mut self) -> Result<u64, DbError>
}
```

Then write a `main` that:

1. Opens (or creates) `target/blockstore.bin`.
2. Allocates three blocks and writes distinct content to each.
3. Drops and reopens the store.
4. Reads block 1 and prints its contents to confirm persistence.

Use the `DbError` type you built in exercise 4.

## Method contracts

**`open`:** Opens the file in read-write mode, creating it if it doesn't exist. (Hint: `OpenOptions`.)

**`num_blocks`:** Returns `file_size / BLOCK_SIZE`. Use `file.seek(SeekFrom::End(0))` to get the file size without reading anything — seek returns the new position, and seeking to the end returns the file length.

**`allocate_block`:** Seeks to the end of the file, writes `BLOCK_SIZE` zero bytes, and returns the new block number. The new block number is `old_num_blocks` before the write.

**`read_block(n)`:** Seeks to `n * BLOCK_SIZE`, reads exactly `BLOCK_SIZE` bytes. Error if `n >= num_blocks`.

**`write_block(n, data)`:** Seeks to `n * BLOCK_SIZE`, writes all bytes. Error if `n >= num_blocks`.

## Hints

<details>
<summary>Hint 1 — opening for read AND write</summary>

`File::open` is read-only. `File::create` truncates the file. You want read-write without truncation:

```rust
OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .open(path)?
```
</details>

<details>
<summary>Hint 2 — getting the file size via seek</summary>

```rust
let size = self.file.seek(SeekFrom::End(0))?;
```

This moves the cursor to the end and returns the byte position, which equals the file size.

After this, your cursor is at the end. If your next operation reads from a specific block, remember to seek there explicitly — the cursor doesn't reset automatically.
</details>

<details>
<summary>Hint 3 — bounds checking in read_block / write_block</summary>

Before seeking, check `n < self.num_blocks()?`. If not, return an error. This is the same bounds check your real Pager will need.

You'll want a new `DbError` variant for this:
```rust
enum DbError {
    Io(std::io::Error),
    InvalidPage(u64),
}
```
</details>

<details>
<summary>Hint 4 — const generics and array sizes</summary>

`[u8; BLOCK_SIZE]` works as a return type and parameter when `BLOCK_SIZE` is a `const usize`. If the compiler complains about the const in a generic position, try `[u8; 64]` directly — const generics are stable but sometimes need explicit types.
</details>

## You're done when

- Data written before a process restart is readable after reopening.
- `num_blocks()` returns the correct count on both fresh and existing files.
- `read_block` on a nonexistent block number returns an `Err`, not a panic.
- You can see that replacing `64` with `4096` and this file becomes `src/pager.rs`.
