---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-03-29T14:05:00
phase: 1b
inputs: [specs/prd.md, specs/architecture/ARCH-INDEX.md]
input-hash: ""
traces_to: specs/prd.md
source_bc: [BC-4.09.003]
module: forge-traffic
proof_method: kani
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: v0.1.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-005: Ring Buffer Bounds and FIFO Ordering

## Property Statement

For a `CaptureBuffer` with capacity `N`:

1. **Bound invariant:** `len()` never exceeds `N` after any sequence of operations.
2. **FIFO eviction:** After `append` when the buffer is full, the oldest message is evicted.
3. **Ordering invariant:** Iteration order always matches insertion order for non-evicted messages.

These three sub-properties together guarantee the ring buffer is memory-bounded and behaviorally correct.

## Source Contract

- **BC-4.09.003** — Per-server capture buffer with configurable size limit and FIFO eviction.
- **NFR-012** — Memory budget: capture buffers must not grow unbounded.
- **DI-006** — Message ordering invariant: captured messages maintain temporal ordering.

## Proof Method

| Aspect | Detail |
|--------|--------|
| Method | Model checking (Kani) |
| Tool | Kani Rust Verifier |
| Input space | Bounded buffer capacity (N=8), bounded operations (16 appends) |
| Bound | `#[kani::unwind(17)]` |
| Expected time | Seconds to minutes |

## Harness Skeleton

```rust
#[kani::proof]
#[kani::unwind(17)]
fn verify_ring_buffer_bounds() {
    let capacity: usize = 8;
    let mut buf = CaptureBuffer::new(capacity);
    let num_ops: u8 = kani::any();
    kani::assume(num_ops <= 16);

    for _ in 0..num_ops {
        let msg = kani::any::<McpMessage>();
        buf.append(msg);
        assert!(buf.len() <= capacity);
    }

    // Verify FIFO ordering
    let items: Vec<_> = buf.iter().collect();
    for i in 1..items.len() {
        assert!(items[i-1].timestamp <= items[i].timestamp);
    }
}
```

### Sub-Property: FIFO Eviction

An additional targeted harness verifies that when the buffer is full, the next append evicts the oldest entry:

```rust
#[kani::proof]
fn verify_fifo_eviction() {
    let mut buf = CaptureBuffer::new(2);
    let msg_a = McpMessage { id: 1, timestamp: 100, .. };
    let msg_b = McpMessage { id: 2, timestamp: 200, .. };
    let msg_c = McpMessage { id: 3, timestamp: 300, .. };

    buf.append(msg_a);
    buf.append(msg_b);
    assert_eq!(buf.len(), 2);

    buf.append(msg_c); // evicts msg_a
    assert_eq!(buf.len(), 2);
    let items: Vec<_> = buf.iter().collect();
    assert_eq!(items[0].id, 2); // msg_b still present
    assert_eq!(items[1].id, 3); // msg_c added
}
```

## Feasibility Assessment

| Factor | Assessment |
|--------|-----------|
| Input space | Bounded buffer (N=8), bounded operations (16) — small state space |
| Complexity | Low — ring buffer is a well-understood data structure |
| Tool support | Kani handles bounded loops and array indexing well |
| Time | Seconds to minutes |
| Verdict | **FEASIBLE** |

## Lifecycle

| Field | Value |
|-------|-------|
| Introduced | v0.1.0 |
| Modified | — |
| Deprecated | — |
| Retired | — |
