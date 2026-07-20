# Phase 1 Spec — Pager & On-Disk Storage

**Goal:** Data survives a process restart. After this phase you own bytes on disk.

**Reading anchor:** DI ch. 3 (file formats). Read it before designing your page layout.

---

## What is and isn't in scope

### In scope

- A `Pager`: a struct that reads and writes fixed-size pages to a file by page number.
- A page layout: a defined byte format for a header + a set of records within a page.
- Record serialization/deserialization by hand — no `serde`.
- Page allocation: handing out a new page ID when new space is needed.
- A free list: reclaiming pages when they're no longer needed.
- Page compaction: defragmenting a page in place (reclaiming space held by deleted/tombstoned records) so a new record
  can fit without allocating a fresh page.
- Wiring the Phase 0 REPL to the pager so `PUT`/`GET`/`DELETE` persist across restarts.

### Explicitly out of scope (do not build yet)

- **Buffer pool / page cache.** In Phase 1 every read goes to disk and every write goes to disk immediately. The buffer
  pool arrives in Phase 3.
- **B-tree.** Your store can be a simple linear scan across pages for now. Phase 2 will replace it with a real index.
- **WAL / crash safety.** A kill mid-write can corrupt state here. That's fine; Phase 3 fixes it.
- **Concurrency.** Single-threaded throughout.

---

## Components to build

### 1. Pager (`src/pager.rs`)

The Pager owns the database file. Its job is simple:

- Open or create a file at a given path.
- Given a page number `n`, read exactly `PAGE_SIZE` bytes starting at offset `n * PAGE_SIZE`.
- Given a page number `n` and a buffer of `PAGE_SIZE` bytes, write them to that offset.
- Allocate a new page: return the next available page number and extend the file.

**What the Pager does NOT do:** it does not interpret the bytes it moves. It is a dumb I/O layer. The meaning of bytes
lives in the page layout layer above it.

**Rust traits you'll use:** `std::fs::File`, `Read`, `Write`, `Seek`. The `?` operator and a custom error type will keep
this clean.

**Design question to answer first:** How does the Pager know how many pages the file already has on startup? Where does
that count come from?

**Design question (optional):** what should happen if the db file is deleted out from under a running process? A real
answer needs a background watcher thread sharing state with the main thread — that's concurrency creep, park it for
Phase 5. A cheap version is enough for now: sample it inline (every Nth operation, check the file still exists) and
raise an error if it's gone.

### 2. Page layout (`src/page.rs`)

A page is `PAGE_SIZE` bytes (4096 is conventional; you can choose). Those bytes are divided into:

- **Header** — fixed-size, at the start of every page. Contains metadata about what's on this page.
- **Record area** — the rest of the page, where actual key/value pairs live.

You need to decide — and document in `docs/decisions/` — what goes in the header. At minimum the header needs to tell a
reader how many records are on this page. Think about what else might be useful.

**Format version & checksum:** consider tagging the page/file format with a version

- version 1 = no checksum,
- version 2 = adds a checksum.

Implementing the checksum yourself (no crate) is the harder but more instructive route if you take it on this phase.

**The slot array / slotted page pattern** (DI ch. 3): one common approach stores records packed from the end of the page
backward, and a small array of (offset, length) entries grows from the header forward. This way adding a record doesn't
require shifting existing ones. You don't have to use this pattern, but read about it and decide consciously.

**Record format:** each record needs to encode at minimum:

- Key length (so you know where the key ends)
- Key bytes
- Value length
- Value bytes

Choose fixed-width length prefixes (e.g., `u16` or `u32`, little-endian). Record your choice and why in
`docs/decisions/`.

**What you must be able to do with a page:**

- Write a key/value record into it, returning an error if there isn't enough space.
- Read record `i` out of it by index.
- Report how much free space remains.
- Report how many records it contains.

The precise bound for that first bullet: `MAX_RECORD_SIZE = PAGE_SIZE - HEADER_SIZE - SINGLE_SLOT_OVERHEAD` (the
per-record cost of a slot array entry — its pointer/offset).

### 3. Free list

When a record is deleted, the page it was on might become empty. You don't want to leak that page — you want to reuse
it.

The simplest approach: a dedicated page (e.g., page 0) that acts as the free list, storing a list of page numbers
available for reuse. On `allocate_page()`, check this list first; only extend the file if the list is empty.

This is intentionally simple. It has one known weakness this phase doesn't have to solve: the free list page itself can
fill up.

Fragmentation is a separate problem worth tackling here: when a page has enough free space in aggregate but no single
gap large enough for a new record, compact it in place (reclaim the space held by deleted/tombstoned records) before
falling back to allocating a fresh page.

### 4. Store integration (`src/store.rs`) — thin, wiring not new design

This isn't a new data structure, it's Pager + Page glued behind the same interface Phase 0's `HashMap` already exposed:

```
put(key, value)
get(key) -> Option<value>
delete(key)
```

Internally it uses the Pager to read/write pages. For Phase 1, a linear scan — read every page, look at every record —
is acceptable. It's slow (`O(n)`) but correct, and Phase 2's B-tree will replace it.

**On delete:** mark the record as deleted somehow (a tombstone byte in the record header, or simply compact the page).
Decide which and document it.

### 5. REPL wiring (`src/main.rs`) — trivial, your test harness, not the point

The REPL from Phase 0 changes only at the store initialization line — swap `HashMap` for your new `Store`. Everything
else stays the same. Its only job here is letting you drive the pager interactively to prove the exit demo works.

---

## Suggested file structure

```
src/
  main.rs       — REPL (minimal changes from Phase 0)
  pager.rs      — Pager struct, file I/O
  page.rs       — Page struct, byte layout, record read/write
  store.rs      — Store struct, put/get/delete via pager
  error.rs      — Your error type (DbError or similar)
```

You don't have to match this exactly. But having `pager.rs` and `page.rs` as separate files is worth it — they have
different responsibilities.

---

## Decisions to document before coding

Create `docs/decisions/` and write a short note for each:

1. **Page size** — Why 4096 (or whatever you pick)?
2. **Page header contents** — What fields, how many bytes each, why?
3. **Record layout** — Key/value length prefix width, byte order, any flags?
4. **Deletion strategy** — Tombstone in place, or compact the page?
5. **Free list design** — Where is it stored, how is it formatted?
6. **Format version & checksum** — start unversioned, or bake in a version byte from day one? If you add a checksum,
   what algorithm, and where does it live in the record/page layout?

These don't need to be long. A few sentences each, plus the actual numbers. Your future self debugging Phase 2 will need
these.

---

## Testing

- Manually corrupt data to confirm data corruption identification:
    - Partial corruption: if only 1 page is corrupted, the other pages should still work fine.
    - Corruption checks even without a checksum — basic sanity checks like `key_len + value_len < MAX_RECORD_SIZE`.
- Best fit vs. first fit allocation.
- Reclamation triggers when a new record can fit only after page compaction, instead of allocating a new page.
- `MAX_RECORD_SIZE` assertion.
- A manual seek shouldn't change the target slot for a subsequent read/write.

---

## Exit demo

```
$ cargo run walrus.db
walrusdb> PUT name alice
OK
walrusdb> PUT city berlin
OK
walrusdb> GET name
alice
walrusdb> ^C

$ cargo run walrus.db
walrusdb> GET name
alice
walrusdb> GET city
berlin
walrusdb> DELETE name
OK
walrusdb> GET name
not found
```

Data survives the restart. That's Phase 1 done.

---

## What you should be able to explain when this phase is complete

- Why databases use fixed-size pages for I/O rather than reading individual records.
- How you lay out a record in bytes: exactly what's at offset 0, offset 2, etc., and how you'd read it back cold.
- Where endianness matters and what you chose.
- What the free list is protecting against and how yours works.
- Why the Phase 1 store is O(n) and what Phase 2 will do about it.