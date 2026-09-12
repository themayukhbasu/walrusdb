# Phase 5 Spec — Transactions & Concurrency

**Goal:** Real ACID, and the phase where Rust's concurrency guarantees genuinely earn their keep. The end of it is **DONE II: "I understand the hard parts."**

**Reading anchor:** DDIA ch. 7 (transactions) — the canonical treatment of isolation levels; **read it twice**, it's that dense with interview material. DI ch. 5 (concurrency control).

---

## What is and isn't in scope

### In scope
- `BEGIN` / `COMMIT` / `ROLLBACK` grouping multiple statements into one atomic unit.
- **Transaction boundaries in the WAL** — begin/commit records; recovery that respects them.
- **Rollback** — undoing a live transaction's changes on abort.
- **Isolation: read-committed via locking** first. (Not snapshot isolation — SI essentially *is* MVCC, which is this phase's stretch, not its start.)
- **Real concurrency**: multiple clients, threads, `Arc`/`Mutex`/`RwLock`, `Send`/`Sync` — coarse first, then refined.
- Deadlock handling (at least detection-by-timeout).
- *(Stretch)* **MVCC** — versioned values so readers don't block writers — and with it, snapshot isolation.

### Explicitly out of scope
- **Serializable isolation** (2PL held to strict discipline, or SSI). Understand it from DDIA; don't build it.
- **MVCC garbage collection** — if you do the stretch, old versions may simply accumulate.
- **Savepoints, nested transactions, distributed transactions.**
- Lock-manager sophistication: no lock escalation, no fairness tuning. Correct and comprehensible beats clever.

---

## Sequencing note (do the single-threaded half first)

This phase is really two phases wearing one name: **(A)** transaction semantics — atomicity via WAL boundaries and rollback, still single-threaded; **(B)** concurrency — locks, threads, isolation. Finish and crash-test A before starting B. A rollback bug under concurrency is misery to debug; the same bug single-threaded is an afternoon.

---

## Components to build

### Part A — Transactions (single-threaded)

#### 1. Transaction boundaries in the WAL

Extend your Phase 3 record enum with begin/commit (and possibly abort) records carrying a transaction ID.

**Design questions:**
- Recovery must ensure a transaction that never committed leaves **no trace**. Your Phase 3 design determines how hard this is — did you flush uncommitted changes to the data file (steal), or can you guarantee you never do (no-steal)? No-steal means replay simply skips records of uncommitted transactions; steal means you must *undo* them, which needs before-images. Go back to your Phase 3 `docs/decisions/` note on steal/no-steal — this is exactly where that decision cashes out. Update the note with what you now know.
- What does replay do with a transaction whose begin record exists but whose commit record doesn't (crash mid-transaction)?

#### 2. Rollback

`ROLLBACK` undoes the current transaction's changes while the database is *running* (this is separate from recovery-time undo, though they rhyme).

**Design questions:**
- What do you need to remember, per transaction, to undo it? An in-memory undo list of inverse operations (`INSERT k` ⇒ undo is `DELETE k`; `DELETE k` ⇒ undo is re-insert — *with the old value*, which means you must have saved it at delete time). Or read the undo information back out of the WAL. Both are legitimate; the WAL route depends heavily on your physical/logical choice from Phase 3.
- In what *order* must undo operations be applied, and why?
- This is why Phase 2's delete path and Phase 4's `DELETE` were non-negotiable — rollback of an insert *is* a delete.

#### 3. Transaction manager (`src/txn.rs`)

Owns transaction IDs, the set of active transactions, and each transaction's undo information. The REPL grows `BEGIN`/`COMMIT`/`ROLLBACK`; statements outside an explicit transaction auto-commit (wrap themselves in one).

**Design question:** what happens to a transaction left open when the client disconnects or the process is killed? (Tie the answer back to component 1's recovery behavior.)

### Part B — Concurrency

#### 4. Locking / isolation (read-committed)

**Design questions:**
- **Granularity:** one global database lock? Table locks? Key locks? Start coarse — a single `RwLock` over the whole engine is a legitimate first cut whose *cost* you should measure and feel — then refine one level. Each refinement is a decision note.
- What does read-committed require of your locks, precisely? (Write locks held to commit; read locks released early — check DDIA's definition against your implementation, then verify with a test that demonstrates the anomaly read-committed *permits*: non-repeatable read.)
- **Deadlock:** two transactions, two keys, opposite order — now what? Timeout-and-abort is the honest simple answer; a waits-for graph is the stretch. Either way you now have transactions that fail *through no fault of their own* — which means `ROLLBACK` (component 2) gets exercised by the system itself, not just the user.

#### 5. Making the engine thread-safe (the Rust payoff)

Multiple client threads (or connections) sharing the engine. This is where the compiler becomes your co-author: wrap shared state, and let `Send`/`Sync` errors *tell you* where your design has aliased mutation.

**Design questions:**
- What actually gets shared — `Arc<Mutex<Engine>>` wholesale, or finer structure (buffer pool, lock table, WAL each with their own synchronization)? Start wholesale; refine with evidence.
- Your Phase 3 buffer-pool API decision resurfaces *hard* here: whatever `get_page` returns must be safe when two threads hold one. If Phase 3's design survives contact, that decision note was money in the bank; if not, the redesign is the journal entry of the phase.
- Keep a running list: every compiler rejection in this phase is a would-be data race. Write down what each was protecting you from. That list is the "why Rust" story for every interview, in your own code.
- **Note from Phase 1:** the "is the db file still there" check was parked as an inline, sampled, single-threaded check (every Nth operation) specifically because a real watcher needs a background thread sharing state with the main thread — `Arc`/`Mutex`/atomics — which was ruled out of Phase 1 and 2 as concurrency creep. Now that those primitives are in scope, promote it: a real background thread doing the fstat-vs-stat inode check on a timer, signaling the main engine through whatever shared-state mechanism you land on for component 5. Small, but it's a genuine instance of "operational thread needs the same `Send`/`Sync` discipline as a client thread" — worth doing deliberately rather than skipping because it feels minor.

#### 6. Stretch — MVCC → snapshot isolation

Versioned values: writers create new versions; readers read the version visible at their transaction's start. Readers stop blocking writers.

If you attempt it, spec it separately when you get there (it deserves its own decision notes: version storage layout, visibility rule, write-write conflict handling). Do not start it until read-committed is solid and stress-tested.

---

## Decisions to document before coding

1. **Recovery with transactions** — the steal/no-steal cash-out; what replay does with uncommitted work.
2. **Undo mechanism** — in-memory undo list vs. WAL-derived; ordering.
3. **Lock granularity** — the starting point and each refinement, with the measured reason.
4. **Deadlock strategy** — timeout vs. detection; who gets aborted.
5. **Shared-state architecture** — what's inside which lock, and how the buffer pool API changed (if it did).

---

## Testing (agent may scaffold the *harness*; scenarios and invariants are yours)

- Single-threaded: `BEGIN`, several statements, `ROLLBACK` → no trace. `BEGIN`, statements, kill -9 before `COMMIT`, restart → no trace. Kill -9 *immediately after* commit record fsync → fully present.
- Concurrent: N threads hammering disjoint keys (no interference), then overlapping keys (locks earn their keep); a deliberate deadlock, resolved.
- A demonstration of the anomaly your isolation level permits (non-repeatable read under read-committed) — proving you know the boundary, not just the guarantee.

---

## Exit demo

Concurrent clients hammer the DB; a transaction that errors midway leaves no partial state; a `kill -9` mid-transaction leaves no partial state; and the compiler proved data-race freedom at compile time.

### ✅ DONE II — "I understand the hard parts."

---

## What you should be able to explain when this phase is complete

- ACID precisely — including the difference between atomicity (transactions, this phase) and durability (WAL, Phase 3).
- What each isolation level permits and forbids, each with a concrete anomaly example — and a demo of the one your level permits, from your own tests.
- How your recovery treats uncommitted transactions, and how that traces back to Phase 3's steal/no-steal position.
- How a deadlock forms in your engine and how it's resolved.
- How `Send`/`Sync` turned a class of concurrency bugs into compile errors — with a specific rejected-code example from your list.
