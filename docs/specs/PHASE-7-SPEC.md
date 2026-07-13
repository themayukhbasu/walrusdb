# Phase 7 Spec — Capstone: Distribution (Raft)

> **This is a sketch, not a working spec — deliberately.** `PLAN.md` commits to detailing the capstone *on arrival*, and that's right: the design decisions here depend on what your single-node engine actually looks like after Phases 1–6, and you'll make better choices about them with six phases of judgment behind you. When you arrive, rewrite this file into a real spec (the shape of the Phase 2–6 specs) as the phase's first task — writing the spec *is* the design exercise.

**Goal:** Go beyond a single node: replication and consensus via Raft. The end is **DONE IV: "Frontier systems work."**

**Reading anchor (do this part before writing the real spec):**
- The Raft paper — "In Search of an Understandable Consensus Algorithm" (Ongaro & Ousterhout). Deliberately readable; read it fully, twice. The condensed Figure 2 is the entire protocol on one page — you will live in it.
- DDIA ch. 5 (replication) and ch. 9 (consistency & consensus).
- DI Part II (distributed systems).

---

## The capstone decision (make it on arrival)

Raft is the headline recommendation — it builds directly on the engine you'll have, and the punchline writes itself: **your Phase 3 WAL becomes the state machine's input, and Raft's replicated log is the WAL generalized across machines.** But confirm the choice when you get here. The alternatives from `PLAN.md` (LSM-tree engine, HNSW vector search) are *separate next projects*, not directions for this phase.

---

## Rough shape (to be detailed in the real spec)

1. **Leader election** — terms, votes, randomized election timeouts. Build and test this *alone*, with no log, before anything else.
2. **Log replication** — AppendEntries, the consistency check, commit indices.
3. **Wire the engine in** — committed log entries drive your WAL/transaction layer; here is where "replicated log ≈ generalized WAL" stops being a slogan and becomes your code.
4. **Failure and recovery** — kill leaders, partition followers, restart nodes; committed data must survive all of it.

## Known-unknowns the real spec must resolve

- **Async Rust**: this phase almost certainly means `tokio` — a deliberate exception to standard-library-first, since async runtimes are not the thing being learned. Budget real learning time for async/await, and note the exception in `docs/decisions/`.
- **What the state machine is**: does Raft replicate SQL statements, WAL records, or key-value operations? (This is *the* design decision of the phase — it determines what "apply" means and interacts with everything from Phase 5.)
- **Testing without real networks**: deterministic simulation vs. actual processes on localhost. Decide early; it shapes the whole build.
- **Scope fences**: snapshotting/log compaction, membership changes, and client-session semantics (exactly-once) are each phase-sized on their own. The real spec must draw explicit in/out lines around them — Raft §6–7 tells you what you're fencing out.

---

## Exit demo (from PLAN.md — the fixed point this spec must hit)

A small cluster; kill the leader; a new leader is elected and committed data is preserved.

### ✅ DONE IV — "Frontier systems work."

---

## What you should be able to explain when this phase is complete

- Why consensus is hard, and what Raft simplifies relative to Paxos.
- How leader election and log replication keep replicas consistent — traced through *your* code, not the paper's figures.
- How your replicated log relates to the single-node WAL from Phase 3.
- What guarantees your cluster can and cannot make, and precisely where linearizability holds or leaks.
