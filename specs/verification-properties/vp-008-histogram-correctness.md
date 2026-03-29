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
source_bc: [BC-6.13.001]
module: forge-health
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

# VP-008: Latency Histogram Mathematical Correctness

## Property Statement

For any non-empty sequence of latency values, the histogram's **p50**, **p95**, and **p99** percentiles MUST be computed correctly — that is, they MUST match a reference sort-based implementation within a tolerance of 1.0ms.

This is an **algorithmic correctness** property: the histogram's streaming/approximate percentile computation MUST agree with the ground-truth sorted-array method.

## Source Contract

- **BC-6.13.001** — Latency metrics collection with percentile reporting (p50, p95, p99) for per-server health monitoring.

## Proof Method

| Aspect | Detail |
|--------|--------|
| Method | Property-based testing (proptest) |
| Tool | `proptest` crate |
| Input space | Non-empty vectors of f64 values in [0.0, 10000.0], length 1..1000 |
| Reference | Sort-based percentile computation |
| Tolerance | 1.0ms (acceptable for streaming histogram approximation) |
| Iterations | 256+ cases |
| Shrinking | Automatic — proptest shrinks failing cases to minimal reproduction |

## Harness Skeleton

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn histogram_percentiles_correct(
        values in prop::collection::vec(0.0f64..10000.0, 1..1000)
    ) {
        let hist = Histogram::from_values(&values);
        let mut sorted = values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let ref_p50 = sorted[sorted.len() / 2];
        let ref_p95 = sorted[(sorted.len() as f64 * 0.95) as usize];
        let ref_p99 = sorted[(sorted.len() as f64 * 0.99) as usize];

        prop_assert!((hist.p50() - ref_p50).abs() < 1.0);  // within 1ms tolerance
        prop_assert!((hist.p95() - ref_p95).abs() < 1.0);
        prop_assert!((hist.p99() - ref_p99).abs() < 1.0);
    }
}
```

### Reference Implementation

The reference is a naive sorted-array percentile:

```rust
fn reference_percentile(sorted: &[f64], p: f64) -> f64 {
    let idx = (sorted.len() as f64 * p) as usize;
    sorted[idx.min(sorted.len() - 1)]
}
```

### Edge Cases Covered by Strategy

- Single-element vector (p50 = p95 = p99 = that element)
- All-same values (all percentiles equal)
- Monotonically increasing / decreasing sequences
- Very small values near zero
- Values at the 10000.0 boundary

## Feasibility Assessment

| Factor | Assessment |
|--------|-----------|
| Input space | Bounded vectors (1..1000 elements, values 0..10000) |
| Complexity | Low — comparison against reference implementation |
| Tool support | Excellent — proptest generates f64 vectors natively |
| Expected time | Milliseconds per case |
| Tolerance | 1.0ms — reasonable for streaming histogram approximation |
| Verdict | **FEASIBLE** |

## Lifecycle

| Field | Value |
|-------|-------|
| Introduced | v0.1.0 |
| Modified | — |
| Deprecated | — |
| Retired | — |
