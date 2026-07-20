# Design

### PageStore should provide the lowest API for fs io operations
- page_store.rs
  - file
  - write
  - read
  - allocate

### Page: the page related models
- page.rs
  - page layout struct
  - page header struct
  - page status enum
  - **TODO**
    - slot layout struct
    - cell layout

> mb
### FreeList manager should provide APIs to manage the free/availability list
> freelist is stored in special page with page_id = 0
- freelist.rs
  - freelist layout struct
  - best_fit(size) -> Some(page_id, slot_idx) : give the slot index for best fit location
    - what if there are 3 slots in a page but there is total 60% free size, then what does this return?
    - do we break up slots? which would also mean we allocate slots ??
    - or do we manage in some other way?
  - can_fit_after_compaction(size)
  - update
  - load

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