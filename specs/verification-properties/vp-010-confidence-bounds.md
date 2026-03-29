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
source_bc: [BC-7.16.004]
module: forge-security
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

# VP-010: Confidence Score Bounds

## Property Statement

For any `SecurityFinding` produced by `analyze_message()` or `detect_schema_drift()`, the `confidence` field satisfies:

```
0.0 <= confidence <= 1.0
```

This is a **value range** invariant: confidence scores are always valid probabilities.

## Source Contract

- **BC-7.16.004** — Security findings include a confidence score for severity assessment and filtering.
- **DI-011** — Confidence score invariant: all security findings must have confidence in [0.0, 1.0].

## Proof Method

| Aspect | Detail |
|--------|--------|
| Method | Model checking (Kani) |
| Tool | Kani Rust Verifier |
| Input space | Bounded `McpMessage` + default `RuleSet` |
| Bound | Depends on message structure complexity |
| Expected time | Seconds to minutes |

## Harness Skeleton

```rust
#[kani::proof]
fn verify_confidence_bounds() {
    let msg: McpMessage = kani::any();
    let rules: RuleSet = default_rules();

    let findings = analyze_message(&msg, &rules);
    for finding in &findings {
        assert!(finding.confidence >= 0.0);
        assert!(finding.confidence <= 1.0);
    }
}
```

### Extended Harness: Schema Drift Path

```rust
#[kani::proof]
fn verify_drift_confidence_bounds() {
    let prev_tools: Vec<ToolDef> = kani::any();
    kani::assume(prev_tools.len() <= 5);
    let curr_tools: Vec<ToolDef> = kani::any();
    kani::assume(curr_tools.len() <= 5);

    let findings = detect_schema_drift(&prev_tools, &curr_tools);
    for finding in &findings {
        assert!(finding.confidence >= 0.0);
        assert!(finding.confidence <= 1.0);
    }
}
```

### Why Kani Over Proptest

While proptest could test random messages, Kani provides **exhaustive** coverage over the bounded input space. This is important because confidence calculation may involve arithmetic that could produce out-of-range values (e.g., division, multiplication of weights). Kani's symbolic execution catches all such paths.

## Feasibility Assessment

| Factor | Assessment |
|--------|-----------|
| Input space | Bounded message structure + fixed rule set |
| Complexity | Medium — depends on number of rules and confidence calculation logic |
| Tool support | Kani handles f64 comparisons via SAT encoding |
| Time | Seconds to minutes |
| Verdict | **FEASIBLE** |

## Lifecycle

| Field | Value |
|-------|-------|
| Introduced | v0.1.0 |
| Modified | — |
| Deprecated | — |
| Retired | — |
