# WALrusDB: A Log-Structured Key-Value Storage Engine

## 1. Vision & Introduction
WALrusDB is an embedded, high-performance, log-structured key-value storage engine written entirely in Go. Initially built on the Bitcask architecture, the project is designed to progressively evolve into a full Log-Structured Merge (LSM) tree. 

To ensure long-term extensibility without polluting the core logic, WALrusDB enforces a strict **Clean Boundary** pattern. The core engine is purely an embedded storage library with zero knowledge of networking, distributed consensus, or sharding. It implements a simple `StorageEngine` interface, allowing higher-level systems to be built on top of it.

## 2. Project Goals
* **Zero-Dependency Core:** The core storage engine must be built using only the Go standard library (`os`, `io`, `bytes`, `sync`).
* **High-Throughput Writes:** Leverage the append-only nature of log-structured systems to bypass random disk I/O penalties.
* **Architectural Evolution:** Start with a pure, high-speed Hash Index (Bitcask) and intentionally pivot to a Sorted MemTable (LSM-Tree) to support range queries.
* **AI & Advanced Extensibility:** Design internal indexing to gracefully support high-dimensional vector spaces and cosine similarity.

## 3. Architecture Overview (Initial State)
The starting architecture follows the Bitcask model—the foundational building block for LSM-Trees.
1.  **Write-Ahead Log (WAL):** All write operations are strictly appended to the end of an active file.
2.  **In-Memory Index (KeyDir):** A concurrent Go map tracks keys to their exact byte offsets on disk.
3.  **Read Path:** A `GET` request retrieves the byte offset from memory, jumps directly to that offset on disk, and returns the value.

---

## 4. Development Roadmap: The 5 Epics

### Epic 1: The Minimum Viable Database (MVD)
*The foundation. A fully functional, thread-safe embedded database.*
* **Phase 1: The WAL:** Establish raw disk interaction and data serialization. Sequentially append binary-encoded records to a single `.cask` file.
* **Phase 2: The Router:** Build a concurrent Go `map` (Primary Hash Index) to enable `O(1)` point reads via `os.File.ReadAt`.
* **Phase 3: Crash Recovery:** Implement a startup routine that sequentially scans the `.cask` file to rebuild the in-memory KeyDir.

### Epic 2: System Robustness & Mechanics
*Implementing the core mechanics required for a stable, long-running Bitcask engine.*
* **Phase 4: Segmented Storage:** Implement file rotation. Freeze the active WAL as a read-only segment when it hits a size threshold to prevent infinite file growth.
* **Phase 5: Bitcask Compaction:** Safely merge read-only segments in the background, discarding deleted (Tombstoned) keys to reclaim disk space.
* **Phase 6: Hint Files:** Generate lightweight index snapshots during compaction to drastically speed up Phase 3 crash recovery.

### Epic 3: The LSM Pivot
*A dedicated architectural shift from an unsorted Hash Index to a true LSM-Tree.*
* **Phase 7: The Sorted MemTable:** Rip out the standard Go `map` and replace it with a thread-safe Skip List or Red-Black Tree. 
* **Phase 8: SSTable Flushes:** Modify the storage layer so that when the MemTable fills up, it flushes to disk as a Sorted String Table (SSTable) rather than just a raw append log.
* **Phase 9: Range Queries:** Expose new API methods (e.g., `Scan(startKey, endKey)`) now that the memory and disk structures are alphabetically sorted.

### Epic 4: Advanced Indexing & AI
*Evolving the feature set on top of the powerful LSM core.*
* **Secondary Indexes:** Add an inverted index map (`Value -> []Key`) to support lookups by value.
* **Vector Indexing (AI Layer):** Introduce schemas for `[]float32` serialization and implement cosine-similarity search algorithms for embedding retrieval.

### Epic 5: The Distributed Horizon
*Wrapping the embedded engine in a distributed network layer.*
* **The Network Layer:** Expose the `StorageEngine` interface over the network using an HTTP or gRPC server.
* **Partitioning & Consensus:** Implement a routing node using Consistent Hashing and Raft to replicate logs across multiple WALrusDB instances.

---

## 5. System Requirements
* **Language:** Go 1.21+
* **Dependencies:** Standard Library only for core DB.

---

## 6. Detailed Technical Specifications

### Epic 1: The Minimum Viable Database (MVD)
* **Phase 1: The WAL (Write-Ahead Log)**
  * **Spec:** Data must be written to an active `.cask` file opened with `os.O_APPEND`. Data is never overwritten. Deletions are handled by writing a "Tombstone" (a record with a special flag or 0-byte value).
  * **Byte Layout:** Every entry must have a fixed-size header to enable exact byte reading.
    * Header: `CRC32` (4 bytes) + `Timestamp` (8 bytes) + `KeySize` (4 bytes) + `ValueSize` (4 bytes) = 20 Bytes.
    * Body: `Key` (variable) + `Value` (variable).
* **Phase 2: The Router (KeyDir)**
  * **Spec:** An in-memory Hash Map mapping a string `Key` to an `IndexEntry` struct (FileID, ValueSize, ValueOffset).
  * **Concurrency:** Must be protected by a `sync.RWMutex` to allow multiple concurrent readers but safe exclusive writes.
  * **Read Path:** Uses `os.File.ReadAt(offset)` to jump directly to the data on disk without scanning the file.
* **Phase 3: Crash Recovery**
  * **Spec:** On startup, the engine opens the `.cask` file and sequentially reads every header. It jumps forward by `KeySize + ValueSize`, updating the KeyDir map for every record. If it encounters a Tombstone, it deletes that key from the KeyDir.

### Epic 2: System Robustness & Mechanics
* **Phase 4: Segmented Storage**
  * **Spec:** A single `.cask` file cannot grow infinitely. When the active file reaches a configured threshold (e.g., 10MB or 1GB), it is closed and marked as immutable (read-only). A new active `.cask` file is created for subsequent writes.
* **Phase 5: Bitcask Compaction**
  * **Spec:** A background Go routine (`goroutine`) periodically scans all read-only `.cask` files. It checks the in-memory KeyDir to see if a record is still the "active" version. If yes, it writes it to a new merged file. If no (it was overwritten or Tombstoned), it discards it. The old fragmented files are then deleted from the OS.
* **Phase 6: Hint Files**
  * **Spec:** Parsing gigabytes of WAL data during Phase 3 recovery is slow. During Phase 5 compaction, the engine will also write a "Hint File" for each compacted segment. The Hint file contains *only* the Keys and their Offsets (bypassing the Values), allowing startup times to be orders of magnitude faster.

### Epic 3: The LSM Pivot
* **Phase 7: The Sorted MemTable**
  * **Spec:** The standard Go `map` is replaced with a custom-built, thread-safe **Skip List** or **Red-Black Tree**. Keys are now stored in strict lexicographical (alphabetical) order in RAM.
* **Phase 8: SSTable Flushes**
  * **Spec:** Because the MemTable is sorted, when it reaches capacity, it is flushed to disk as a **Sorted String Table (SSTable)** instead of an unsorted append log.
* **Phase 9: Range Queries**
  * **Spec:** Expose a `Scan(startKey, endKey)` API. The engine will merge data from the active MemTable and the on-disk SSTables, returning keys in guaranteed sorted order.

### Epic 4: Advanced Indexing & AI
* **Phase 10: Secondary Indexes**
  * **Spec:** Maintain an additional in-memory data structure mapping `Value -> []Key`. This allows users to query by value (e.g., "Find all users with age=30").
* **Phase 11: Vector Indexing (AI Layer)**
  * **Spec:** Define a schema to serialize `[]float32` arrays as database values. Implement a mathematical `Search(queryVector, topK)` function that iterates through the data and calculates the Cosine Similarity, returning the nearest neighbors for AI embedding workloads.

### Epic 5: The Distributed Horizon
* **Phase 12: The Network Layer**
  * **Spec:** Wrap the core `StorageEngine` interface in a gRPC or standard Go HTTP server. Define Protobufs or JSON payloads for network-based `Get`, `Put`, and `Delete` requests.
* **Phase 13: Partitioning & Consensus**
  * **Spec:** Implement a routing layer using Consistent Hashing to shard data across multiple WALrusDB server instances. Integrate the Raft consensus algorithm to replicate the WAL across nodes for fault tolerance.
