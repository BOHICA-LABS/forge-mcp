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
source_bc: [BC-4.09.001]
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

# VP-006: Message Capture Preserves Content

## Property Statement

For any `McpMessage` `m`, after `buffer.append(m.clone())`, retrieving the message from the buffer yields a message where `raw_bytes` is **byte-identical** to the original.

This is a **data integrity** property: the capture buffer must never corrupt, truncate, or transform message content.

## Source Contract

- **BC-4.09.001** — Full message capture: every JSON-RPC message passing through the proxy is captured with its complete raw bytes.
- **DI-005** — Captured message integrity: raw bytes stored in the buffer must be identical to the bytes received from the transport.

## Proof Method

| Aspect | Detail |
|--------|--------|
| Method | Property-based testing (proptest) |
| Tool | `proptest` crate |
| Input space | Arbitrary `McpMessage` instances via custom `Arbitrary` impl |
| Iterations | Default 256 cases (configurable) |
| Shrinking | Automatic — proptest shrinks failing cases to minimal reproduction |

## Harness Skeleton

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn capture_preserves_content(msg in arb_mcp_message()) {
        let mut buf = CaptureBuffer::new(100);
        let original_bytes = msg.raw_bytes.clone();
        buf.append(msg);
        let retrieved = buf.iter().last().unwrap();
        prop_assert_eq!(&retrieved.raw_bytes, &original_bytes);
    }
}
```

### Strategy: `arb_mcp_message`

Generates arbitrary `McpMessage` instances with:
- Random `raw_bytes` (0 to 64KB)
- Random `method` strings
- Random `id` values (integer, string, or null)
- Random `params` / `result` JSON values
- Monotonically increasing `timestamp` values

```rust
fn arb_mcp_message() -> impl Strategy<Value = McpMessage> {
    (
        prop::collection::vec(any::<u8>(), 0..65536),
        "[a-z]{1,20}",
        any::<u64>(),
    ).prop_map(|(bytes, method, ts)| McpMessage {
        raw_bytes: bytes,
        method: Some(method),
        timestamp: ts,
        ..Default::default()
    })
}
```

## Feasibility Assessment

| Factor | Assessment |
|--------|-----------|
| Input space | Arbitrary messages — well-suited for property-based testing |
| Complexity | Low — append + retrieve is a simple round-trip check |
| Tool support | Excellent — proptest is Rust's standard PBT library |
| Time | Milliseconds per case, seconds total |
| Verdict | **FEASIBLE** |

## Lifecycle

| Field | Value |
|-------|-------|
| Introduced | v0.1.0 |
| Modified | — |
| Deprecated | — |
| Retired | — |
