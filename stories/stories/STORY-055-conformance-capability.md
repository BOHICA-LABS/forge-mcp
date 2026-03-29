---
document_type: story
story_id: STORY-055
epic_id: EPIC-08
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-013, STORY-002]
blocks: [STORY-056, STORY-057, STORY-058]
behavioral_contracts: [BC-8.19.001]
verification_properties: []
priority: P1
assumption_validations: []
risk_mitigations: []
---

# STORY-055: Conformance Capability Negotiation Validation

## Narrative
- **As an** MCP Server Author
- **I want to** validate that my server correctly implements capability negotiation
- **So that** I can be confident my server works with all compliant MCP clients

## Acceptance Criteria

### AC-001 (traces to BC-8.19.001 postcondition — negotiation test suite)
`forge-mcp test <server>` runs the capability negotiation conformance suite: T-CAP-001 (server advertises ≥1 capability), T-CAP-002 (capabilities persist across method calls), T-CAP-003 (graceful rejection of unknown capabilities).
- **Test:** `test_BC_8_19_001_negotiation_suite()`

### AC-002 (traces to BC-8.19.001 postcondition — test result structure)
Each conformance test produces `ConformanceResult { test_id, description, status: Pass|Fail|Skip, error_detail: Option<String> }`.
- **Test:** `test_BC_8_19_001_result_structure()`

### AC-003 (traces to BC-8.19.001 — incomplete server handling)
When a server fails some negotiation tests but responds to others, individual test results show Pass/Fail granularly. Not all-or-nothing.
- **Test:** `test_BC_8_19_001_incomplete_server_granular()`

### AC-004 (traces to BC-8.19.001 — spec version conformance)
Suite tests both `2024-11-05` and `2025-11-25` spec version behavior. Tests are tagged with applicable spec version.
- **Test:** `test_BC_8_19_001_dual_spec_version()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| Conformance test suite | `forge-conformance/src/suite.rs` | Effectful (runs tests) |
| Test assertions | `forge-conformance/src/assertions.rs` | Pure |
| `ConformanceResult` | `forge-conformance/src/types.rs` | Pure |

## UX Screens
- SCR-008 (Conformance Test Runner)

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Server crashes during test | Test status = Fail with crash detail |
| EC-002 | Server times out on capability negotiation | Test status = Fail with timeout detail |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Assertions | Pure | Expected vs actual behavior check |
| Test suite runner | Effectful | Sends real protocol messages |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~800 |
| BC-8.19.001 | ~500 |
| **Total** | **~1,300** |
| Agent context window | 200K |
| **Budget usage** | **~0.7%** |

## Tasks

1. [ ] Write failing tests (uses forge-test-server STORY-002)
2. [ ] Define `ConformanceResult` type
3. [ ] Implement capability negotiation test cases
4. [ ] Implement dual spec version testing
5. [ ] Verify Red Gate
6. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-013 | Capability negotiation established | Conformance tests re-run initialization | Test must create fresh connection per test |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| forge-conformance L2 (depends only on forge-core) | dependency-graph.md | No forge-traffic imports |
| Assertions pure, runner effectful | purity-boundary-map.md | |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| (no new deps) | | | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-conformance/src/suite.rs` | Test runner | NO — this story creates it |
| `crates/forge-conformance/src/assertions.rs` | Pure assertions | NO |
| `crates/forge-conformance/src/types.rs` | ConformanceResult | NO |
