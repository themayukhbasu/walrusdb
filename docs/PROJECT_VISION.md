# WalRusDB — Project Vision

> **WalRusDB** = **WAL** (write-ahead log) + **Rus**(t). A database engine built by hand, the hard way, to actually understand it. Mascot: a walrus, naturally.

## The one-line version

Build **WalRusDB** — a relational database engine, on top of a key-value storage core, in Rust — as a vehicle for deep systems mastery and for learning Rust properly, the hard way.

## Why this project exists

The database is the *vehicle*; Rust's ownership model is the *teacher*. A database is, at its heart, a long series of hard questions about who owns a chunk of memory and for how long — who owns a page in the buffer pool, when it's safe to evict, how long a slice into that page stays valid, how two threads touch the same page without corrupting it. Rust's borrow checker is essentially that problem turned into compiler rules, so the language and the domain reinforce each other.

Just as importantly: the *concepts* — storage layout, indexing, buffer management, concurrency control, write-ahead logging, recovery, query planning — are language-agnostic and are exactly what distinguishes a senior systems/data engineer. Even the parts of this project that never become a "Rust job" pay for themselves in any backend, data, or platform interview.

This is explicitly **not** a project optimized for speed of completion or for résumé volume. It is optimized for the depth of understanding you walk away with.

## What "learning" means here (the non-negotiable)

You write the code. Agents and tools assist, but **the learning-critical logic — the data structures, the algorithms, the systems mechanics — is yours to write.** An agent that hands you a finished B-tree has stolen the entire point of the exercise. The agents in this repo are configured as Socratic tutors, not autocomplete. See `AGENTS.md` and `CLAUDE.md`.

The test of success is not "does it run" but **"can I explain why it's built this way?"**

## What we're building

A single-process, embedded database engine — SQLite-flavored — built in clearly separated layers:

- A **storage engine** at the core: fixed-size pages on disk, a B-tree index, a write-ahead log.
- A **relational layer** on top: a schema/catalog, a small SQL subset, and query execution.
- **ACID transactions and concurrency** woven through.
- A **query processor**: planning, secondary indexes, joins.
- A **distributed capstone**.

It will be slow, minimal, and gloriously un-production-ready. That is correct.

## Guiding principles

1. **Learning over shipping.** Every shortcut that skips understanding is a bug.
2. **You write the core.** Plumbing and scaffolding can be delegated; the database is not.
3. **Explanation always.** No code or fix appears anywhere without the reasoning behind it.
4. **Document as you go.** Problems, dead-ends, and design choices get written down (see below) — by you, because writing is where the learning sticks.
5. **Standard library first for core components.** Hand-roll the B-tree, the page format, the WAL, the parser. Reach for external crates only for things that are *not* the thing you're trying to learn.
6. **No time pressure.** Depth over speed. Rust's curve makes this a long, deliberate haul, and that's a feature.

## What success looks like — multiple "done" objectives

Nothing in this project is "optional." Instead, there are several legitimate places to plant a flag, each marking a real level of mastery. Reaching any of them is a genuine win.

- **Done I — "I built a database."** A crash-safe engine you drive with SQL. *(end of Phase 4)*
- **Done II — "I understand the hard parts."** A genuinely ACID, concurrent engine. *(end of Phase 5)*
- **Done III — "It's production-shaped."** A single-node engine with a real query processor — planning, secondary indexes, joins. *(end of Phase 6)*
- **Done IV — "Frontier systems work."** A distributed engine. *(end of Phase 7)*

Calibrate your own ambition against these. You can stop at any one with pride and a deep, defensible understanding.

## Non-goals

- Not production-grade, not fast, not feature-complete, not a SQLite competitor.
- Not optimizing for time-to-finish.
- Not assembling off-the-shelf storage/parser/consensus crates for the core — that would teach you an API, not a database.

## The learning journal

A `docs/` directory is part of the deliverable, not an afterthought:

- `docs/journal/` — dated entries as problems are faced: **problem → what I tried → resolution → what I learned.**
- `docs/decisions/` — short design-decision records (mini-ADRs): the choice, the alternatives, why.

Agents prompt you to write these and may scaffold an empty entry, but the content is yours. Six months from now this folder is the proof — to a future employer and to yourself — that you understood every hard part.

## A note to future-me

You picked the slower language and the harder framing on purpose. The first phases will feel disproportionately difficult; that's the borrow checker teaching you, not you failing. Keep a journal entry every time you get stuck. Trust the curve.
