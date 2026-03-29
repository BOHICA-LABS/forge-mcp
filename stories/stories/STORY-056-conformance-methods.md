---
document_type: story
story_id: STORY-056
epic_id: EPIC-08
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-055]
blocks: [STORY-058]
behavioral_contracts: [BC-8.19.002]
verification_properties: []
priority: P1
assumption_validations: []
risk_mitigations: []
---

# STORY-056: Conformance Method Coverage & Error Handling

## Narrative
- **As an** MCP Server Author
- **I want to** verify that my server handles all required MCP methods correctly
- **So that** my server passes automated conformance in CI/CD pipelines

## Acceptance Criteria

### AC-001 (traces to BC-8.19.002 postcondition — method coverage)
Conformance suite exercises ≥ 90% of MCP 2025-11-25 methods (~25 total). Each method invocation is recorded as Pass/Fail. Summary shows method coverage percentage. (NFR-010.)
- **Test:** `test_BC_8_19_002_method_coverage_90pct()`

### AC-002 (traces to BC-8.19.002 postcondition — error handling conformance)
For each method, suite tests that the server returns correct JSON-RPC error codes for invalid requests: -32600 (invalid request), -32601 (method not found), -32602 (invalid params), -32603 (internal error).
- **Test:** `test_BC_8_19_002_error_code_conformance()`

### AC-003 (traces to BC-8.19.002 — advertised capability enforcement)
If server advertises tools capability but returns -32601 for `tools/list`, that is a conformance failure (capability/method mismatch).
- **Test:** `test_BC_8_19_002_capability_method_mismatch()`

### AC-004 (traces to BC-8.19.002 — skip unavailable capabilities)
Methods for capabilities NOT advertised by the server are marked `status: Skip` (not Fail).
- **Test:** `test_BC_8_19_002_skip_unadvertised_methods()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| Method coverage tracker | `forge-conformance/src/coverage.rs` | Pure |
| Method test cases | `forge-conformance/src/methods.rs` | Effectful |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Server advertises unknown capability | Tests for that capability are Skipped |
| EC-002 | Method times out instead of erroring | Fail with timeout message |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Coverage tracker | Pure | Count and percentage math |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~700 |
| BC-8.19.002 | ~500 |
| NFR-010 | ~200 |
| **Total** | **~1,400** |
| Agent context window | 200K |
| **Budget usage** | **~0.7%** |

## Tasks

1. [ ] Write failing tests for all 4 ACs
2. [ ] Build complete MCP 2025-11-25 method inventory (~25 methods)
3. [ ] Implement test case per method
4. [ ] Implement error code validation
5. [ ] Implement coverage percentage calculation
6. [ ] Verify Red Gate
7. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-055 | ConformanceResult type established | Add method-coverage fields | 90% of ~25 = need ≥ 23 methods tested |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| ≥90% method coverage (NFR-010) | nfr-catalog.md | Assert in CI |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| (no new deps) | | | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-conformance/src/methods.rs` | Method test cases | NO — this story creates it |
| `crates/forge-conformance/src/coverage.rs` | Coverage tracker | NO |
