# Phase 3 Spec — Buffer Pool, WAL & Crash Recovery

**Goal:** Pages live in a real in-memory cache with an explicit ownership story, and the database survives being killed mid-write. Durability and atomicity at the single-operation level.

**Reading anchor:** DI ch. 5 — the buffer-management opening *and* the recovery portions. DDIA ch. 3 for WALs in context. Read DI's ARIES discussion as *read-but-don't-implement*.

---

## What is and isn't in scope

### In scope
- A **buffer pool**: in-memory page cache with pinning, dirty-page tracking, and eviction. All page access routes through it.
- A **WAL**: append-only log file, a record per change, fsync at deliberate points.
- **Write-path ordering** that upholds the write-ahead invariant everywhere — including eviction.
- **Replay** on startup, idempotently.
- A **checkpoint/truncation** strategy so the log doesn't grow forever.
- Crash testing at deliberately awkward moments.

### Explicitly out of scope (do not build yet)
- **Multi-operation transactions.** One `PUT`/`DELETE` = one atomic unit. `BEGIN`/`COMMIT` and transaction boundaries in the log are Phase 5.
- **Concurrency.** Single-threaded still. (But your buffer pool's ownership design is a dress rehearsal for Phase 5 — design like the concurrent version is coming, because it is.)
- **ARIES.** Read about it; do not build it. Your recovery scheme will be simpler — knowing *which* ARIES problems you're allowed to skip (and why) is part of the phase.
- **Group commit / performance tuning** of fsync. Correct first.

---

## Components to build

### 1. Buffer pool (`src/buffer.rs`)

Sits between the store/B-tree and the Pager. A cache of frames keyed by page ID, each frame holding the page bytes plus bookkeeping: pin count, dirty flag.

**Design questions to answer first:**
- **The API — and this is the phase's Rust boss-fight:** what does `get_page(page_id)` *return*? A `&mut [u8]` borrows the whole pool for as long as it lives; the borrow checker will make that hurt, and it's right to — that's exactly the aliasing a concurrent Phase 5 couldn't allow either. Work through the options (copy out / explicit `pin`+`unpin` protocol / an RAII guard whose `Drop` unpins) and understand what each trades before choosing. This decision is the vision's "who owns a page and for how long" question made concrete — give it a full `docs/decisions/` note.
- **Eviction:** when the pool is full and a miss needs a frame, which page goes? Start with something simple (LRU or clock). What *categorically cannot* be evicted? (Pinned pages — and once the WAL exists, a second rule appears; see component 3.)
- **Pool size:** fixed frame count. Small on purpose (e.g., 8–16 frames) so eviction actually happens in tests — a pool bigger than your test data tests nothing.
- What happens on `flush` — and who calls it, when?

### 2. WAL format (`src/wal.rs`)

An append-only file, separate from the database file, holding one record per change.

**Design questions:**
- **Physical or logical logging?** Physical: "page N, offset X, these bytes." Logical: "PUT key=…, value=…". This is the phase's central design decision — it shapes replay, checkpoint, and Phase 5's rollback (logical undo needs to know the *operation*; physical undo needs before-images). Decide first, document in `docs/decisions/`.
  - **Note from Phase 1:** page compaction (in-place repack of a fragmented page, same page ID, no key/value semantics involved) has no natural logical record — it's not a client operation. A physical before/after page image logs it trivially; a logical scheme needs an explicit answer for it. Account for this case when you decide, don't discover it mid-implementation.
- What's in a record besides the payload? At minimum you need framing (a length prefix) so records can be read back, and a way to detect a **torn tail** — the process died mid-append and the last record is garbage. How does replay recognize and safely ignore it? (A checksum per record is the standard answer; a length that runs past EOF is the cheap version.)
- Do records carry a sequence number (LSN)? Not strictly required for Phase 3's scheme — but think about what replay-idempotence mechanism you're choosing (component 4) before deciding.
- Model record types as a Rust **enum**; hand-roll the serialization exactly like Phase 1 records (this is deliberate reps).

### 3. Write-path ordering

The invariant: **the log record reaches disk before the page change does.** Concretely, that means two rules:

1. On each operation: append the WAL record, fsync the WAL, *then* mutate the page in the buffer pool. (Note: mutate in *memory* — the page doesn't need to hit disk at all yet. Sit with why that's still durable.)
2. On eviction: a dirty page must not be written to disk before the WAL records that dirtied it are durable. With rule 1's fsync-per-record this holds trivially — but state the rule explicitly in code or comments, because the moment you relax fsync frequency (a natural optimization), this is the rule that silently breaks.

**Design question:** where exactly does fsync go, and what does each placement cost? Every record (slow, simple, correct) vs. every operation vs. batched. Start correct; measure later.

### 4. Replay (`recover()` on startup)

Read the WAL front to back; re-apply each record to bring pages to a consistent state.

**Design questions:**
- **What makes replay idempotent?** Re-running recovery (or replaying a record whose effect already reached the data file) must be harmless. Two standard mechanisms: naturally idempotent records ("set key to value" — applying twice is a no-op), or comparing a per-page LSN against the record's. Which does your record format from component 2 give you? If neither, redesign the format, not the replay loop.
- When does replay stop, and what does it do with the torn tail?
- After successful replay: what state are the buffer pool, data file, and WAL in? Draw it.

### 5. Checkpoint & truncation

Without this the WAL grows forever and recovery time grows with it. Now precisely definable: **flush all dirty pages to the data file, fsync the data file, then truncate the WAL** — every logged change is now in the data pages, so the log is dead weight.

**Design questions:**
- What triggers a checkpoint — WAL size threshold? Every N operations? On clean shutdown?
- What is the exact ordering of the three steps above, and what happens if you crash *between* any two of them? (Walk each gap; this is a great journal entry.)

### 6. Crash testing

The agent may scaffold the harness (spawning the process, killing it at signals/timings, checking invariants after restart) — the *kill points* and the *invariants checked* are yours to choose.

Awkward moments worth hitting: mid-WAL-append (torn tail), after WAL fsync but before the in-memory page mutation, mid-checkpoint (each gap from component 5), immediately after truncation, mid-compaction-rewrite (the Phase 1 page-compaction overwrite is a single non-atomic write like any other page mutation — it needs the same write-ahead protection, so kill it mid-flight too).

---

## Decisions to document before coding

1. **Buffer pool API & ownership** — what `get_page` returns; pin/unpin protocol; why.
2. **Eviction policy** — and the two never-evict rules.
3. **Logging granularity** — physical vs. logical, and the Phase 5 rollback implications you're accepting.
4. **fsync policy** — where, how often, what it costs.
5. **Torn-tail detection** — checksum, length-vs-EOF, or other.
6. **Checkpoint trigger and ordering** — with the crash-between-steps analysis.

---

## Exit demo

```
$ cargo run walrus.db &
walrusdb> PUT balance 100
$ kill -9 <pid>        # at a deliberately awkward moment (use the harness)

$ cargo run walrus.db
walrusdb> GET balance
100                    # or "not found" — but never garbage, never a corrupt file
```

Every operation is either fully applied or fully absent after any kill. That's Phase 3 done.

> **What the demo actually proves:** `kill -9` tests write *ordering* and replay — not `fsync` durability, because the OS page cache survives process death. Only power loss (or a fault-injection layer) exercises fsync. Know precisely which guarantee your demo demonstrates; "what does kill -9 actually test?" is a classic interview trap.

---

## What you should be able to explain when this phase is complete

- Why a pinned page must not be evicted — in database terms and Rust borrow terms, and why those are the same statement.
- Where your design sits in the steal/no-steal, force/no-force space (yes, your design *is* somewhere in that space — find it), and what the position costs.
- Why the log write must hit disk before the page write — including before eviction — and what breaks if reversed.
- Physical vs. logical logging: your choice, and what the other would have made harder.
- How your replay is idempotent, concretely.
- What a checkpoint is, your trigger, and what happens if you crash between its steps.
- What `kill -9` actually tests, what it doesn't, and what would.
