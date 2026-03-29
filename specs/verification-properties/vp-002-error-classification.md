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
source_bc: [BC-2.05.010]
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

# VP-002: Error Classification Correctness

## Property Statement

For any JSON-RPC response `resp`, `classify_error(resp)` MUST return:
- `McpError::Tool` if and only if `resp.result.isError == true`
- `McpError::Protocol` if and only if `resp.error` is present with a JSON-RPC error code
- `Ok` otherwise

These three states MUST be **mutually exclusive** and **exhaustive**. No response can be classified into more than one category, and every response MUST be classified into exactly one.

This is a **classification correctness** property: the error classifier partitions all responses into exactly three non-overlapping categories.

## Source Contract

- **BC-2.05.010** — Error classification: tool errors (`isError` in result) vs protocol errors (JSON-RPC error object) must be distinguished for correct error propagation.
- **DI-020** — The error classification invariant guarantees downstream consumers always receive correctly typed errors.

## Proof Method

| Aspect | Detail |
|--------|--------|
| Method | Model checking (Kani) |
| Tool | Kani Rust Verifier |
| Input space | 3 booleans (`has_error_field`, `is_error_flag`, `has_result`) + bounded payload |
| Bound | Exhaustive over boolean combinations; bounded JSON payload |
| Expected time | Seconds |

## Harness Skeleton

```rust
#[kani::proof]
fn verify_error_classification() {
    let has_error_field: bool = kani::any();
    let is_error_flag: bool = kani::any();
    let has_result: bool = kani::any();

    let resp = make_response(has_error_field, is_error_flag, has_result);
    let classification = classify_error(&resp);

    // Mutual exclusivity
    match classification {
        McpError::Tool(_) => {
            assert!(is_error_flag && has_result && !has_error_field);
        }
        McpError::Protocol(_) => {
            assert!(has_error_field && !has_result);
        }
        Ok(_) => {
            assert!(has_result && !is_error_flag && !has_error_field);
        }
    }
}
```

### Helper: `make_response`

Constructs a synthetic JSON-RPC response with the given field presence/flags. Must cover all valid combinations of `error`, `result`, and `result.isError`.

## Feasibility Assessment

| Factor | Assessment |
|--------|-----------|
| Input space | Small — 3 booleans + bounded payload structure |
| Complexity | Low — classification is a simple match on field presence |
| Tool support | Excellent — Kani handles boolean + enum reasoning natively |
| Expected time | Seconds |
| Verdict | **FEASIBLE** |

## Lifecycle

| Field | Value |
|-------|-------|
| Introduced | v0.1.0 |
| Modified | — |
| Deprecated | — |
| Retired | — |
