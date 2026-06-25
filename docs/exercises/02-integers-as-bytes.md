# Exercise 2 — Integers as Bytes

**File to create:** `examples/ex02_integers_as_bytes.rs`

## Goal

Write integers to a binary file as raw bytes and read them back exactly. Understand why byte order (endianness) matters and how Rust handles it.

## Why this matters for Phase 1

Every field in your page header and record format is an integer stored as bytes: the record count, key lengths, value lengths. You need to be able to say "at byte offset 0, I have a `u16` in little-endian; at byte offset 2, I have a `u32` in little-endian" and implement that precisely.

## Concepts introduced

- `u16::to_le_bytes()` / `u32::to_le_bytes()`
- `u16::from_le_bytes()` / `u32::from_le_bytes()`
- Why little-endian (x86 native) is the conventional choice for on-disk formats
- Slicing a byte buffer: `&buf[0..2]`, `[u8; 2]` vs `[u8; 4]`

## Your task

Write a program that:

1. Encodes the following three values into a single byte buffer (a `[u8; 8]` is enough):
   - A `u16` with value `42` at bytes 0–1
   - A `u32` with value `100_000` at bytes 2–5
   - A `u8` with value `7` at byte 6
   - Byte 7 unused (zero)

2. Writes that buffer to `target/ints.bin`.

3. Reads the buffer back from the file into a fresh `[u8; 8]`.

4. Decodes and prints each value, confirming they match what you wrote.

Do the encoding and decoding manually — no structs, no `serde`, just array indexing and the `to_le_bytes`/`from_le_bytes` methods.

## Hints

<details>
<summary>Hint 1 — to_le_bytes gives you an array</summary>

```rust
let n: u32 = 100_000;
let bytes: [u8; 4] = n.to_le_bytes();
```

Now `bytes` is a 4-element array you can copy into a larger buffer.
</details>

<details>
<summary>Hint 2 — copying bytes into a buffer at a specific offset</summary>

```rust
buf[2..6].copy_from_slice(&bytes);
```

`copy_from_slice` requires the source and destination slices to be the same length. If they're not, it will panic. Count carefully.
</details>

<details>
<summary>Hint 3 — from_le_bytes requires a fixed-size array, not a slice</summary>

```rust
let val = u32::from_le_bytes(buf[2..6].try_into().unwrap());
```

`from_le_bytes` takes `[u8; 4]` (an array), but `buf[2..6]` is a `&[u8]` (a slice). `.try_into()` converts the slice to a fixed-size array — it returns a `Result` because the conversion can fail if the length doesn't match, hence the `.unwrap()`.
</details>

## You're done when

- The decoded values are exactly `42`, `100_000`, and `7`.
- Open `target/ints.bin` in a hex editor (or `xxd target/ints.bin`) and read the raw bytes — can you verify by hand that `42` in little-endian is `2a 00`?
- You can answer: what would those same bytes look like in big-endian?
