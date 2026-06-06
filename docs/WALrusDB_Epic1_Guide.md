# WALrusDB: Epic 1 Implementation Guide
**The Minimum Viable Database (MVD)**

Welcome to Epic 1. Your goal here is to build a thread-safe, single-file key-value store that reads and writes raw binary data. 

Since you are brand new to Go, this guide assumes zero prior knowledge. It will walk you through setting up your environment, explaining *why* we use certain Go features, and providing links to the best official documentation for your journey.

---

## Part 0: The Absolute Basics (Setting up Go)

Before writing database code, you need a working Go environment. Go is famous for its incredible tooling. You don't need external package managers like `pip` or virtual environments; everything is built into the `go` command.

### 1. Installation & The First Run
1. **Install Go:** Download and install Go from [go.dev](https://go.dev/doc/install).
2. **Create your project folder:**
   ```bash/Users/mayukhbasu/Downloads/WALrusDB_Epic1_Guide.md /Users/mayukhbasu/Downloads/WALrusDB_Project_Vision_Final.md/Users/mayukhbasu/Downloads/WALrusDB_Epic1_Guide.md /Users/mayukhbasu/Downloads/WALrusDB_Project_Vision_Final.md
   mkdir walrusdb
   cd walrusdb
   ```
3. **Initialize the Go Module:** This creates a `go.mod` file, which tracks your project's dependencies (similar to `requirements.txt` in Python, but built-in).
   ```bash
   go mod init github.com/yourusername/walrusdb
   ```

### 2. Your Best Friends for Learning Go
Keep these open in your browser while you code:
* **[A Tour of Go](https://go.dev/tour/welcome/1):** The absolute best place to learn Go syntax. Take an hour to click through the sections on Variables, Structs, Maps, and Methods.
* **[Go by Example](https://gobyexample.com/):** Bite-sized snippets showing how to do specific things (like reading files).
* **[The Go Standard Library Docs](https://pkg.go.dev/std):** Go developers rarely use third-party libraries for basic things. The standard library is incredibly powerful.

---

## Phase 1: The Write-Ahead Log (WAL)
**Goal:** Write a Go function that takes a string Key and a byte array Value, and appends them securely to a file on your hard drive.

### Step 1: The Record Struct
In Go, we use `structs` instead of classes to define data shapes. Because we are writing to a disk, we need to be incredibly precise about memory.

Create a file named `record.go`:
```go
package walrusdb

// Record represents a single key-value entry.
type Record struct {
    CRC       uint32 // 4 bytes: Checksum for data integrity
    Timestamp uint64 // 8 bytes: When this was written
    KeySize   uint32 // 4 bytes: Length of the key
    ValueSize uint32 // 4 bytes: Length of the value
    Key       []byte // Variable length
    Value     []byte // Variable length
}
```
* **Why `uint32`?** Unlike Python, where an integer changes size dynamically, a `uint32` is guaranteed to be exactly 4 bytes. This guarantees our header (CRC + Timestamp + KeySize + ValueSize) is **always exactly 20 bytes long**.

### Step 2: Binary Encoding
You cannot write a `struct` directly to a file. You must squash it flat into a `[]byte` (a "slice" of bytes).
* **Documentation to read:** [`encoding/binary`](https://pkg.go.dev/encoding/binary). 
* **Your Task:** Write a method like `func (r *Record) Encode() []byte`. Use `binary.LittleEndian.PutUint32` to convert your integers into bytes, and append them all together.

### Step 3: Appending to the File
* **Documentation to read:** [`os.OpenFile`](https://pkg.go.dev/os#OpenFile).
* **Your Task:** Open a file named `data.cask`. You must use the flags `os.O_APPEND | os.O_CREATE | os.O_RDWR`. This tells the operating system: "Create this file if it doesn't exist, and *only* ever write to the absolute end of it."

---

## Phase 2: The Router (KeyDir)
**Goal:** Keep an in-memory map so that when someone asks for a key, you instantly know exactly which byte in the file holds their data.

### Step 1: The Map and Mutex
Go was built for high concurrency. If two users try to write to a standard map at the exact same time, Go will panic and crash your program to protect your memory.
* **Documentation to read:** [`sync.RWMutex`](https://pkg.go.dev/sync#RWMutex) and [Go Maps](https://gobyexample.com/maps).
* **Your Task:** Create your main database struct:
```go
import "sync"

type IndexEntry struct {
    ValuePos  int64  // The exact byte offset in the file
    ValueSize uint32 // How many bytes to read
}

type DB struct {
    mu      sync.RWMutex
    keyDir  map[string]IndexEntry
    logFile *os.File
}
```

### Step 2: Tying Put() Together
When someone calls `db.Put(key, value)`:
1. Lock the mutex (`db.mu.Lock()`).
2. Encode the Record and append it to the file (Phase 1).
3. The file write will return the byte offset where it just wrote the data.
4. Update the map: `db.keyDir[key] = IndexEntry{ValuePos: offset, ValueSize: size}`.
5. Unlock the mutex (`db.mu.Unlock()`).

### Step 3: The Get() Method (The Magic of O(1) Reads)
* **Documentation to read:** [`os.File.ReadAt`](https://pkg.go.dev/os#File.ReadAt).
* **Your Task:** When someone calls `db.Get(key)`:
1. Lock for reading (`db.mu.RLock()`).
2. Look up the key in the `keyDir` map. If it's not there, return an error.
3. If it is there, use `db.logFile.ReadAt(byteSlice, entry.ValuePos)` to jump directly to that byte on the hard drive and pull the value. No scanning required!

---

## Phase 3: Crash Recovery
**Goal:** The `keyDir` map lives in RAM. If the power goes out, the map is erased. You need to rebuild it from the `.cask` file when the database starts.

### Step 1: The Bootstrapper
* **Documentation to read:** [`io.EOF`](https://pkg.go.dev/io#pkg-variables) (End of File error).
* **Your Task:** Write a `OpenDB()` function. When it runs, it should:
1. Open the `.cask` file.
2. Read the first **20 bytes** (your fixed header).
3. Look at `KeySize` and `ValueSize` to figure out how long the record is.
4. Read the Key.
5. Put the Key and its offset into the `keyDir` map.
6. Jump forward to the next record and repeat.
7. Stop when you hit the `io.EOF` error (which means you reached the end of the file).

### Step 2: Handling Tombstones (Deletions)
Remember our design for deletions! If you are recovering the database and you read a record where the `ValueSize` is `0` (or however you decided to flag a Tombstone), you must **remove** that key from the `keyDir` map, rather than adding it.

---
## Summary of Your First Go Milestones
By completing this guide, you will have successfully:
1. Mastered Go `structs` and explicit memory types.
2. Handled raw binary encoding.
3. Interacted with the OS file system directly.
4. Protected memory using Concurrency Mutexes.
5. Built a functional, hyper-fast database engine.
