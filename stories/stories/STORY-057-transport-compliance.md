---
document_type: story
story_id: STORY-057
epic_id: EPIC-08
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 3
depends_on: [STORY-055, STORY-003]
blocks: [STORY-058]
behavioral_contracts: [BC-8.19.003]
verification_properties: []
priority: P1
assumption_validations: []
risk_mitigations: []
---

# STORY-057: Conformance Transport Compliance

## Narrative
- **As an** MCP Server Author
- **I want to** validate that my server correctly implements the transport layer
- **So that** clients with different transport expectations can connect reliably

## Acceptance Criteria

### AC-001 (traces to BC-8.19.003 postcondition — stdio transport compliance)
For stdio servers: verifies newline-delimited JSON-RPC framing, correct Content-Length semantics (if used), and proper process lifecycle.
- **Test:** `test_BC_8_19_003_stdio_transport_compliance()`

### AC-002 (traces to BC-8.19.003 postcondition — HTTP transport compliance)
For HTTP servers: verifies `Mcp-Session-Id` header presence, correct HTTP status codes per spec, SSE streaming format.
- **Test:** `test_BC_8_19_003_http_transport_compliance()`

### AC-003 (traces to BC-8.19.003 — transport mismatch detection)
If a server registered as stdio actually behaves like HTTP (or vice versa), reports a conformance failure `transport_mismatch`.
- **Test:** `test_BC_8_19_003_transport_mismatch_detected()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| Transport compliance tests | `forge-conformance/src/transport.rs` | Effectful |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Server sends HTTP despite stdio config | Transport mismatch failure |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Transport tests | Effectful | Real connection + I/O |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~500 |
| BC-8.19.003 | ~300 |
| **Total** | **~800** |
| Agent context window | 200K |
| **Budget usage** | **~0.4%** |

## Tasks

1. [ ] Write failing tests for all 3 ACs
2. [ ] Implement stdio transport compliance checks
3. [ ] Implement HTTP transport compliance checks (uses forge-test-http-server)
4. [ ] Verify Red Gate
5. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-055 | ConformanceResult type | Use same type | HTTP compliance needs forge-test-http-server |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| (no new rules) | | |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| (no new deps) | | | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-conformance/src/transport.rs` | Transport compliance tests | NO — this story creates it |
