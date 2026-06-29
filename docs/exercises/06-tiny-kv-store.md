# Exercise 6 — Tiny Key-Value File Store

**File to create:** `examples/ex06_tiny_kv.rs`

## Goal

Build a persistent key-value store backed by your block store. It supports `put`, `get`, and `delete` with a linear scan, just like the Phase 1 Store layer will. This is the capstone of the exercise series.

## Why this matters for Phase 1

This is Phase 1's Store layer, shrunk to an exercise. The main difference in `src/store.rs` will be a real page layout with a header, multiple records per page, and a free list. The logic — scan, match key, read value, tombstone on delete — is the same.

## Concepts introduced

- Variable-length binary records (key + value have different lengths each time)
- Length-prefixed encoding: writing a length before the data so you know how much to read back
- Tombstone deletion: marking a record as deleted without removing it
- Linear scan: iterating all blocks to find a key
- Pulling together all five prior exercises

## Record format

Each block (64 bytes) stores **one record** with this layout:

| Offset | Size | Field |
|--------|------|-------|
| 0 | 1 | `status`: `0` = empty, `1` = live, `2` = deleted (tombstone) |
| 1 | 2 | `key_len`: `u16` little-endian |
| 3 | 2 | `value_len`: `u16` little-endian |
| 5 | up to 59 | key bytes followed immediately by value bytes |

The key and value together must fit in 59 bytes. For this exercise, that's an acceptable limit — in Phase 1 you'll have 4096 bytes per page and multiple records per page.

## Your task

Build this struct on top of your `BlockStore` from exercise 5:

```rust
struct TinyKV {
    store: BlockStore,
}

impl TinyKV {
    fn open(path: &str) -> Result<Self, DbError>
    fn put(&mut self, key: &str, value: &str) -> Result<(), DbError>
    fn get(&mut self, key: &str) -> Result<Option<String>, DbError>
    fn delete(&mut self, key: &str) -> Result<(), DbError>
}
```

### Method contracts

**`put`:** Scan all blocks. If you find a live record with this key, overwrite it in place (update the value — if the new value is a different length, tombstone the old block and write a new block). If not found, allocate a new block and write the record there.

**`get`:** Scan all blocks. Return the value of the first live record whose key matches. Skip tombstoned and empty blocks. Return `None` if not found.

**`delete`:** Scan all blocks. Find the live record with this key and set its status byte to `2` (tombstone). Write the block back. If not found, that's fine — `delete` on a missing key is a no-op.

## Then write a main that:

1. Opens `target/tinykvstore.bin`.
2. Puts several key-value pairs.
3. Gets and prints them.
4. Deletes one key.
5. Confirms `get` returns `None` for the deleted key.
6. Terminates and restarts — reopens the file and confirms the surviving keys are still there.

## Hints

<details>
<summary>Hint 1 — encoding a record into a block</summary>

```rust
fn encode_record(key: &str, value: &str) -> Result<[u8; 64], DbError> {
    let k = key.as_bytes();
    let v = value.as_bytes();
    if k.len() + v.len() > 59 {
        return Err(DbError::RecordTooLarge);
    }
    let mut buf = [0u8; 64];
    buf[0] = 1; // live
    buf[1..3].copy_from_slice(&(k.len() as u16).to_le_bytes());
    buf[3..5].copy_from_slice(&(v.len() as u16).to_le_bytes());
    buf[5..5 + k.len()].copy_from_slice(k);
    buf[5 + k.len()..5 + k.len() + v.len()].copy_from_slice(v);
    Ok(buf)
}
```

Try to write `decode_record` yourself — it's the inverse.
</details>

<details>
<summary>Hint 2 — scanning in get / delete</summary>

```rust
for n in 0..self.store.num_blocks()? {
    let block = self.store.read_block(n)?;
    let status = block[0];
    if status != 1 { continue; } // skip empty and tombstoned
    // decode key, compare, act
}
```
</details>

<details>
<summary>Hint 3 — tombstoning in delete</summary>

Read the block, set `buf[0] = 2`, write it back. You're not erasing the data — you're just marking it dead. The next scan will skip it.

This is exactly how deletion works in an LSM-tree (and in your Phase 1 store). The "real" deletion happens during compaction, which is a later phase concern.
</details>

<details>
<summary>Hint 4 — what does this exercise not handle?</summary>

- Multiple records per page (Phase 1 does this with a proper page header)
- A free list for tombstoned blocks (Phase 1 adds this)
- Duplicate keys during `put` when value sizes change (simplify: tombstone + append)

These are not bugs — they're deliberately left for Phase 1. The goal here is to feel the scan-and-tombstone pattern, not to build a production store.
</details>

## Tests to write

Add a `#[cfg(test)]` block and write these tests. Run with `cargo test --example ex06_tiny_kv`. Use unique file paths per test.

1. **`put_and_get_returns_value`** — put a key, get it back, assert the value matches.
2. **`get_missing_key_returns_none`** — get a key that was never put, assert the result is `Ok(None)`.
3. **`delete_removes_key`** — put a key, delete it, get it, assert the result is `Ok(None)`.
4. **`overwrite_existing_key`** — put a key with value `"a"`, put the same key with value `"b"`, get it, assert `"b"` comes back.
5. **`delete_nonexistent_key_is_noop`** — delete a key that doesn't exist, assert no error is returned.
6. **`data_persists_across_reopen`** — put two keys, drop the store, reopen it, get both keys, assert the values are still correct.

Test 6 is the capstone — it's the same thing the Phase 1 exit demo proves.

## You're done when

- All six tests pass with `cargo test --example ex06_tiny_kv`.
- Data survives a process restart (test 6 proves it).
- You can answer: why is linear scan O(n), and why does it get worse over time with deletions? What would fix it? (That's the B-tree. Phase 2.)
- You can see exactly where Phase 1 extends this: a page header that holds multiple records per page, and a free list that recycles tombstoned space.
