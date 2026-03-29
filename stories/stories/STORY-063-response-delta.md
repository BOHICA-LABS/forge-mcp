---
document_type: story
story_id: STORY-063
epic_id: EPIC-10
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-016]
blocks: []
behavioral_contracts: [BC-10.25.001]
verification_properties: []
priority: P2
assumption_validations: []
risk_mitigations: []
---

# STORY-063: Response Behavior Delta Testing

## Narrative
- **As an** MCP Server Author
- **I want to** run the same tool calls against two server instances and compare responses
- **So that** I can verify behavioral equivalence between versions

## Acceptance Criteria

### AC-001 (traces to BC-10.25.001 postcondition — parallel execution)
`compare_responses(conn_a, conn_b, tool_calls) -> ResponseDelta` sends the same tool calls to both servers and records responses. Runs sequentially (one pair at a time) to avoid timing artifacts.
- **Test:** `test_BC_10_25_001_parallel_response_comparison()`

### AC-002 (traces to BC-10.25.001 postcondition — response diff)
For each tool call, compares: `isError` (must be same), `content` structure (semantic JSON diff), response time (A faster/slower/equal). Summary: `equivalent: bool`.
- **Test:** `test_BC_10_25_001_response_diff()`

### AC-003 (traces to BC-10.25.001 — CLI command)
`forge-mcp diff <server-a> <server-b> --run-tests` runs response delta testing using a configurable test script (JSON file with tool calls).
- **Test:** `test_BC_10_25_001_cli_run_tests()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `compare_responses()` | `forge-config/src/response_delta.rs` | Effectful (sends RPC) |
| Response diff | `forge-config/src/response_delta.rs` | Pure |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Server A errors, B succeeds | Not equivalent — reported |
| EC-002 | Timing differences only | Noted but not equivalence failure |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Response diff | Pure | JSON comparison |
| compare_responses() | Effectful | RPC calls |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~700 |
| BC-10.25.001 | ~400 |
| **Total** | **~1,100** |
| Agent context window | 200K |
| **Budget usage** | **~0.6%** |

## Tasks

1. [ ] Write failing tests for all 3 ACs
2. [ ] Implement `compare_responses()` with sequential tool execution
3. [ ] Implement JSON response diff (semantic)
4. [ ] Add --run-tests flag to CLI diff command
5. [ ] Verify Red Gate
6. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-061 | CLI diff command | --run-tests extends it | forge-config L2: depends on forge-core but not forge-traffic |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| forge-config L2 depends on forge-core | dependency-graph.md | Uses forge-core for RPC |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| json-patch | >= 0.3 | Response JSON diff | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-config/src/response_delta.rs` | compare_responses() | NO — this story creates it |
