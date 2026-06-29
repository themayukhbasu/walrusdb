# Exercise 3 — Fixed-Size Binary Records

**File to create:** `examples/ex03_fixed_records.rs`

## Goal

Combine exercises 1 and 2: write a sequence of structured binary records to a file, seek to any one by index, and decode it back correctly.

## Why this matters for Phase 1

This is a simplified version of your page layout. A page is a fixed-size block that holds multiple records. Before tackling variable-length records, it helps to nail the fixed-size case — where seeking to record `i` is arithmetic, not parsing.

## Concepts introduced

- Encoding a struct as a fixed-size byte array by hand
- Seeking to record `i` using a computed offset
- Isolating encode/decode into functions (not methods yet — just functions)

## Your task

Define a "player record" with these fields:

| Field | Type | Bytes |
|-------|------|-------|
| `score` | `u32` | 4 |
| `level` | `u16` | 2 |
| `active` | `u8` (0 or 1) | 1 |
| padding | — | 1 (zero, reserved) |

Total: **8 bytes per record**.

Write these functions:

```rust
fn encode(score: u32, level: u16, active: bool) -> [u8; 8]
fn decode(bytes: [u8; 8]) -> (u32, u16, bool)
```

Then write a program that:

1. Encodes and writes **four records** to `target/records.bin`.
2. Seeks to **record index 2** (the third one) and reads its 8 bytes.
3. Decodes and prints the fields to confirm correctness.
4. Does the same for record index 0.

No loops required — just hardcode four records with different values so you can tell them apart.

## Hints

<details>
<summary>Hint 1 — building the encode function</summary>

Start with a `[u8; 8]` buffer filled with zeros, then write each field into the right slice:

```rust
fn encode(score: u32, level: u16, active: bool) -> [u8; 8] {
    let mut buf = [0u8; 8];
    buf[0..4].copy_from_slice(&score.to_le_bytes());
    // ... fill in level at bytes 4-5, active at byte 6
    buf
}
```
</details>

<details>
<summary>Hint 2 — offset arithmetic</summary>

If each record is 8 bytes, record `i` starts at byte `i * 8`. This is the same math as the Pager, just with a smaller block size.
</details>

<details>
<summary>Hint 3 — what does padding buy you?</summary>

Nothing in this exercise. But in real formats, padding keeps records aligned to powers of two, which matters for memory-mapped I/O. That's a Phase 2+ concern — for now just zero it out.
</details>

## Tests to write

Add a `#[cfg(test)]` block and write these tests. Run with `cargo test --example ex03_fixed_records`.

1. **`encode_decode_roundtrip`** — call `encode(1000, 5, true)`, pass the result to `decode`, assert the output matches `(1000, 5, true)`. No file I/O needed — just test the functions as pure transformations.
2. **`decode_encode_roundtrip`** — same idea in reverse: construct a known `[u8; 8]` by hand and assert `decode` gives the right values.
3. **`seek_to_record_2`** — write four records to a file, seek to index 2, decode, assert the correct values. Use a unique file path.
4. **`seek_to_record_0`** — same, but record index 0, to confirm the first record is not accidentally skipped.

Tests 1 and 2 don't need files and can run in parallel safely.

## You're done when

- All tests pass with `cargo test --example ex03_fixed_records`.
- You can seek to any record by index and decode the correct values.
- You can see how, if the record size were `PAGE_SIZE` instead of 8, these functions would be your Pager's `read_page` and `write_page`.
