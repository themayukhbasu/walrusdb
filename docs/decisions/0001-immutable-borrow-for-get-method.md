Date: 03rd July 2026
Status: Implemented:

## Context

I reviewed the code to understand what it was doing while simultaneously learning Rust fundamentals. During this process, I explored the difference between &self and &mut self.

I learned that &self provides an immutable reference to the instance, allowing methods to read data without modifying it. In contrast, &mut self provides a mutable reference, allowing methods to modify the instance's state. Rust requires mutable access only when a value is intended to change.

## Decision:

We changed the method signature from:
fn get(&mut self, key: &str) -> Option<&String>
to:
fn get(&self, key: &str) -> Option<&String>

## Rationale: 
The get method performs a read-only operation and does not modify the state of the database. Therefore, requiring &mut self is unnecessary.

This change aligns with Rust's principle of least privilege: methods should request only the level of access they actually need. Since get only reads data, an immutable reference (&self) is the appropriate choice.
