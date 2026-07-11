# Phase 1 Exercises Journal

## A. What I built:

- Completed the 6 exercises in prep for Phase 1 implementation

## B. What surprised me about Rust:

### 1. Error is Different in Rust

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

### 2. FileSystem acted weird

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

## C. Where I got stuck:

### 1. comprehending and internalizing the concept of `usize` in Rust

- `usize` is a type whose _**width** matches the pointer width_ of the target (32-bit on a 32-bit target, 64-bit on
  64-bit) system, and it's guaranteed big enough to hold the size of any single object or index in that process's
  memory space. It's fundamentally about **addressing memory in this process**.
- `usize` is enforced specifically for indexing and sizing things that live in this process's own memory — array/slice
  indexing (`arr[i]`), `.len()`, `size_of::<T>()`. It's not a general "memory" rule — file offsets and block indices
  (like `BlockStore`'s `block_idx`/`num_blocks`, which are `u64`) are external, OS-managed quantities, not part of the
  process's address space, so they aren't forced into `usize`.

### 2. `From<DecodeError> for DBError` felt circular

**Issue**

- While trying to create hierarchical Errors, where DBError had a DecodeError element so that errors can be logically
  grouped, I got confused when some functions like Decode which should return DecodeError also had some lines which
  propagated DBError.
- Example:

```rust
fn decode(...) -> Result<Self, DecodeError> {
    ...
    if key_len + val_len > 59 {
        return Err(DBError::RecordDataOutOfBounds(key_len, val_len));
    }
    ...
}
```

- Since we were raising DBError but the function needed to return DecodeError
- So I was thinking for a while to impl the `From` trait for converting DBError to DecodeError
- But this raises a big problem, logically the DecodeError is part of DBError, but here if I implement way to convert
  DBError to DecodeError, that makes it look like a **circular dependency**

**Fix**

- The fix was however quite simple.
- Just had to change the thought process where if we face the error where `key_len + value_len > 59`, that means that we
  are trying to decode a corrupted data which means its time for a new `DecodeError`
- And thus, `DecodeError::InvalidRecordLength` was born

### 3. Why the compiler couldn't infer the error type in a `match` + `?`

The match + ? one: This was in put. You had a match where every arm ended in Ok(()), followed by }?; and then a separate return Ok(()); after it. The compiler error was:     
error[E0282/E0283]: type annotations needed                                                                                                                                   
cannot infer type of the type parameter `E` declared on the enum `Result`                                                                                                     
The fix was deleting the ?; and the redundant return Ok(());, letting the match itself be put's tail expression.

**Note**: _Hit this error, tried a few things based on the compiler's suggestion until it compiled — didn't fully          
understand why at the time._

### 4. `break` needing to match the function's `Result` return type

The break one: This was in repl. You had loop { ... break; ... } with repl's signature -> Result<(), DBError>. The compiler said:                                             
error[E0308]: mismatched types                                                                                                                                                
expected `Result<(), DBError>`, found `()`                                                                                                                                    
help: give the `break` a value of the expected type: `break Ok(());`

**Note**: _Hit this error, tried a few things based on the compiler's suggestion until it compiled — didn't fully          
understand why at the time._