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