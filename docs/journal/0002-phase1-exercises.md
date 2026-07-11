# Phase 1 Exercises Journal

## What I built:

## What surprised me about Rust:

### Error is Different in Rust

> In exception-based languages (Java, Python, etc.), an error is special — it's not a value, it's a control-flow event
> that unwinds the stack until something catches it. You can't casually hold onto an exception in a variable and inspect
> it later; it interrupts execution immediately.
>
> In Rust, Result<T, E> is just a regular enum — no different in kind from your own DBError, or Option<T>, or any
> struct you'd write. Err(some_error) doesn't halt anything by itself; it's ordinary data sitting in a variable until
> you decide to match on it, ? it, .unwrap() it, log it, pass it to another function, put it in a Vec, whatever. Nothing
> forces you to deal with it right at the call site.
>
> That's why let result = read_number(path_str); works exactly like storing any other value — because that's genuinely
> all it is. The "error handling" only happens later, at whatever point you choose to inspect the enum's variant.

### FileSystem acted weird

- After implementing the TinyKV, I tried to simulate an error where I delete the database file while I am in REPL
- But contrary to expectation, the program didn't crash and happily worked - GET, PUT, DELETE, DUMP - everything worked.
- The file is opened only once with `std::fs::File`. After that all operations are done through the file object.
- This is what I learnt from Claude:
    - What ends up happening is that on POSIX filesystems (Linux/macOS), a file has two separate identities:
        - the inode: the real thing - metadata plus pointers to the actual data blocks on disk
        - the directory entry: a name, like `target/tinykvstore.bin`, that maps to an inode number
        - `rm` doesn't touch the inode or its data at all - it just removes the directory entry, i.e. it deletes the
          name, not the data.
        - The `BlockStore.file: std::fs::File` was handed a file descriptor at open() and that descriptor points
          straight at the inode - it never looks at the path again.
        - So `rm`-ing the path mid-REPL is invisible to the already open file;
        - reads/writes keep hitting the same inode exactly as before
    - How OS handles `rm`:
        - OS's version is _reference-counted_: an inode tracks how many directory entries point to it (link count)
        - and how many open file descriptors reference it.
        - `rm` decrements the link count to zero, but the inode isn't actually freed while the REPL process still holds
          it open
        - it lingers in a kind of "unlinked-but-alive" limbo.
        - The moment the process exits and closes that last descriptor, the reference count hits true zero,
        - and then the kernel actually reclaims the inode and its data blocks - for real this time, unrecoverably.
        - That's why the restart shows an empty DB:
            - `.create(true)` finds nothing at the path and creates a brand-new empty inode.

## Where I got stuck:

### comprehending and internalizing the concept of `usize` in Rust

    - `usize` is a type whose _**width** matches the pointer width_ of the target (32-bit on a 32-bit target, 64-bit on
      64-bit) system, and it's guaranteed big enough to hold the size of any single object or index in that process's
      memory space. It's fundamentally about **addressing memory in this process**.

### `From<DecodeError> for DBError` felt circular

### Why the compiler couldn't infer the error type in a `match` + `?`

### `break` needing to match the function's `Result` return type