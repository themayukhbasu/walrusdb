# WalRusDB — Learning Guide

This maps each phase to **the database concepts to study**, **the Rust concepts you'll meet** (you're new to Rust, so the language is woven in deliberately), and **what you should be able to explain** afterward. The "explain" checks are your interview-readiness bar — if you can't explain it, you haven't finished, regardless of whether the code runs.

**Abbreviations:** **DI** = *Database Internals* (Alex Petrov). **DDIA** = *Designing Data-Intensive Applications* (Martin Kleppmann).

---

## Before Phase 0 — Rust on-ramp

Don't try to learn all of Rust first; learn it *as the project demands it*. But get a minimal floor:

- *The Rust Programming Language* ("the Book"), chapters 1–6 (ownership, structs, enums, `match`).
- `rustlings` for hands-on reps.
- Accept that you'll fight the borrow checker. When you do, that's a journal entry: *what did it reject, and what was it protecting me from?*

---

## Phase 0 — Setup & first win

**DB concepts:** the idea of a key-value store as the substrate beneath everything; the read-eval loop of a database shell.

**Rust concepts:** `cargo`, modules, `String` vs `&str`, `Option<T>`, `HashMap`, basic pattern matching, `Result<T, E>` for the first time.

**Read:** the Book ch. 1–4. DI ch. 1 (introduction & terminology) for vocabulary.

**Be able to explain:**
- Why `Option` exists instead of null.
- The difference between an owned `String` and a borrowed `&str`, and why Rust makes you care.

---

## Phase 1 — Pager / on-disk storage

**DB concepts:** pages as the unit of I/O; why fixed-size pages; record/tuple layout; slotted pages; free-space management.

**Rust concepts:** `&[u8]` slices, `Vec<u8>`, byte manipulation, `from_le_bytes`/`to_le_bytes`, `std::fs::File`, `Read`/`Write`/`Seek` traits, error handling with `Result` and the `?` operator, defining your own error type.

**Read:** DI ch. 3 (file formats) — the central reference for this phase. DDIA ch. 3 (the opening on the simplest possible log-structured store) for intuition.

**Be able to explain:**
- Why databases do I/O in fixed-size pages rather than byte-by-byte.
- How you lay out a record in bytes and read it back, and where endianness matters.
- What a free list is and what problem it solves.

---

## Phase 2 — B-tree index

**DB concepts:** B-tree vs B+-tree; fanout and tree height; why the structure stays balanced; node splits and the special case of splitting the root; range scans via cursors; (stretch) deletion, merging, rebalancing.

**Rust concepts:** recursion with ownership in play, `Box<T>` for owned indirection, why you'll likely route node access *through the pager* rather than holding references into pages (the borrow checker will resist long-lived borrows — understand *why* before you reach for `Rc`/`RefCell`), lifetimes starting to matter.

**Read:** DI ch. 2 (B-tree basics) and ch. 4 (implementing B-trees) — read these slowly, they're the core of the whole project. DI ch. 6 (B-tree variants) if you want depth.

**Be able to explain:**
- Why B-trees are shallow and what that buys you on disk.
- Exactly what happens, step by step, when an insert overflows a leaf — and when it overflows the root.
- Why holding a long-lived reference into a page is dangerous, in both database terms and Rust terms (note how the two reasons are the same reason).

---

## Phase 3 — Buffer pool, WAL & crash recovery

**DB concepts:** buffer management — page cache, pinning, dirty pages, eviction policies (LRU/clock), and the steal/no-steal, force/no-force taxonomy; the write-ahead principle (log before you mutate); atomicity and durability; replay; idempotent recovery; checkpointing; physical vs. logical logging; `fsync` and the durability/performance tradeoff.

**Rust concepts:** designing a cache the borrow checker will accept — who owns cached page bytes and how callers get access to them; interior mutability as a *considered* decision rather than an escape hatch; realizing that a pin count is a hand-rolled runtime borrow. Plus buffered vs unbuffered writes, explicit flushing/syncing, sequencing side effects deterministically, modeling log records as enums.

**Read:** DI ch. 5 (transaction processing & recovery) — the buffer-management opening *and* the recovery portions. DDIA ch. 3 (write-ahead logs in context). Also read DI's ARIES discussion as *read-but-don't-implement*: knowing what a production recovery algorithm handles — and why your simpler scheme deliberately doesn't — is as instructive as what you do build.

**Be able to explain:**
- Why a pinned page must not be evicted — in database terms and in Rust borrow terms, and why those are the same statement.
- Where your design sits in the steal/no-steal, force/no-force space, and what that choice costs.
- Why the log write must hit disk *before* the page write (including before a dirty-page eviction), and what breaks if you reverse them.
- Physical vs. logical logging: what you chose, and what the other choice would have made harder.
- How replay makes recovery idempotent.
- What a checkpoint is and why you need one.
- What `kill -9` actually tests, what it doesn't (`fsync`), and what would.

---

## Phase 4 — Relational layer: schema + tiny SQL

**DB concepts:** the relational model; a system catalog; tokenizing and parsing; mapping rows to keys/values; the difference between *what* a query asks and *how* it's executed.

**Rust concepts:** enums as an AST, iterators and iterator adapters, recursive-descent parsing, `match` at full power, the typestate-ish discipline of turning a string into structured data safely.

**Read:** DDIA ch. 2 (data models & query languages). For parsing technique, *Crafting Interpreters* (Nystrom) — the scanning and parsing chapters — is the friendliest reference even though it's not a database book.

**Be able to explain:**
- The pipeline from SQL text → tokens → AST → execution.
- How a `SELECT ... WHERE` maps onto operations your storage engine already provides.
- Why the catalog is "just data" stored in your own engine.

---

## Phase 5 — Transactions & concurrency

**DB concepts:** ACID precisely (not hand-wavily); isolation levels and the anomalies each prevents; two-phase locking; deadlocks; MVCC and why readers needn't block writers.

**Rust concepts:** the heart of why you chose Rust — `Send`/`Sync`, `Arc<T>`, `Mutex`/`RwLock`, interior mutability, and how the compiler refuses to let you ship a data race. Possibly atomics.

**Read:** DDIA ch. 7 (transactions) — the canonical treatment of isolation levels; read it twice. DI ch. 5 (concurrency control).

**Be able to explain:**
- What each isolation level permits and forbids, with a concrete anomaly example.
- The difference between atomicity (transactions) and durability (the WAL from Phase 3).
- How Rust's `Send`/`Sync` turned a class of concurrency bugs into compile errors, with an example from your own code.

---

## Phase 6 — Query processing

**DB concepts:** logical vs physical plans; secondary indexes; predicate pushdown; cost-based decisions (index scan vs full scan); join algorithms (nested-loop, hash join).

**Rust concepts:** trait objects / enums to represent plan nodes, the iterator/volcano execution model expressed idiomatically, generics where they earn their keep.

**Read:** DDIA ch. 3 (the section on indexes and the OLTP/column distinction). DI for index mechanics. Supplement with any introductory query-optimizer material.

**Be able to explain:**
- Why a query optimizer exists — what goes wrong without one.
- When an index scan beats a full scan, and how a cost model would decide.
- How a nested-loop join works and why a hash join can beat it.

---

## Phase 7 — Capstone: distribution (Raft)

**DB concepts:** replication; consensus; leader election; the replicated log; failure and recovery; the consistency guarantees you can and can't make.

**Rust concepts:** async Rust (likely `tokio`), message passing between nodes, modeling distributed state machines, timeouts and retries.

**Read:** DDIA ch. 5 (replication) and ch. 9 (consistency & consensus). DI Part II (distributed systems). The original Raft paper ("In Search of an Understandable Consensus Algorithm") — it's deliberately readable.

**Be able to explain:**
- Why consensus is hard and what Raft simplifies relative to Paxos.
- How leader election and log replication keep replicas consistent.
- How your replicated log relates to the single-node WAL you built in Phase 3.

---

## A standing habit

For every phase, before you write the "be able to explain" answers as polished prose, try explaining them *out loud to no one* or in a `docs/journal/` entry. The gaps in your explanation are precisely the gaps in your understanding — and they're cheaper to find now than in an interview.
