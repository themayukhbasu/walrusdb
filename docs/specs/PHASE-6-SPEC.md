# Phase 6 Spec — Query Processing

**Goal:** Make it production-*shaped*: the engine chooses *how* to answer a query. The end of it is **DONE III: "It's production-shaped."**

**Reading anchor:** DDIA ch. 3 (indexes; the OLTP/analytics distinction). DI for index mechanics. Any introductory query-optimizer material as a supplement.

---

## What is and isn't in scope

### In scope
- A **logical plan → physical plan** separation, replacing Phase 4's "one statement, one strategy" executor.
- **Secondary indexes**: `CREATE INDEX` creating more B-trees, maintained on every write.
- **Predicate pushdown** and a simple **cost-based choice**: index scan vs. full scan.
- An **`EXPLAIN`-style output** that shows which plan was chosen — this is how the exit demo *proves* the planner works.
- **Nested-loop join** for two tables; **hash join** as a stretch.
- The SQL subset grows just enough to exercise this: `CREATE INDEX`, `SELECT` with a two-table `FROM`/join condition, and (now cheap to add) `AND` in `WHERE`.

### Explicitly out of scope
- **Real cost models** — no statistics histograms, no selectivity estimation beyond the crudest counts. Your cost model can be embarrassingly simple; the lesson is that *a* principled choice happens, not that it's well-calibrated.
- **Join ordering** — with exactly two tables there's little to reorder. Left table as the outer loop is fine (or make the swap decision part of your cost model if it's easy).
- **Sort-merge join, `ORDER BY`, `GROUP BY`, aggregates, subqueries.** Next projects, not this one.
- **Query result caching, prepared statements, parallel execution.**

---

## Components to build

### 1. Logical plan (`src/plan.rs`)

The parser's AST says what the user *wrote*; the logical plan says what must *happen*, still without saying how: a small tree of operators — scan(table), filter(predicate), join(left, right, condition), project.

**Design questions:**
- Represent plan nodes as an **enum** or as **trait objects**? (LEARNING_GUIDE flags both.) An enum is the natural first choice given your AST experience — closed set, exhaustive `match`. Understand what trait objects would buy (open extension) and cost (dynamic dispatch, object-safety rules) before deciding; the tradeoff *is* the Rust lesson of this phase.
- Where does the AST → logical plan translation live, and what does it validate (tables exist, columns exist — this is where the catalog gets consulted, once, up front)?

### 2. Secondary indexes

An index on `users(name)` is just another B-tree — you built the hard part in Phase 2. The new problems are around the edges:

**Design questions:**
- **What are the index entries?** The natural form: key = indexed-column value, value = primary key (then fetch the row from the main tree — the classic two-step). But your Phase 2 tree may assume unique keys — two users named `alice` breaks that. The standard fix is making the index key the *composite* (column value, primary key). Work through what that does to your key-encoding and to range-scanning "all alices."
- **Write-path maintenance:** every `INSERT`/`DELETE` must now update the main tree *and every index*, inside the same transaction — a partial write is index corruption. What in your Phase 5 machinery guarantees this atomicity? (Nothing new to build if Phase 5 is right — but *prove* it with a test that kills between the two writes.)
- Where are index definitions stored? (You know this one: the catalog. It's just data.)
- What does `CREATE INDEX` on a non-empty table do? (Scan and backfill — your first long-running operation.)

### 3. Physical plan & the planner (`src/planner.rs`)

The planner maps a logical plan to a physical one, making actual choices:

- **Index selection:** `WHERE name = 'alice'` with an index on `name` → index scan; without → full scan. What does the planner need to know to make that call, and where does it look (catalog)?
- **Cost-based choice:** even `index available → use it` is a *rule*, not a *cost*. To make it cost-based you need at least a row count per table — which means maintaining one (where? catalog again) — and a cost formula, even a crude one (full scan ∝ row count; index lookup ∝ tree height + matches). **Design question:** construct a case where the index is available but the full scan wins (the predicate matches nearly every row). If your planner can't ever prefer the full scan, it's rule-based — fine as a first cut, but know which one you built.
- **Predicate pushdown:** filter *during* the scan rather than materializing everything and filtering after. With the volcano model (component 4) this becomes concrete: where does the filter operator sit in the tree, and what work does moving it save?

### 4. Execution — the volcano/iterator model (`src/exec.rs`)

Each physical operator implements a common `next() -> Option<Row>` interface; plans execute by pulling from the root. A filter pulls from its child and skips non-matching rows; a scan pulls from a B-tree cursor.

**Design questions:**
- This is Rust's `Iterator` trait's home turf — can your operators literally *be* `Iterator` implementations, chained with adapters? Try it; notice where it shines (filter, project are nearly free) and where it fights you (the join holding two children; state that resets). Where it fights you, a hand-rolled `next()` on your plan enum is honest and fine.
- Your Phase 2 cursor is the leaf of every plan. Does its API (seek, advance) fit the pull model cleanly, or does it need an adapter?

### 5. Joins

**Nested-loop join** (required): for each row of the outer, scan the inner for matches. Correct, quadratic, and the baseline everything else is measured against.

**Design questions:**
- If the join condition hits an *indexed* column of the inner table, the inner scan becomes an index lookup — nested-loop's respectable cousin. Does your planner notice?
- **Hash join** (stretch): build a hash table over the smaller side, probe with the other. Measure both on a few thousand rows and *feel* the asymptotic difference — then check whether your cost model would have predicted the winner.

### 6. `EXPLAIN`

`EXPLAIN SELECT ...` prints the chosen physical plan as an indented tree instead of executing. Cheap to build (you have the plan tree; walk and print it) and it's both your debugging tool and the exit demo's proof.

---

## Decisions to document before coding

1. **Plan representation** — enum vs. trait objects, and why.
2. **Index entry format** — the composite-key answer to duplicates, and its encoding.
3. **Cost model** — the formula, however crude, and the stats it needs; rule-based vs. cost-based, honestly labeled.
4. **Executor model** — `Iterator`-native vs. hand-rolled `next()`, and where each fit or fought.

---

## Exit demo

```
walrusdb> CREATE INDEX idx_name ON users (name);
OK
walrusdb> EXPLAIN SELECT * FROM users WHERE name = 'alice';
IndexScan(idx_name, name = 'alice')
walrusdb> EXPLAIN SELECT * FROM users WHERE age = 30;
Filter(age = 30)
  FullScan(users)
walrusdb> SELECT * FROM users, orders WHERE users.id = orders.user_id;
...correct joined rows...
```

An indexed `WHERE` provably uses the index; a two-table join returns correct rows.

### ✅ DONE III — "It's production-shaped."

---

## What you should be able to explain when this phase is complete

- Why a query optimizer exists — what concretely goes wrong without one (have the `EXPLAIN` output from both plans of the same query ready).
- When an index scan beats a full scan and when it doesn't, and how your cost model decides — including the case where yours prefers the full scan.
- How the composite-key trick makes non-unique indexes work on a unique-key B-tree.
- How a nested-loop join works, why a hash join beats it, and what your measurements showed.
- Logical vs. physical plans: what question each answers, and what lives in the gap between them.
