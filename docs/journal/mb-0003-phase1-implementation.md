# Phase 1 Exercises Journal

> Author: @mb0850

## A. What I built:

let file = OpenOptions::new()
.create(true)
.write(true)
.read(true)
.append(true)
.open(path)?;

## B. What surprised me about Rust:

### 1. Rust OpenOptions append can screw up our DB

- So I wanted to understand more of `OpenOptions` that we use for our db file
- I was thinking about whether I should add `.append(true)` like the below:

```rust
let file = OpenOptions::new()
.create(true)
.write(true)
.read(true)
.append(true) // <-- here
.open(path) ?;
```

- I assumed that `write` operations generally truncate and write while append doesn't truncate.
- But in Rust, as per the [OpenOptions doc](https://doc.rust-lang.org/std/fs/struct.OpenOptions.html#method.write),
  write calls overwrite the content without truncating the file.
- And the append mode guarantees that writes will be positioned at the current end of file, even when there are other
  processes or threads appending to the same file.
- which means that the manual seeks that we do when we have to say overwrite a page (say page 3) in a multi-page file,
  we will never be able to do that
- `append(true)` makes the position argument to seek irrelevant for writes; the kernel forces every write to EOF
  regardless.
- this isn't a Rust-specific quirk — it's POSIX `O_APPEND` semantics (`man 2 open`), which Rust's `append(true)` maps
  directly onto. Any language/runtime that opens the file with `O_APPEND` inherits the same behavior.
- so putting `append(true)` is not just a redundant option to `write(true)`,
    - its actively wrong: it would silently turn every "overwrite page 3" into "corrupt the file by appending garbage
      past the end"
    - its a nastier bug than a compiler error since it won't fail loudly.

## C. Where I got stuck:
