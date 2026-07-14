# Phase 2 Spec — B-tree Index

**Goal:** Ordered, persistent storage. This is the heart of the project — the phase that most repays doing yourself. Expect it to expand, and let it.

**Reading anchor:** DI ch. 2 (B-tree basics) and ch. 4 (implementing B-trees). Read them *slowly, before* designing your node layout — these two chapters are the core of the whole project. DI ch. 6 (variants) if you want depth.

---

## What is and isn't in scope

### In scope
- A B-tree built on top of Phase 1 pages: node layout for internal and leaf nodes.
- Search: walk root → leaf.
- Insert without splits (the happy path first).
- Node splitting on overflow, **including splitting the root**.
- A cursor for ordered range scans.
- A minimal delete path (find the key in its leaf, remove it). **Required, not stretch** — Phase 5's `ROLLBACK` of an `INSERT` is a delete.
- Replacing the Phase 1 linear-scan store with the B-tree behind the same `put`/`get`/`delete` interface.

### Explicitly out of scope (do not build yet)
- **Buffer pool.** Keep going straight through the pager; the cache arrives in Phase 3.
- **Merge/rebalance on underflow.** Deleting a key is required; keeping the tree perfectly balanced after deletes is a *stretch* mini-project. A tree that gets a bit sparse is acceptable.
- **Concurrency.** Single-threaded throughout.
- **Secondary indexes** (Phase 6) and B-tree variants (prefix compression, bulk loading, etc.).

---

## Components to build

### 1. Node layout (`src/node.rs` or an extension of `page.rs`)

A node *is* a page. You need two kinds:

- **Leaf node** — holds keys and values.
- **Internal node** — holds separator keys and child page IDs. For `N` keys it needs `N + 1` children; think through where that extra child pointer lives in your layout.

**Design questions to answer first (on paper, then in `docs/decisions/`):**
- How does a reader know whether a page is a leaf or an internal node? (A type byte in the page header is the classic answer — does your Phase 1 header have room?)
- **B-tree or B+-tree?** In a B+-tree, values live only in leaves and internal nodes are pure routing. DI ch. 2 explains why real databases almost universally choose B+. Decide consciously and write down why.
- Do leaves carry **sibling pointers** (next-leaf page ID)? Think ahead to how your cursor will move from one leaf to the next before answering.
- What is your **overflow rule** — is a node "full" by key count or by bytes used? Variable-length keys make this less obvious than it looks.

### 2. Root discovery & the meta page

**Design question:** after a restart, how does the tree find its root? And the harder version: when the root splits, the root *changes* — either the root's page number must be recorded somewhere durable, or the root must be pinned to a fixed page number and its *contents* moved on split. Both work; they have different split-path consequences. Decide and document.

(You already faced the sibling of this question in Phase 1: "how does the Pager know how many pages the file has?" Same family — durable metadata needs a durable home.)

### 3. Search

Walk from the root, choosing the correct child at each internal node, until you reach a leaf. Within a node, decide how you find the right key/child — linear scan is fine at first; note what a sorted slot array would enable later.

### 4. Insert without splits

Assume the target leaf has room and get the full path working: search to the leaf, insert the record in key order, write the page back. Get this rock-solid before touching splits — most "split bugs" are actually insert bugs.

### 5. Splits

The tricky one. Before coding, walk through these **in words, on paper**, with a drawing per case:

- A leaf is full and a new key arrives. Where does the new node come from? Which keys move? What key gets pushed up to the parent, and is it a *copy* or a *move* (your B vs. B+ decision answers this)?
- The parent is also full. How far up can this cascade?
- The **root** is full. This is the only way the tree grows taller — a new root is created *above*. Connect this to your root-discovery decision from component 2.
- **Split timing:** do you split proactively on the way down (split any full node as you pass it), or reactively on overflow (split when the insert fails, propagating upward)? They lead to genuinely different code shapes; DI ch. 4 discusses both. Decide and document.

### 6. Cursor (`src/cursor.rs` or similar)

A cursor represents a position in the tree and supports "give me the current record" and "advance to the next." Range scan = seek to a start key, advance until past the end key.

**Design questions:**
- What state does a cursor hold — a (page ID, slot index) pair? A stack of the path from the root? Your sibling-pointer decision largely dictates this.
- What happens when a cursor advances past the last record of a leaf?

**Ownership heads-up (from `PLAN.md`):** the cursor is where you'll be most tempted to hold a long-lived reference into a page. Don't — keep node access mediated through the pager, re-fetching by page ID. The borrow checker will resist the reference-holding design anyway; understand *why* before reaching for `Rc`/`RefCell` (LEARNING_GUIDE Phase 2 covers this — the Rust reason and the database reason are the same reason).

### 7. Delete (minimal path required; rebalance is stretch)

Required: search to the leaf, remove the key, write the page back. Decide what happens when a leaf becomes completely empty (leave it? return it to the Phase 1 free list?).

Stretch: underflow handling — borrowing from siblings, merging nodes, collapsing the root. If you attempt it, treat it as its own mini-project with its own decision note and test suite.

---

## Decisions to document before coding

1. **B-tree vs. B+-tree** — and what it means for where values live and what splits push up.
2. **Node layout** — header additions, internal-node child-pointer placement, overflow rule (count vs. bytes).
3. **Root discovery** — meta page vs. fixed root page; what updates on a root split.
4. **Split timing** — proactive descent vs. reactive propagation.
5. **Sibling pointers** — yes/no, and the cursor design that follows.
6. **Key comparison** — byte-wise comparison order; what that implies for how integers must be encoded to sort correctly (think back to your Phase 1 endianness choice — does it still sort right?).

---

## Testing (you write the tests; agent may help design *cases*)

Splits are where the bugs live. Make sure your cases include, at minimum:

- Insert enough sequential keys to force a leaf split, then an internal split, then a **root split** (verify tree height actually grew).
- Insert keys in *random* order and in *reverse* order — different split patterns.
- Keys/values large enough that only a few fit per page (small effective fanout forces deep trees with few inserts — cheap way to stress every path).
- Range scans that start mid-leaf, span a leaf boundary, and cover the whole tree.
- Delete then re-insert the same key.
- Every one of the above, followed by a restart, verified again.

---

## Exit demo

```
$ cargo run walrus.db
walrusdb> PUT banana 1
walrusdb> PUT apple 2
walrusdb> PUT cherry 3
...   (enough keys, in random order, to force multiple splits)
walrusdb> SCAN
apple 2
banana 1
cherry 3
...   (all keys, sorted)
walrusdb> ^C

$ cargo run walrus.db
walrusdb> SCAN
...   (identical sorted output)
```

Insert in random order; a range scan returns sorted; the tree survives restart. That's Phase 2 done.

---

## What you should be able to explain when this phase is complete

- Why B-trees are shallow and what that buys you on disk (fanout → height math, roughly).
- Exactly what happens, step by step, when an insert overflows a leaf — and when it overflows the root.
- Why holding a long-lived reference into a page is dangerous, in both database terms and Rust terms — and why those are the same reason.
- Your B vs. B+ choice and what it changed about your split logic.
- How your cursor finds "the next record" across a leaf boundary.
