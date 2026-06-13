# WalRusDB — Phased Plan

The committed roadmap for building WalRusDB, phase by phase.

## How to read this plan

- Phases are **milestones**, not time-boxes. There is deliberately **no hour or week estimate** attached — Rust's learning curve makes that noise. A phase is done when its **exit demo** passes, not when a clock runs out.
- Each phase is broken into **steps** sized to give you frequent, satisfying wins. Knock them off one at a time.
- Every phase names what **you build** (learning-critical, yours alone) versus what an **agent may scaffold** (plumbing). When in doubt: *if writing it teaches you a database or Rust concept, you write it.*
- The **exit demo** is a concrete thing you can run and show. It is the definition of "this phase is done."
- Difficulty is relative within this project: Easy / Medium / Hard.

## Design decisions (recap, so the plan reads in context)

- **B-tree** is the storage core (canonical relational design; maps cleanly onto Rust ownership; LSM is a later comparison, not the spine).
- The **relational/SQL layer is in the spine**, because a SQL prompt is the demo.
- Relational sits **after durability, before transactions**, so the first "done" is a crash-safe SQL database reached without first conquering the balloon-prone transactions phase. (Tradeoff: you'll retrofit transaction semantics into the query layer later. That's acceptable and itself instructive.)

---

## Phase 0 — Setup & first win  *(Easy)*

**Goal:** A working build-run-experiment loop, and your first taste of Rust, before anything hard.

**Steps:**
1. Install the toolchain (`rustup`, `cargo`), create the project, get `cargo run` working.
2. Build a REPL: read a line, parse a command word, print something back.
3. Back it with an in-memory `HashMap` store supporting `GET`, `PUT`, `DELETE`.
4. Write your first `docs/journal/` entry and your first unit test.

**You build:** all of it (it's small and it's your Rust warm-up).
**Agent may scaffold:** nothing yet — keep this hands-on.

**Exit demo:** `cargo run`, then interactively `PUT foo bar`, `GET foo` → `bar`, `DELETE foo`, `GET foo` → not found.

---

## Phase 1 — Pager / on-disk storage  *(Medium)*

**Goal:** Data survives a restart. You now own bytes on disk.

**Steps:**
1. Decide a fixed page size (e.g., 4 KB). Write a `Pager` that reads/writes page N to/from a file at the right offset.
2. Design a **page layout** on paper first (header + slots/records). Write it down in `docs/decisions/`.
3. Implement record serialization/deserialization **by hand** (no `serde` yet) — you want to feel byte layout and endianness.
4. Add page allocation + a free list for reuse.
5. Make the in-memory store from Phase 0 persist through the pager.

**You build:** pager, page layout, serialization, free list.
**Agent may scaffold:** file-open boilerplate, test fixtures.

**Exit demo:** `PUT` some keys, kill the process, restart, `GET` them back.

---

## Phase 2 — B-tree index  *(Hard — the core)*

**Goal:** Ordered, persistent storage. This is the heart of the project; expect it to expand, and let it.

**Steps:**
1. On paper, work out node layout (internal vs leaf), order/fanout, and the split rule. Record it in `docs/decisions/`.
2. Implement search (walk root → leaf).
3. Implement insert **without** splits (assume space) — get the happy path working.
4. Implement **node splitting** on overflow, including splitting the root (the tricky one).
5. Implement a **cursor** for ordered range scans.
6. *(Stretch within the phase)* deletion with merge/rebalance — notoriously fiddly; treat it as its own mini-project.

**You build:** the entire B-tree. This is the phase that most repays doing yourself.
**Agent may scaffold:** nothing core. It may help you design *test cases* that stress splits.

**Exit demo:** insert keys in random order; a range scan returns them sorted; the tree survives restart.

> **Heads-up for later:** the choices you make here about page references and node ownership will shape Phases 3 and 5. Keep node access mediated through the pager rather than holding long-lived references into pages — your future concurrent self will thank you. The borrow checker will likely push you this way anyway; listen to it.

---

## Phase 3 — WAL & crash recovery  *(Hard)*

**Goal:** It survives being killed mid-write. Durability and atomicity at the single-operation level.

**Steps:**
1. Understand the invariant: **log the change before you change the page** (write-ahead).
2. Implement an append-only WAL: serialize each change as a record, fsync appropriately.
3. On startup, **replay** the WAL to bring pages to a consistent state.
4. Implement a checkpoint/truncation strategy so the log doesn't grow forever.
5. Test recovery by killing the process at deliberately awkward moments.

**You build:** WAL format, write path ordering, replay logic.
**Agent may scaffold:** a test harness that simulates crashes.

**Exit demo:** `PUT`, kill -9 mid-operation, restart → the database is in a consistent state (either the write fully applied or fully absent).

### → DONE I: a crash-safe database... almost. Reach it fully after Phase 4.

---

## Phase 4 — Relational layer: schema + tiny SQL  *(Medium–Hard)*

**Goal:** Drive the engine with SQL. This is the demo.

**Steps:**
1. Define a **catalog**: tables, columns, types. Persist it (it's just data in your own store).
2. Write a **tokenizer**, then a **parser** for a tiny SQL subset: `CREATE TABLE`, `INSERT`, `SELECT ... WHERE`. Hand-rolled — this is parsing fundamentals, not a crate job.
3. Map a row to a B-tree key/value (e.g., primary key → serialized row).
4. Build a small **execution** path: a `SELECT` becomes a B-tree scan + filter; `INSERT` becomes a B-tree insert.

**You build:** catalog, tokenizer, parser, executor.
**Agent may scaffold:** nothing core. Parsing is a classic learning target.

**Exit demo:** `CREATE TABLE users (id INT, name TEXT);` then `INSERT`, then `SELECT * FROM users WHERE id = 3;` returns the row — and survives a restart.

### ✅ DONE I — "I built a database." A crash-safe database you drive with SQL.

---

## Phase 5 — Transactions & concurrency  *(Hard)*

**Goal:** Real ACID, and the phase where Rust's concurrency guarantees genuinely earn their keep.

**Steps:**
1. Add `BEGIN` / `COMMIT` / `ROLLBACK`; group operations into atomic units (extend the WAL with transaction boundaries).
2. Study isolation levels; pick one to implement first (e.g., snapshot isolation or read-committed).
3. Add concurrency control: start with coarse locking/latching, understand its cost, then refine.
4. *(Stretch)* a taste of **MVCC** — versioned values so readers don't block writers.
5. Let Rust enforce thread-safety: `Send`/`Sync`, `Arc`, `Mutex`/`RwLock`. Fight the compiler here; it's teaching you data-race freedom.

**You build:** transaction manager, locking/concurrency control.
**Agent may scaffold:** concurrent stress-test harness.

**Exit demo:** concurrent clients hammer the DB; a transaction that errors mid-way leaves no partial state; no data races (and Rust proved it at compile time).

### ✅ DONE II — "I understand the hard parts."

---

## Phase 6 — Query processing  *(Hard)*

**Goal:** Make it production-*shaped*: the engine chooses *how* to answer a query.

**Steps:**
1. Introduce a **logical plan** → **physical plan** separation.
2. Add **secondary indexes** (more B-trees) and teach the planner to use them.
3. Implement **predicate pushdown** and a simple **cost-based** choice (index scan vs. full scan).
4. Implement at least one **JOIN** algorithm (nested-loop first; hash join as a stretch).

**You build:** planner, index selection, join execution.
**Agent may scaffold:** query test fixtures.

**Exit demo:** a query with a `WHERE` on an indexed column uses the index (prove it via an `EXPLAIN`-style output you build); a two-table join returns correct rows.

### ✅ DONE III — "It's production-shaped."

---

## Phase 7 — Capstone: distribution  *(Hard)*

**Goal:** Go beyond a single node. **Headline direction: replication + consensus via Raft.**

You will pick the capstone direction when you arrive here. Raft is the recommended headline because it builds naturally on a complete single-node engine and lands squarely in your distributed-systems wheelhouse. The alternatives below are **not** sub-tasks of this project — they're your *next* Rust projects:

- **LSM-tree storage engine** — build an alternate storage core and benchmark it against your B-tree.
- **HNSW vector search** — an AI-adjacent similarity-search index; your bridge to the ML-infra space.

**Steps (Raft path, sketch — to be detailed on arrival):**
1. Implement leader election.
2. Implement log replication.
3. Wire your WAL/transaction layer to the replicated log.
4. Handle failure/recovery scenarios.

**Exit demo:** a small cluster; kill the leader; a new leader is elected and committed data is preserved.

### ✅ DONE IV — "Frontier systems work."

---

## Progress tracking

- Keep a running checklist of phase steps wherever you like (issues, a `PROGRESS.md`, or your tool of choice).
- **End every working session** by either closing a step or writing a `docs/journal/` entry about where you got stuck. The journal is the real progress log.
- When you make a non-obvious design choice, drop a `docs/decisions/` note *before* you move on — your future self debugging Phase 5 will need to know why Phase 2 looks the way it does.

## Book anchoring (see LEARNING_GUIDE.md for chapter-level detail)

- **Database Internals (Petrov)** — Phases 1–3, 5 (storage, B-trees, recovery, transactions) and Part II for Phase 7.
- **Designing Data-Intensive Applications (Kleppmann)** — Phases 4–7 (data models, transactions/isolation, replication, consensus).
