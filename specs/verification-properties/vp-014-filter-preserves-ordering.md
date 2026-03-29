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
source_bc: [BC-4.10.001]
module: forge-traffic
proof_method: proptest
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

# VP-014: Traffic Filter Preserves Message Ordering

## Property Statement

For any `CaptureBuffer` `b` and `TrafficFilter` `f`, the result of `b.filter(f)` MUST preserve the **relative ordering** of messages from the original buffer. That is, if message A appears before message B in the buffer, and both pass the filter, then A MUST appear before B in the filtered result.

This is an **order preservation** property: filtering MUST be a monotonic projection — it removes elements but MUST NOT reorder them.

## Source Contract

- **BC-4.10.001** — Traffic filtering by method, direction, server, and time range. Filtered views must maintain temporal ordering for debuggability.
- **DI-006** — Message ordering invariant: captured messages maintain temporal ordering through all operations including filtering.

## Proof Method

| Aspect | Detail |
|--------|--------|
| Method | Property-based testing (proptest) |
| Tool | `proptest` crate |
| Input space | Arbitrary buffers (0..100 messages) × arbitrary filters |
| Iterations | 256+ cases |
| Shrinking | Automatic — fails shrink to minimal buffer + filter |

## Harness Skeleton

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn filter_preserves_ordering(
        messages in prop::collection::vec(arb_mcp_message(), 0..100),
        filter in arb_traffic_filter()
    ) {
        let mut buf = CaptureBuffer::new(1000);
        for msg in &messages {
            buf.append(msg.clone());
        }
        let filtered = buf.filter(&filter);
        for i in 1..filtered.len() {
            prop_assert!(
                filtered[i-1].timestamp <= filtered[i].timestamp,
                "Order violated: msg[{}].ts={} > msg[{}].ts={}",
                i-1, filtered[i-1].timestamp, i, filtered[i].timestamp
            );
        }
    }
}
```

### Strategy: `arb_traffic_filter`

Generates arbitrary filter configurations:

```rust
fn arb_traffic_filter() -> impl Strategy<Value = TrafficFilter> {
    (
        prop::option::of("[a-z/]{1,30}"),          // method filter
        prop::option::of(arb_direction()),          // direction filter
        prop::option::of("[a-z-]{1,20}"),           // server name filter
        prop::option::of((0u64..1000, 0u64..1000)) // time range
            .prop_map(|opt| opt.map(|(a, b)| (a.min(b), a.max(b)))),
    ).prop_map(|(method, dir, server, time_range)| TrafficFilter {
        method,
        direction: dir,
        server,
        time_range,
    })
}
```

### Strategy: `arb_mcp_message` (with monotonic timestamps)

To ensure the buffer starts ordered (as it would in production), messages are generated with monotonically increasing timestamps:

```rust
fn arb_mcp_messages_ordered(count: Range<usize>) -> impl Strategy<Value = Vec<McpMessage>> {
    prop::collection::vec(arb_mcp_message_base(), count)
        .prop_map(|mut msgs| {
            msgs.sort_by_key(|m| m.timestamp);
            msgs
        })
}
```

### Composition with VP-005

This property composes with VP-005 (ring buffer FIFO ordering): VP-005 guarantees the buffer maintains order on insert, and VP-014 guarantees filtering preserves that order. Together they establish end-to-end temporal ordering.

## Feasibility Assessment

| Factor | Assessment |
|--------|-----------|
| Input space | Bounded buffers (0..100 messages), structured filters |
| Complexity | Low — order check is a simple pairwise comparison |
| Tool support | Excellent — proptest with custom strategies |
| Expected time | Milliseconds per case |
| Verdict | **FEASIBLE** |

## Lifecycle

| Field | Value |
|-------|-------|
| Introduced | v0.1.0 |
| Modified | — |
| Deprecated | — |
| Retired | — |
