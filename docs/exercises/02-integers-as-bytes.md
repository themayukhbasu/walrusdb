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

## Tests to write

Add a `#[cfg(test)]` block and write these tests. Run with `cargo test --example ex02_integers_as_bytes`.

1. **`encode_decode_roundtrip`** — encode the three values into a buffer, write to a file, read back into a fresh buffer, decode, assert all three values match exactly.
2. **`u16_is_little_endian`** — encode `42u16`, check that the byte at index 0 is `0x2A` and the byte at index 1 is `0x00`. This verifies the byte order, not just the round-trip.
3. **`u32_correct_byte_layout`** — encode `100_000u32` at bytes 2–5, check the individual bytes match what you'd expect in little-endian (`0xA0, 0x86, 0x01, 0x00`).

For tests 2 and 3 you don't need a file — just encode into a buffer and inspect the bytes directly.

## You're done when

- All tests pass with `cargo test --example ex02_integers_as_bytes`.
- You can run `xxd target/ints.bin` and read the raw bytes, verifying by hand that `42` in little-endian is `2a 00`.
- You can answer: what would those same bytes look like in big-endian?
