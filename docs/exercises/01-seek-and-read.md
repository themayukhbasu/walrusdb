# Exercise 1 — Seek and Read at a Byte Offset

**File to create:** `examples/ex01_seek.rs`

## Goal

Read from and write to a specific position in a file, not just from the beginning or end.

## Why this matters for Phase 1

The Pager reads page N by seeking to offset `n * PAGE_SIZE` in the file and reading exactly `PAGE_SIZE` bytes. This exercise makes that feel natural.

## Concepts introduced

- `std::io::Seek` and `SeekFrom`
- `file.seek(SeekFrom::Start(offset))`
- `file.read_exact(&mut buf)` — reads exactly `buf.len()` bytes or errors
- `file.write_all(&buf)` — writes all bytes or errors
- Fixed-size byte buffers: `[u8; N]`

## Your task

Write a program that:

1. Creates a file `target/slots.bin`.
2. Writes **three "slots"** into it, each exactly 32 bytes. Fill them with recognizable content so you can tell them apart — e.g., slot 0 filled with `b'A'`, slot 1 with `b'B'`, slot 2 with `b'C'`.
3. Closes (drops) the file.
4. Reopens the file for reading.
5. Seeks directly to **slot 1** (not slot 0) and reads 32 bytes.
6. Prints the bytes to confirm you got the right slot.

No strings. No `read_to_string`. Work with `[u8; 32]` buffers.

## Hints

<details>
<summary>Hint 1 — how to fill a buffer with a single byte value</summary>

```rust
let buf = [b'B'; 32];
```

That's a fixed array of 32 bytes, all set to the ASCII value of `'B'`.
</details>

<details>
<summary>Hint 2 — how to seek</summary>

```rust
use std::io::{Seek, SeekFrom};

file.seek(SeekFrom::Start(offset_in_bytes))?;
```

`SeekFrom::Start` takes an absolute byte position from the beginning of the file. What is the byte offset of slot 1 if each slot is 32 bytes?
</details>

<details>
<summary>Hint 3 — read_exact vs read</summary>

`file.read(&mut buf)` is allowed to return fewer bytes than you asked for. `file.read_exact(&mut buf)` either fills the buffer completely or returns an error. For fixed-size pages you always want `read_exact`.
</details>

## You're done when

- The program prints 32 `B` bytes (not `A`, not `C`).
- You can change the index from 1 to 0 or 2 and get the right slot back.
- You understand: given a slot size `S` and slot number `n`, the byte offset is `n * S`. That is the Pager.