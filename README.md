# 🦭 WalRusDB

![Built with Rust](https://img.shields.io/badge/built%20with-Rust-000000?logo=rust)
![Status](https://img.shields.io/badge/status-learning%20project%20%F0%9F%9A%A7-orange)

**WalRusDB** = **WAL** (write-ahead log) + **Rus**(t).

A relational database engine built from scratch in Rust — on top of a hand-rolled key-value storage core — as a deliberate, ground-up exercise in understanding how databases actually work.

> ⚠️ **This is a learning project, not a production database.** It is built by hand, the slow way, on purpose. It will be minimal, unoptimized, and gloriously un-production-ready. That is the point.

---

## Why this exists

The database is the vehicle; Rust's ownership model is the teacher. A database is, at its core, a long series of hard questions about who owns a piece of memory and for how long — and Rust's borrow checker turns exactly those questions into compiler rules. The goal isn't to ship a database; it's to be able to **explain why every part is built the way it is.**

The full rationale lives in [`docs/PROJECT_VISION.md`](docs/PROJECT_VISION.md).

---

## Architecture

WalRusDB is built in layers, each one resting on the one below:

```
┌─────────────────────────────────────────┐
│  Query processing  (planner, joins)      │   Phase 6
├─────────────────────────────────────────┤
│  Transactions & concurrency  (ACID)      │   Phase 5
├─────────────────────────────────────────┤
│  Relational layer  (catalog, SQL)        │   Phase 4
├─────────────────────────────────────────┤
│  Durability  (write-ahead log, recovery) │   Phase 3
├─────────────────────────────────────────┤
│  B-tree index  (ordered storage)         │   Phase 2
├─────────────────────────────────────────┤
│  Pager  (pages on disk)                  │   Phase 1
└─────────────────────────────────────────┘
```

Everything in the core — the pager, the B-tree, the WAL, the SQL parser — is written by hand using only the standard library. No storage, parser, or consensus crates: assembling those would teach an API, not a database.

---

## Roadmap

Nothing is "optional." Instead there are several legitimate finish lines, each marking a real level of mastery. Full detail in [`docs/PLAN.md`](docs/PLAN.md).

- [ ] **Phase 0** — Setup & first win (in-memory KV REPL)
- [ ] **Phase 1** — Pager / on-disk storage
- [ ] **Phase 2** — B-tree index
- [ ] **Phase 3** — WAL & crash recovery
- [ ] **Phase 4** — Relational layer: schema + SQL &nbsp;→ &nbsp;🏁 **Done I: "I built a database."**
- [ ] **Phase 5** — Transactions & concurrency &nbsp;→ &nbsp;🏁 **Done II: "I understand the hard parts."**
- [ ] **Phase 6** — Query processing &nbsp;→ &nbsp;🏁 **Done III: "It's production-shaped."**
- [ ] **Phase 7** — Capstone: distribution (Raft) &nbsp;→ &nbsp;🏁 **Done IV: "Frontier systems work."**

**Current status:** Phase 0.

### What it's working toward

By Done I, the target interface looks like this:

```sql
CREATE TABLE users (id INT, name TEXT);
INSERT INTO users VALUES (3, 'walrus');
SELECT * FROM users WHERE id = 3;
```

...backed by a real B-tree on disk that survives a crash. (Not implemented yet — see the roadmap.)

---

## Getting started

> Under active development — early phases are still being built.

```bash
git clone https://github.com/themayukhbasu/walrusdb.git
cd walrusdb
cargo run
```

New to Rust? Start with [`docs/GETTING_STARTED.md`](docs/GETTING_STARTED.md), which covers the toolchain and just enough of the language to begin Phase 0.

---

## Repository layout

```
walrusdb/
├── AGENTS.md          # Learning-first contract for any AI agent
├── CLAUDE.md          # Claude-specific working instructions
├── README.md
├── Cargo.toml
├── src/
└── docs/
    ├── PROJECT_VISION.md   # The why, and what "done" means
    ├── PLAN.md             # The phased roadmap
    ├── LEARNING_GUIDE.md   # Concepts + book references per phase
    ├── GETTING_STARTED.md  # Rust on-ramp + guided Phase 0
    ├── journal/            # Dated notes: problem → tried → resolved → learned
    └── decisions/          # Short design-decision records
```

---

## A note on the AI-agent files

[`AGENTS.md`](AGENTS.md) and [`CLAUDE.md`](CLAUDE.md) configure any AI assistant used on this repo as a **Socratic tutor, not a code generator** — the learning-critical code is written by hand, and agents guide with questions and explanations rather than handing over solutions. If you're here to learn from the code, those files explain the house rules.

---

## Documentation

- [Project Vision](docs/PROJECT_VISION.md) — why this exists and what success looks like
- [Plan](docs/PLAN.md) — the phased roadmap
- [Learning Guide](docs/LEARNING_GUIDE.md) — what to study per phase
- [Getting Started](docs/GETTING_STARTED.md) — Rust on-ramp

---

## Built with

- **Rust** (standard library only, for the core)
- A lot of paper diagrams and a healthy respect for the borrow checker

## Reading list

This project follows two books closely:

- *Database Internals* — Alex Petrov (storage, B-trees, recovery, transactions, distribution)
- *Designing Data-Intensive Applications* — Martin Kleppmann (data models, transactions, replication, consensus)

## License

_TBD_ — MIT is a sensible default for a learning project. Add a `LICENSE` file when you decide.
