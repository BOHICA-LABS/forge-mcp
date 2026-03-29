---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-03-29T11:25:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: ""
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: "Traffic Inspection"
capability: "CAP-009"
lifecycle_status: active
introduced: v0.1.0
---

# BC-4.09.003 — Capture Buffer Management with Bounded Memory

## Summary

The capture buffer stores captured JSON-RPC messages in memory with a configurable maximum size (default: 100MB per NFR-012). When the buffer reaches capacity, the oldest messages are evicted using FIFO policy to make room for new captures. The user is warned when eviction begins. Messages are never silently dropped — every eviction or capacity event is communicated to the user.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | Capture subsystem is active (BC-4.09.001) |
| PRE-002 | Buffer max size is configured (default 100MB, overridable via `capture.buffer_max_mb` config) |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Buffer memory usage never exceeds the configured maximum |
| POST-002 | When buffer is full, oldest messages (by capture timestamp) are evicted first (FIFO) |
| POST-003 | First eviction event triggers user warning E-CAP-001 in the TUI status bar |
| POST-004 | Evicted message count is tracked and displayed (e.g., "Buffer full: 1,234 messages evicted") |
| POST-005 | New messages are always captured — eviction makes room, never rejects new captures |
| POST-006 | Buffer size is reported in the TUI status bar (e.g., "Buffer: 45MB / 100MB") |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | Buffer memory usage ≤ configured maximum at all times (NFR-012) |
| INV-002 | No message is silently dropped — drops only via explicit FIFO eviction with user notification |
| INV-003 | Eviction order is strictly FIFO (oldest first) |
| INV-004 | Process memory usage stays within overall limits (FM-012 OOM prevention) |
| INV-005 | Buffer operations (insert, evict, query) are thread-safe |

## Edge Cases

| ID | Scenario | Expected Behavior | Source |
|----|----------|-------------------|--------|
| EC-001 | Buffer at 100% capacity, new message arrives | Evict oldest message(s) to accommodate new message; capture new message | FM-010 |
| EC-002 | Single message larger than buffer max size | Reject this message (cannot fit); emit E-CAP-003 warning "Message too large for buffer ({size}MB > {max}MB)" | — |
| EC-003 | Buffer max set to 0 (effectively disabled) | No messages captured; emit E-CAP-004 "Capture buffer disabled (max_size=0)" at startup | — |
| EC-004 | Buffer max set very high (e.g., 10GB) on a machine with 4GB RAM | Respect OS memory limits; if allocation fails, reduce effective max and warn | FM-012 |
| EC-005 | Rapid eviction (sustained input faster than display can show eviction count) | Batch eviction count updates; TUI shows cumulative evicted count | — |
| EC-006 | User exports buffer while eviction is happening | Export sees a consistent snapshot; no partial messages or torn reads | — |
| EC-007 | Buffer_max_mb changed at runtime via config reload | New limit takes effect immediately; if current usage > new limit, evict to comply | — |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-HP-001 | Buffer at 50MB, 100MB max, new 1KB message arrives | Message captured; buffer now 50.001MB; no eviction |
| TV-HP-002 | Buffer at 99.99MB, 100MB max, new 100KB message arrives | Oldest messages evicted to make 100KB room; new message captured; E-CAP-001 warning shown |
| TV-HP-003 | Buffer at 80MB, status bar | Status bar shows "Buffer: 80MB / 100MB (80%)" |
| TV-HP-004 | 500 evictions occurred | Status bar shows "Buffer: 100MB / 100MB — 500 evicted" |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-EC-001 | 150MB message, 100MB buffer | Message rejected; E-CAP-003 "Message too large" warning |
| TV-EC-002 | Buffer max = 0 | E-CAP-004 at startup; no messages captured |
| TV-EC-003 | Config reload changes max from 100MB to 50MB; current usage is 80MB | 30MB of oldest messages evicted immediately; buffer at 50MB |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-ERR-001 | Memory allocation fails during buffer insert | Log E-CAP-005 "Memory allocation failed"; attempt emergency eviction (evict 25% of buffer); retry | FM-012 |
| TV-ERR-002 | Buffer data structure becomes corrupted | Detect via size mismatch; reinitialize buffer; log E-CAP-006; lose in-memory messages with warning to user |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | For all states: buffer_used_bytes ≤ buffer_max_bytes (NFR-012) | Invariant test |
| VP-002 | After N inserts and M evictions: buffer contains the N-M most recent messages | Property-based test |
| VP-003 | Concurrent insert and read operations do not produce data races | Concurrency test (ThreadSanitizer) |
| VP-004 | Eviction always removes the oldest messages first (FIFO property) | Sequence test |
| VP-005 | No message is captured without notification to user if eviction occurred | State machine test |

## Traceability

| Item | Reference |
|------|-----------|
| L2 Capability | CAP-009 (Message Capture & Analysis) |
| NFR | NFR-012 (100MB default buffer limit) |
| Failure Mode | FM-010 (buffer overflow), FM-012 (OOM prevention) |
| Error Code | E-CAP-001 (eviction started), E-CAP-003 (message too large), E-CAP-004 (buffer disabled), E-CAP-005 (alloc failure) |
| Related BCs | BC-4.09.001 (capture — data producer), BC-4.10.001 (filtering — reads from buffer) |
