# File Handling Exercises

Six exercises that take you from your current level to being ready to build the Phase 1 Pager.

Each exercise is a standalone `examples/` file. Write it yourself; the markdown just describes the task.

## Progression

| # | File | Concept introduced | Phase 1 skill it builds |
|---|------|--------------------|--------------------------|
| 1 | `ex01_seek.rs` | `Seek`, reading at a byte offset | Pager reads page N at offset `n * PAGE_SIZE` |
| 2 | `ex02_integers_as_bytes.rs` | `to_le_bytes` / `from_le_bytes` | Record header: encoding lengths and flags as integers |
| 3 | `ex03_fixed_records.rs` | Fixed-size binary struct layout | Page layout: writing a struct to bytes, reading it back |
| 4 | `ex04_error_handling.rs` | Custom error type, `?` operator | Clean error propagation throughout the Pager |
| 5 | `ex05_block_store.rs` | Block-indexed file, `write_block` / `read_block` | The Pager itself |
| 6 | `ex06_tiny_kv.rs` | Variable-length records, linear scan, tombstones | The Phase 1 Store layer |

Do them in order. Each one uses what the previous one taught.