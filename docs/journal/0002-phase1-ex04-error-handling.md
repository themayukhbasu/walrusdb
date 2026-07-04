# Phase 1 Exercises Journal — Ex04: Error Handling

**What I built:**

**What surprised me about Rust:**

-   **Important**
    > In exception-based languages (Java, Python, etc.), an error is special — it's not a value, it's a control-flow event that unwinds the stack until something catches it. You can't casually hold onto an exception in a variable and inspect it later; it interrupts execution immediately.
    > 
    > In Rust, Result<T, E> is just a regular enum — no different in kind from your own DBError, or Option<T>, or any struct you'd write. Err(some_error) doesn't halt anything by itself; it's ordinary data sitting in a variable until you decide to match on it, ? it, .unwrap() it, log it, pass it to another function, put it in a Vec, whatever. Nothing forces you to deal with it right at the call site.
    > 
    > That's why let result = read_number(path_str); works exactly like storing any other value — because that's genuinely all it is. The "error handling" only happens later, at whatever point you choose to inspect the enum's variant.
- 

**Where I got stuck:**