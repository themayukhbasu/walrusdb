# Design

## Layering

- `KVDB + Record`  →  `Pager`  →  `Page`  →  `PageStore`
- `PageStore`: dumb I/O. Byte offsets only. Never interprets.
- `Page`: pure layout model. Just describes where header / pointer array / cells sit. No decode, no compare.
- `Pager`: page management (free list, compaction, which page/cell) + raw `&[u8]` key comparison for
  `find_position`, using `key_start`/`key_end` off `Pointer` (see `page.rs` below). No `Record::decode`.
- `KVDB + Record`: owns the record format. Full `Record::decode`/`encode`. Computes `key_start`/`key_end` when
  writing, hands them down as plain numbers.

### PageStore should provide the lowest API for fs io operations
- PageStore.rs
  - file
  - write
  - read
  - allocate
    - knows nothing about page numbers — only byte offsets. `allocate` extends the file by one `PAGE_SIZE` zeroed
      block and returns the `u64` offset it just created (the value comes from the `seek` done internally, not
      re-derived by the caller via a separate `size()` call — avoids a race between "ask for size" and "allocate").
    - decided against `File::set_len` (sparse/logical extension) in favor of an eager real write of zero bytes —
      simpler, portable, avoids a page being "allocated" without disk space actually backing it. Parked the
      sparse-file / disk-provisioning rabbit hole for a later phase.

### Page: the page related models
- page.rs
  - page layout struct
  - page header struct
  - page status enum
  - terminology: **pointer array** (not "slot array") + **cells** (not a separate "slot"/"cell" split) — page =
    header + pointer array + cells. Deliberately not using SQLite's file-format naming; picking our own and
    learning by getting it wrong if needed.
  - **pointer array**:
    - holds one fixed-size entry per **live** record cell only — free space is not represented here at all.
    - kept sorted by key at all times, so lookup within a page can binary search it instead of scanning linearly.
    - rearranged (entries shifted) on every insert/delete to stay sorted and gap-free. Cheap because we're
      shifting small fixed-stride pointer entries, never the variable-length cell bytes themselves.
    - on delete: the pointer entry for that key is removed outright (not retagged/tombstoned in place) — the
      freed cell's `(offset, length)` gets handed to the availability list instead (see freelist.rs below).
  - **cells**: not a stored struct/collection. Live record bytes packed from the end of the page backward; never
    move once written except during explicit compaction. A page has **no memory of its own free space** — freed
    byte ranges are tracked entirely externally (availability list), not by the page itself.
  - **struct split**: `Page` holds `header: Header` (decoded), `pointer_array: PointerArray` (decoded), and
    `data: Vec<u8>` — raw bytes for everything after the pointer array. No persistent `Cell` struct or `Vec<Cell>`:
    a "cell" is just a byte range inside `data`, addressed on demand via a `Pointer`'s `offset`/`length`. Avoids
    needing any identity/id scheme for cells — same as how live cells are already located without one.
  - **naming convention**: everywhere a byte range needs describing, use `offset` + `len` (or a `_offset`/`_len`
    suffixed pair) — not `start`/`end`. Consistent across `Pointer.offset`/`length`, `Pointer.key_offset`/`key_len`,
    and the availability list's free-cell entries.
  - **Pointer** fields, all plain values (no references), **all absolute from page byte 0**:
    - `offset`, `length` — location of the cell. Plays the same role as a `RID`/`TupleID` in a real engine: stable
      indirection without pinning anything in memory.
    - `key_offset`, `key_len` — bounds the key within the cell.
    - why absolute rather than relative to `data`'s start or the cell's own start: `data`'s start boundary moves
      every time `PointerArray` grows/shrinks, but cells never move except during compaction — absolute offsets
      stay correct regardless of `PointerArray`'s current size. Cost: compaction has to update `key_offset`
      alongside `offset` when it relocates a cell — but since the whole cell moves as one contiguous block, both
      shift by the same delta, so it's one computed delta applied twice, not two independently-drifting values.
  - **PointerArray**'s job is narrowly "shift" — insert-at-index / remove-at-index, pure array bookkeeping on its
    own `Vec` of pointers. It does not decode or compare keys itself.
  - **find_position** lives in `Pager`, not `Page` and not `PointerArray`:
    - uses `Pointer.key_offset`/`key_len` to slice raw key bytes straight out of `data` and compares `&[u8]`
      (byte-lexicographic `Ord`, free in Rust) — no `Record::decode`, no `String`.
    - to index into `data`: `data_start = header.pointer_array_offset + header.pointer_array_len`, then
      `data[pointer.key_offset - data_start .. ]` (since `pointer.key_offset` is page-absolute, `data` is not).
    - `key_offset`/`key_len` are computed by `KVDB`/`Record` (they own the record format) and handed down as plain
      numbers when writing.
  - **accepted duplication**: key boundaries exist in two places — implicitly inside the cell's own bytes
    (`key_len` in the record header) and explicitly in `Pointer.key_offset`/`key_len`.
    - deliberate, same shape as `free_list` mirroring `availability_list`: a cache for speed, not the source of
      truth.
    - mitigation: checksum validation to catch drift between the two. Not yet designed — needs its own note once
      the checksum approach (see `PHASE-1-SPEC.md` decision #6) is picked.
  - **borrow-checker shape of `Page::insert`**:
    1. `find_position` (in `Pager`) takes only `&` access to `pointers`/`data`, returns a plain `usize`. That
       borrow ends there.
    2. Mutation phase (shift `pointers`, write `data`, bump `header`'s count) runs sequentially after, each step
       taking and releasing its own `&mut` — never overlapping with another live borrow.
    - general rule: borrow-checker pain comes from a borrow *outliving* the read it was taken for, or a struct
      storing a reference inside itself that outlives what it points to — not from a function touching multiple
      fields/structs, and not from `Vec`-backed fields.
  - **TODO**
    - exact split mechanics when a free cell found via the availability list is larger than the record being
      inserted (where the leftover free cell's bookkeeping goes, whether the pointer array ever needs to grow by
      more than one entry per insert)
    - cell layout (byte-level record encoding within a cell)

### FreeList manager should provide APIs to manage the free/availability list
> stored in special page with page_id = 0 (hard limit for now — accepting the known Phase-1 weakness that this
> page itself can fill up if fragmentation gets bad; documented as a conscious simplification, not an oversight)
- freelist.rs
  - two distinct structures, kept separately:
    - **free_list**: per-page *aggregate* total free bytes. Small, fixed-size per page, cheap to scan across all
      pages. Technically derivable from the availability_list, but cached separately to avoid rescanning every
      cell in every page just to answer "does any page have ~N bytes free."
    - **availability_list**: per-page list of free cells and their sizes — the actual byte-level detail of
      *which* ranges are free, not just how many bytes.
  - allocation lookup is two-step:
    1. check `free_list` for a candidate page whose aggregate free bytes ≥ requested size (best fit at the page
       level).
    2. check that page's `availability_list` for an actual single cell big enough. Aggregate ≥ N does **not**
       guarantee a single free cell ≥ N exists (classic fragmentation case: several small free cells that sum to
       enough but none big enough alone) — if no single cell fits, trigger compaction on that page before
       falling back to allocating a fresh page via PageStore.
  - freelist layout struct
  - best_fit(size) -> Some(page_id, slot_idx) : give the slot index for best fit location
    - what if there are 3 slots in a page but there is total 60% free size, then what does this return?
    - do we break up slots? which would also mean we allocate slots ??
    - or do we manage in some other way?
  - can_fit_after_compaction(size)
  - update
  - load
  - **TODO**
    - keeping `free_list` totals consistent with `availability_list` on every mutating op (insert consumes/splits
      a cell, delete frees one, compaction consolidates) — splitting/coalescing shouldn't change a page's
      aggregate total, only insert/delete should
    - whether availability_list needs to live on more than just page 0 once it's not a toy-size database

> pd
### Pager should manage the pages
- pager.rs
  - validate_page_id
  - num_pages / size
  - compact_page
  - manage versions
  - manage free list using freelist manager API
  - pointer slot array management
  - next_empty(size):
    > this should basically check the free list for best_fit(size), return (page_id, slot_idx) if found
    > if can_fit_after_compaction, trigger compaction, return (page_id, slot_idx)
    > else allocate_page
    - scan_empty(size)
    - scan_free_list(size)

### Record deals with record APIs
- record.rs
  - record status enum
  - record layout struct
  - encode
  - decode
  - size: returns size of record

### KV DB should only manage on a record level
- kvdb.rs
  - pub
    - init
    - get key
    - put key value
    - delete key
    - dump
  - not pub:
    - scan_live
    - write
    - read
    - **TODO** update pointer (in logical order - in our case alphabetic)
    - update header

- main.rs
  - init db
  - repl