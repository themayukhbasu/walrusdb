# Phase 4 Spec — Relational Layer: Schema + Tiny SQL

**Goal:** Drive the engine with SQL. This is the demo — and the end of it is **DONE I: "I built a database."**

**Reading anchor:** DDIA ch. 2 (data models & query languages). For parsing technique, *Crafting Interpreters* (Nystrom), the scanning and parsing chapters — friendliest treatment anywhere, even though it's not a database book.

---

## What is and isn't in scope

### In scope
- A **catalog**: tables, columns, types (INT and TEXT are enough) — persisted *in your own store*.
- A hand-rolled **tokenizer** and **recursive-descent parser** for exactly this subset:
  - `CREATE TABLE t (col TYPE, ...);`
  - `INSERT INTO t VALUES (...);`
  - `SELECT * FROM t;` and `SELECT * FROM t WHERE col = literal;`
  - `DELETE FROM t WHERE col = literal;`
- **Row ↔ key/value mapping**: a row becomes a B-tree entry.
- A small **executor** that turns each statement into storage-engine calls.

### Explicitly out of scope (do not build yet)
- **`UPDATE`.** Stretch if the phase goes fast; not required (Phase 5's rollback needs delete, which you have).
- **Joins, secondary indexes, planning, `EXPLAIN`** — all Phase 6. There is exactly one way to execute each query here; the *choice* of how comes later.
- **Expressions**: no `AND`/`OR`, no `<`/`>`, no arithmetic, no functions. `WHERE col = literal` only. (Adding `AND` later is easy; parsing precedence climbing now is a detour.)
- **NULLs, column selection (`SELECT a, b`), constraints, `DROP`.** Decide-and-skip; note it.
- **Transactions.** Each statement auto-commits through the Phase 3 machinery.

---

## Components to build

### 1. Catalog (`src/catalog.rs`)

The catalog answers "what tables exist, and what are their columns/types?" — and it's *just data in your own engine*.

**Design questions:**
- **The bootstrap problem** (the fun one): if table schemas live in a catalog table, how do you read the catalog table's schema? Something must be hard-coded — a reserved key prefix, a well-known table name, or a fixed serialization the engine knows innately. Every real database has this wired-in kernel (SQLite's `sqlite_master`, Postgres's `pg_class`). Decide what yours is and document it.
- How do you serialize a table definition into a value? (You have Phase 1 record-encoding reps; this is the same skill.)
- How do table names avoid colliding with user data in a single key space? A key prefix/namespace convention is the classic answer — this decision also shapes component 4.

### 2. Tokenizer (`src/lexer.rs` or `token.rs`)

SQL text → a `Vec<Token>`. Model tokens as an enum: keywords, identifiers, integer literals, string literals, punctuation.

**Design questions:**
- Keywords vs. identifiers: `SELECT` is a keyword, `users` is an identifier — where in the tokenizer does that distinction happen, and is it case-sensitive? (Real SQL keywords aren't; decide.)
- String literal delimiters and what characters you allow inside.
- What does the tokenizer do with input it doesn't recognize — panic, or return your error type? (You know the answer; make the error message say *where*.)

### 3. Parser (`src/parser.rs`)

Tokens → an AST. Model the AST as enums — this is `match` and Rust ADTs at full power, and coming from Scala this is the part of the phase that will feel like home.

**Design questions:**
- Write the grammar down first, in something BNF-ish, in a `docs/decisions/` note. Four statements, each a straightforward production — recursive descent means roughly one function per production.
- What does a parse error look like? Aim for "expected X, found Y at token N" — you'll thank yourself.
- Parser output is a *statement*, not an action: `Statement::Select { table, filter }`, not a function call. The gap between those two is the executor's job — keep the layers separate even though the subset is tiny; Phase 6 will widen exactly this gap into a planner.

### 4. Row encoding (`src/row.rs` or inside the executor)

A row must become a (key, value) pair for the B-tree.

**Design questions:**
- **What's the key?** First column as primary key is the simple rule — fine, but state it. What happens on a duplicate key insert — error or overwrite? Decide and document.
- Recall your Phase 2 key-comparison decision: keys sort byte-wise. If the primary key is an INT, does your integer encoding sort correctly byte-wise? (This is the endianness lesson from Phase 1 coming back with teeth — worth a journal entry when it bites.)
- **Value = the serialized row.** You need a format that can encode INT and TEXT columns in sequence and decode them *knowing the schema from the catalog*. Does the row itself carry any layout info, or is the catalog's schema the sole decoder ring? Both work; different failure modes when schemas evolve. Decide.
- How do keys from different tables share one B-tree without colliding? (Same namespace question as the catalog — one consistent answer covers both. Alternative: multiple B-trees, one per table, if your Phase 2 root/meta design supports it. Either is legitimate; document the choice.)

### 5. Executor (`src/executor.rs`)

Takes an AST statement, returns rows or a count. No planning — each statement has exactly one strategy:

- `CREATE TABLE` → validate, write to catalog.
- `INSERT` → validate against schema (column count, types), encode, B-tree insert.
- `SELECT` → full scan via your Phase 2 cursor, decode each row, apply the filter, collect.
- `DELETE` → scan, collect matching *keys*, then delete them.

**Design question worth sitting with:** why "scan, collect keys, *then* delete" rather than deleting while the cursor is walking? What could deleting-under-a-live-cursor corrupt? (Your cursor design from Phase 2 makes this concrete — and notice the borrow checker has an opinion about the two-borrows shape too. Same reason, again.)

### 6. REPL wiring — thin

The REPL's parse-command-word dispatch gets replaced by: read a line → tokenize → parse → execute → print rows or an error. Keep `PUT`/`GET`/`DELETE` around as a debug backdoor if you like.

---

## Decisions to document before coding

1. **Catalog bootstrap** — the hard-coded kernel and the catalog's own encoding.
2. **Key namespace** — one shared B-tree with prefixes, or one tree per table.
3. **Grammar** — the BNF-ish sketch of the four statements.
4. **Primary key rule** — which column, duplicate behavior, and the sortable-integer-encoding answer.
5. **Row format** — self-describing vs. catalog-as-decoder-ring.
6. **Skipped-on-purpose list** — NULLs, UPDATE, column selection, etc., so future-you knows they're absent by choice.

---

## Exit demo

```
$ cargo run walrus.db
walrusdb> CREATE TABLE users (id INT, name TEXT);
OK
walrusdb> INSERT INTO users VALUES (1, 'alice');
OK
walrusdb> INSERT INTO users VALUES (3, 'carol');
OK
walrusdb> SELECT * FROM users WHERE id = 3;
3 | carol
walrusdb> DELETE FROM users WHERE id = 3;
OK
walrusdb> ^C

$ cargo run walrus.db
walrusdb> SELECT * FROM users;
1 | alice
```

SQL in, rows out, survives a restart — including the schema.

### ✅ DONE I — "I built a database."

---

## What you should be able to explain when this phase is complete

- The full pipeline: SQL text → tokens → AST → execution — and what each layer's *only* job is.
- How `SELECT ... WHERE` maps onto operations your storage engine already provides, and why nothing new was needed down there.
- Why the catalog is "just data" in your own engine, and how the bootstrap problem is solved.
- Why your integer keys sort correctly as bytes (or the war story of when they didn't).
- Why the executor collects keys before deleting them.
