# Design

## Layering

KVDB + Record -> Pager -> Page -> PageStore

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

Page layout: header + fixed pointer directory + cells.

- **pointer directory** (Phase 1: fixed capacity):
    - reserved region immediately after the header; does not grow into cell free space
    - capacity hard-capped (see Decisions) — a full directory means the page cannot take another record
    - locator length in the header always describes the full reserved region
    - live entry count is tracked separately; only the live prefix is valid
    - live entries kept sorted by key; lookup binary-searches that prefix
    - shifted on every insert/delete to stay sorted and gap-free within the directory
    - on delete: entry removed outright; the freed cell byte range goes to the availability list
    - unused directory slots are ignored (not free-list-managed)
- **cells**:
    - not a stored structure — live record bytes packed from the end of the page backward
    - never move except during compaction
    - a cell is just a byte range, addressed via its pointer entry
- conceptually, a page holds:
    - a decoded header
    - a decoded pointer directory (live prefix meaningful)
    - the raw bytes for everything after the directory (cell area + free holes)
- a pointer entry, page-absolute, no references:
    - cell location: an offset + a length (plays the role of a RID/TupleID)
    - key bounds within the cell: a key offset + a key length
- the pointer directory's own job is narrowly insert-at-index / remove-at-index bookkeeping over its
  backing list — it does not decode or compare keys
- **TODO**
    - leftover free-range bookkeeping when a chosen free cell is larger than the record being inserted
      (Pager computes the leftover hole; free-list `update` applies it)
    - cell layout (byte-level record encoding within a cell)

### FreeList

Stored in the special page with id 0 (known Phase-1 limitation: this page can itself fill up under
fragmentation — accepted, not fixed, for now).

- **free list**: per-page aggregate free-byte count. Cheap to scan across all pages; cached rather than
  derived from the availability list on every query.
- **availability list**: per-page list of actual free cells and their sizes.
- **what free-list manages**: cell-area holes only. Header and the fixed pointer directory are out of
  scope — never registered as free space.
- **fresh data page**: one initial free range covering everything after header + directory; aggregate
  equals that length.
- **allocation lookup**, two-step:
    1. free list for a candidate page with aggregate free bytes ≥ requested size.
    2. that page's availability list for one cell big enough (aggregate ≥ N doesn't guarantee a single free
       cell ≥ N — classic fragmentation). If no single cell fits, compact that page before falling back to a
       fresh page.
- **requested size** for best-fit is the cell/record byte length only (directory slots are reserved, not
  carved from free space).
- **`update`**:
    - insert/delete change a page's aggregate free-byte total
    - reshaping free ranges alone (leftover after a partial carve, coalesce of adjacent holes) must not
      change the aggregate — same free bytes, redistributed
    - adjacent free holes are coalesced inside `update` so the availability list stays merged
    - reject mutations for page 0 (free-list metadata page)
    - caller (Pager) supplies the post-mutation leftover range(s); free list does not encode slotted
      page geometry
- capabilities: best-fit lookup (size -> page + free range if found), a "fits after compaction" check,
  update, load
- **open questions**
    - whether the availability list needs to live on more than page 0 past toy scale
- **TODO**
    - keep the free list totals consistent with the availability list on every mutating op (see `update`
      rules above)
- **Best-fit lookup algorithm** (candidate-based):
    1. **Candidate selection (free list scan)**:
        - filter pages where `aggregate_free >= requested_size`
        - rank survivors ascending by `aggregate_free - requested_size` (closest fit first)
        - take the top `n` (fixed constant) as candidates
        - fewer than `n` pages may qualify — candidate list is not fixed-size
    2. **Slot selection (availability list scan)**:
        - for each candidate page, in rank order, scan its availability-list entries
        - pick the smallest single cell that still satisfies `cell_size >= requested_size`
        - first candidate with a satisfying cell wins -> return `Found`
    3. **Outcomes**:
        - a candidate has a fitting cell -> `Found(page_num, free_range)`
        - candidates exist but none has a single fitting cell (fragmentation) -> `NeedsCompaction(candidates)`
        - no page satisfies step 1 at all -> `NoSpace`
    4. **Not free-list's job**: directory-full pages. Pager refuses another insert when live count is at
       capacity even if cell free space remains; that is not `NeedsCompaction`.

### Pager

Manages pages.

- read page
- write page
- page-id validation, page count/size, page compaction, version management
- manages the free list via the FreeList module
- pointer-array management
- **position lookup** for a key:
    - binary search over the pointer array
    - uses each entry's key offset/length to slice raw key bytes directly out of the page
    - compares raw bytes directly (byte-lexicographic `Ord`, free in Rust) — no record decoding, no strings
- **next-empty-slot lookup** for a size:
    - best-fit -> if it only fits after compaction, compact then return the slot -> else allocate a fresh page
    - after `Found`, enforce pointer-directory capacity (live count at cap => skip page / allocate fresh)
    - on `NoSpace`, allocate a fresh page, register its initial post-directory free range via free-list
      `update`, then place the record
    - helper scans: an empty-slot scan, a free-list scan
- **owns file growth**: free list never calls allocate; Pager does

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

## Open Validation / Error-Handling TODOs

- **Cell Pointer Validations** - check the validity of cell pointer in `page.rs` and `freelist_manager.rs` and other
  possible places.

## Decisions

- **Naming**: byte ranges are always described as an offset + a length, never start/end — applies to a
  pointer's cell location, its key bounds, and availability-list entries.
- **Terminology**: "pointer array" (not "slot array"), "cells" (not a separate slot/cell split). Deliberately
  not reusing SQLite's file-format naming.
- **The page's raw byte field** (everything after the pointer array) is referred to as "bytes", not "data".
- **Pointer offsets are page-absolute**, not relative to the byte field or the cell's own start:
    - with a fixed directory, the cell-area start is stable; cells still don't move except during compaction
    - absolute offsets stay valid regardless
    - cost: compaction updates a cell's offset and key offset together — but since the whole cell moves as one
      block, it's the same delta applied to both
- **Fixed pointer directory (Phase 1)**:
    - growing the directory into the free gap made one insert touch two different free regions (directory
      growth + best-fit cell hole) — too much variable geometry before insert even works
    - reserve a fixed directory after the header; hard cap on records per page accepted for Phase 1
    - capacity: 64 entries (512 bytes at 8 bytes/entry)
    - header locator length always equals the full reserved directory; live count alone tracks valid entries
    - free list never manages header or directory bytes
    - best-fit requested size = cell length only
    - directory full => Pager treats the page as unable to take another insert (not a compaction outcome)
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
- **Page ID or Page Num ?**
    - Use page_num as standard for phase 1 since it literally depicts the position of the page i.e. page_num = 7 is the
      7th page in the file.
    - page_id can be used in a later phase when the positional information of the page is separate from the identifier.
      i.e. page_id = 7 doesn't necessarily need to be the 7th page in the file.
- **Best-fit lookup is candidate-based, not first-fit**: free-list scan ranks the top `n` pages by closeness of fit
  rather than stopping at the first page that qualifies. Costs a small ranking pass; buys the ability to later tune `n`
  or bias candidate selection based on fragmentation level - giving control over compaction pacing without changing the
  lookup's shape.