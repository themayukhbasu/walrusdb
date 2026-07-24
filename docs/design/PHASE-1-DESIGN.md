# Design

## Layering

KVDB + Record → Pager → Page → PageStore

- **PageStore**: dumb I/O, byte offsets only, never interprets.
- **Page**: pure layout model — header / pointer array / cells. No decode, no compare.
- **Pager**: page management (free list, compaction, page/cell lookup) + raw byte-level key comparison for
  position lookup. No record decoding.
- **KVDB + Record**: owns the record format — full encode/decode. Computes key bounds when writing, hands
  them down as plain numbers.

## Modules

### PageStore

Lowest-level API for filesystem I/O.

- read, write, and an allocate operation
- allocate:
  - knows nothing about page numbers — only byte offsets
  - extends the file by one zeroed page-sized block, returns the offset it just created
  - that offset comes from the write itself, not a separate "check current size" call — avoids a race
    between "ask for size" and "allocate"

### Page

Page layout: header + pointer array + cells.

- **pointer array**:
  - one fixed-size entry per live cell — no entries for free space
  - kept sorted by key; lookup binary-searches it
  - shifted on every insert/delete to stay sorted and gap-free
  - on delete: entry removed outright; the freed byte range goes to the availability list
- **cells**:
  - not a stored structure — live record bytes packed from the end of the page backward
  - never move except during compaction
  - a cell is just a byte range, addressed via its pointer entry
- conceptually, a page holds:
  - a decoded header
  - a decoded pointer array
  - the raw bytes for everything after the pointer array
- a pointer entry, page-absolute, no references:
  - cell location: an offset + a length (plays the role of a RID/TupleID)
  - key bounds within the cell: a key offset + a key length
- the pointer array's own job is narrowly insert-at-index / remove-at-index bookkeeping over its backing
  list — it does not decode or compare keys
- **TODO**
  - split mechanics when a free cell found via the availability list is larger than the record being
    inserted (where the leftover goes, whether the pointer array can grow by more than one entry per insert)
  - cell layout (byte-level record encoding within a cell)

### FreeList

Stored in the special page with id 0 (known Phase-1 limitation: this page can itself fill up under
fragmentation — accepted, not fixed, for now).

- **free list**: per-page aggregate free-byte count. Cheap to scan across all pages; cached rather than
  derived from the availability list on every query.
- **availability list**: per-page list of actual free cells and their sizes.
- **allocation lookup**, two-step:
  1. free list for a candidate page with aggregate free bytes ≥ requested size.
  2. that page's availability list for one cell big enough (aggregate ≥ N doesn't guarantee a single free
     cell ≥ N — classic fragmentation). If no single cell fits, compact that page before falling back to a
     fresh page.
- capabilities: best-fit lookup (size → page + slot if found), a "fits after compaction" check, update, load
- **open questions**
  - best-fit when free space is spread across several slots that individually don't fit but sum to enough —
    do we split slots? what does the return value mean then?
- **TODO**
  - keep the free list totals consistent with the availability list on every mutating op (split/coalesce
    must not change a page's aggregate total; only insert/delete should)
  - whether the availability list needs to live on more than page 0 past toy scale

### Pager

Manages pages.

- page-id validation, page count/size, page compaction, version management
- manages the free list via the FreeList module
- pointer-array management
- **position lookup** for a key:
  - binary search over the pointer array
  - uses each entry's key offset/length to slice raw key bytes directly out of the page
  - compares raw bytes directly (byte-lexicographic `Ord`, free in Rust) — no record decoding, no strings
- **next-empty-slot lookup** for a size:
  - best-fit → if it only fits after compaction, compact then return the slot → else allocate a fresh page
  - helper scans: an empty-slot scan, a free-list scan

### Record

- record status, record layout
- encode, decode, size

### KVDB

Manages records; doesn't know about pages.

- public: init, get, put, delete, dump
- private: scan live records, write, read
- **TODO**: update pointer (logical/alphabetic order), update header

### Entry point

- init db, REPL

## Decisions

- **Naming**: byte ranges are always described as an offset + a length, never start/end — applies to a
  pointer's cell location, its key bounds, and availability-list entries.
- **Terminology**: "pointer array" (not "slot array"), "cells" (not a separate slot/cell split). Deliberately
  not reusing SQLite's file-format naming.
- **The page's raw byte field** (everything after the pointer array) is referred to as "bytes", not "data".
- **Pointer offsets are page-absolute**, not relative to the byte field or the cell's own start:
  - the byte field's start shifts as the pointer array grows/shrinks, but cells don't move except during
    compaction
  - absolute offsets stay valid regardless
  - cost: compaction updates a cell's offset and key offset together — but since the whole cell moves as one
    block, it's the same delta applied to both
- **PageStore's allocate** eagerly writes real zero bytes rather than a sparse/logical extension — simpler,
  portable, avoids a page being "allocated" without disk space backing it. Sparse-file provisioning parked
  for a later phase.
- **Delete** removes the pointer entry outright (no tombstoning in place); the freed cell goes to the
  availability list.
- **Accepted duplication**: key boundaries exist both inside the cell's own bytes (as part of the record
  header) and explicitly in the pointer entry. Same shape as the free list mirroring the availability list —
  a cache for speed, not the source of truth. Mitigation: checksum validation to catch drift (not yet
  designed — see `PHASE-1-SPEC.md` decision #6).
- **Insert's borrow shape** (general pattern, applies wherever a lookup precedes a mutation):
  - the position lookup takes only read access and returns a plain index, before any mutation begins
  - the mutation phase (shift pointers, write bytes, bump header count) runs after, each step
    taking/releasing its own mutable access
  - general rule: borrow-checker pain comes from a borrow outliving the read it was taken for, not from a
    function touching multiple fields/structures