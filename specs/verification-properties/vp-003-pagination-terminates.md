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
source_bc: [BC-2.05.001]
module: forge-core
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

# VP-003: Pagination State Machine Termination

## Property Statement

For any sequence of pagination cursors returned by a server, `collect_all_pages()` MUST terminate in at most `MAX_PAGES` (default 100) iterations. If a cursor repeats (cycle detection), iteration MUST stop immediately and return accumulated results. The function MUST NOT enter an infinite loop regardless of server behavior.

This is a **termination** property: the pagination collector always completes within bounded time.

## Source Contract

- **BC-2.05.001** — Paginated result collection must handle arbitrary cursor sequences from MCP servers.
- **DI-019** — Pagination must terminate: the proxy cannot loop indefinitely fetching pages.
- **FM-019** — Failure mode: cursor cycle causes infinite pagination. Mitigated by cycle detection + hard cap.

## Proof Method

| Aspect | Detail |
|--------|--------|
| Method | Model checking (Kani) |
| Tool | Kani Rust Verifier |
| Input space | Bounded cursor set (max 10 unique cursors), max 100 iterations |
| Bound | `#[kani::unwind(101)]` |
| Expected time | Seconds to minutes (depending on cursor set size) |

## Harness Skeleton

```rust
#[kani::proof]
#[kani::unwind(101)]
fn verify_pagination_terminates() {
    let num_pages: u8 = kani::any();
    kani::assume(num_pages <= 100);
    let has_cycle: bool = kani::any();

    let cursors = generate_cursor_sequence(num_pages, has_cycle);
    let result = collect_all_pages_bounded(&cursors, 100);

    assert!(result.pages_fetched <= 100);
    if has_cycle {
        assert!(result.terminated_early);
    }
}
```

### Helper: `generate_cursor_sequence`

Generates a sequence of cursor strings. If `has_cycle` is true, introduces a repeated cursor at a random position. The function is deterministic given Kani's symbolic inputs.

### Helper: `collect_all_pages_bounded`

Pure-core version of the pagination collector that takes a cursor sequence and max-page bound. No I/O — operates on pre-generated cursor list.

## Feasibility Assessment

| Factor | Assessment |
|--------|-----------|
| Input space | Bounded loop (100 max iterations), bounded cursor set (10 unique) |
| Complexity | Low — loop with counter + set membership check |
| Tool support | Kani handles bounded loops with `#[kani::unwind]` |
| Expected time | Seconds to minutes |
| Verdict | **FEASIBLE** |

## Lifecycle

| Field | Value |
|-------|-------|
| Introduced | v0.1.0 |
| Modified | — |
| Deprecated | — |
| Retired | — |
