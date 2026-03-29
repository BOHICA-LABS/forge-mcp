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
source_bc: [BC-7.16.003]
module: forge-security
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

# VP-011: Schema Drift Detection Completeness

## Property Statement

For any two tool sets that differ in at least one field (name, description, input schema, annotations) of at least one tool, `detect_schema_drift(prev, current)` returns a **non-empty** `Vec<SecurityFinding>`.

This is a **completeness** property: the drift detector must catch every possible mutation. Zero false negatives for single-field changes.

## Source Contract

- **BC-7.16.003** — Schema drift detection: compare current tool definitions against previously seen definitions and flag changes.
- **R-014** — Risk: tool schema manipulation (tool poisoning). Mitigation depends on detecting all schema changes.
- **FM-020** — Failure mode: silent schema drift. Mitigated by comprehensive drift detection.

## Proof Method

| Aspect | Detail |
|--------|--------|
| Method | Property-based testing (proptest) |
| Tool | `proptest` crate |
| Input space | Arbitrary tool sets (1..10 tools) with single-field mutations |
| Strategy | Generate base tool set, then apply exactly one known mutation |
| Iterations | 256+ cases |

## Harness Skeleton

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn drift_detects_any_change(
        base_tools in arb_tool_set(1..10),
        mutation in arb_tool_mutation()
    ) {
        let mutated = apply_mutation(&base_tools, &mutation);
        let findings = detect_schema_drift(&base_tools, &mutated);
        prop_assert!(
            !findings.is_empty(),
            "Drift not detected for mutation: {:?}",
            mutation
        );
    }
}
```

### Strategy: `arb_tool_set`

Generates a vector of `ToolDef` with:
- Random tool names (1-20 chars, alphanumeric)
- Random descriptions (0-200 chars)
- Random input schemas (JSON Schema objects with 0-10 properties)
- Random annotations (key-value pairs)

```rust
fn arb_tool_set(size: Range<usize>) -> impl Strategy<Value = Vec<ToolDef>> {
    prop::collection::vec(arb_tool_def(), size)
}
```

### Strategy: `arb_tool_mutation`

Generates exactly one mutation type:

```rust
#[derive(Debug)]
enum ToolMutation {
    ChangeName { tool_idx: usize, new_name: String },
    ChangeDescription { tool_idx: usize, new_desc: String },
    ChangeInputSchema { tool_idx: usize, new_schema: JsonSchema },
    ChangeAnnotation { tool_idx: usize, key: String, new_value: String },
    AddTool { tool: ToolDef },
    RemoveTool { tool_idx: usize },
}
```

### Coverage Matrix

| Mutation Type | Drift Finding Expected |
|--------------|----------------------|
| Name change | Yes — tool renamed |
| Description change | Yes — description altered |
| Schema change | Yes — input schema modified |
| Annotation change | Yes — annotation modified |
| Tool added | Yes — new tool appeared |
| Tool removed | Yes — tool disappeared |

## Feasibility Assessment

| Factor | Assessment |
|--------|-----------|
| Input space | Bounded tool sets (1..10 tools) with structured mutations |
| Complexity | Medium — requires custom strategies for tool definitions |
| Tool support | Excellent — proptest handles custom enum strategies well |
| Time | Milliseconds per case |
| Verdict | **FEASIBLE** |

## Lifecycle

| Field | Value |
|-------|-------|
| Introduced | v0.1.0 |
| Modified | — |
| Deprecated | — |
| Retired | — |
