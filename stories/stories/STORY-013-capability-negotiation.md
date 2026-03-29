---
document_type: story
story_id: STORY-013
epic_id: EPIC-02
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-007, STORY-008]
blocks: [STORY-014, STORY-015, STORY-016, STORY-017, STORY-018, STORY-019, STORY-020, STORY-021, STORY-022, STORY-023, STORY-027, STORY-037, STORY-055, STORY-062]
behavioral_contracts: [BC-2.04.001, BC-2.04.002]
verification_properties: [VP-001, VP-002, VP-003, VP-013]
priority: P0
assumption_validations: []
risk_mitigations: [R-002]
---

# STORY-013: Bidirectional Capability Negotiation

## Narrative
- **As an** AI Platform Engineer
- **I want to** have Forge MCP perform correct MCP capability negotiation on connection
- **So that** both sides know what features are available and the tool works with any compliant MCP server

## Acceptance Criteria

### AC-001 (traces to BC-2.04.001 postcondition — negotiation completes)
`connection.initialize()` sends `initialize` request via rmcp with Forge MCP client info and receives a server `InitializeResult`. The negotiated server capabilities are stored on the `McpConnection` and accessible via `connection.server_capabilities()`.
- **Test:** `test_BC_2_04_001_negotiation_completes()`

### AC-002 (traces to BC-2.04.001 postcondition — capability set stored)
After `initialize`, `connection.server_capabilities()` returns the server's advertised capabilities: tools, resources, prompts, logging, sampling, elicitation, roots. Absent fields default to None/empty.
- **Test:** `test_BC_2_04_001_capability_set_stored()`

### AC-003 (traces to BC-2.04.001 — guard on uncapable methods)
Calling a method on a server that didn't advertise the corresponding capability returns `Err(E-PRO-003: method <m> not available — server lacks <cap> capability)` without sending the request.
- **Test:** `test_BC_2_04_001_guards_uncapable_methods()`

### AC-004 (traces to BC-2.04.002 postcondition — client capability advertisement)
The `initialize` request includes Forge MCP's client capabilities: `sampling: {}` (handled), `elicitation: {}` (handled), `roots: { listChanged: true }`. (Traces to BC-2.04.002.)
- **Test:** `test_BC_2_04_002_client_capabilities_advertised()`

### AC-005 (traces to BC-2.04.001 invariant — rmcp used exclusively)
Capability negotiation is performed via `rmcp::ClientCapabilitiesBuilder`. No custom `initialize` JSON construction. (AD-002, DI-004, NFR-014.)
- **Test:** Code review (no hand-rolled initialize JSON)

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `connection.initialize()` | `forge-core/src/connection.rs` | Effectful (RPC) |
| `ServerCapabilities` | `forge-core/src/types.rs` | Pure |
| Capability guard | `forge-core/src/connection.rs` | Pure check, effectful invocation |

## UX Screens
- SCR-003 (Capability Browser) — populated from server_capabilities()

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Server advertises empty capabilities | All guards active; only ping works |
| EC-002 | initialize fails (E-CON-005) | Connection enters Error state |
| EC-003 | Server sends extra unknown capability fields | Ignored (forward-compatible) |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Capability guard | Pure | Checks capabilities map, no I/O |
| initialize() | Effectful | rmcp RPC call |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~1,000 |
| BC-2.04.001, BC-2.04.002 | ~1,000 |
| rmcp ClientCapabilitiesBuilder API | ~400 |
| **Total** | **~2,400** |
| Agent context window | 200K |
| **Budget usage** | **~1.2%** |

## Tasks

1. [ ] Write failing tests for all 5 ACs (uses forge-test-server STORY-002)
2. [ ] Implement `initialize()` on McpConnection using rmcp
3. [ ] Define `ServerCapabilities` type in forge-core
4. [ ] Implement capability guard logic
5. [ ] Build client capabilities via rmcp ClientCapabilitiesBuilder
6. [ ] Add fuzz target for JSON-RPC parse (VP-001)
7. [ ] Verify Red Gate
8. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-007 | McpConnection is the connection handle | All protocol operations go through it | Must call initialize() before any other operation |
| STORY-008 | HTTP sessions need session ID on every request | Same applies after initialize | |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| rmcp for all protocol operations (AD-002) | ARCH-INDEX.md | Use rmcp::ClientCapabilitiesBuilder |
| forge-core L0 — no internal deps | dependency-graph.md | No imports from L1+ |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| rmcp | workspace | ClientCapabilitiesBuilder | `rmcp::client::ClientCapabilitiesBuilder` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-core/src/connection.rs` | initialize() + capability guard | YES (from STORY-007) |
| `crates/forge-core/src/types.rs` | ServerCapabilities added | YES (from STORY-005) |
| `fuzz/fuzz_targets/jsonrpc_parse.rs` | Fuzz for VP-001 | NO — this story creates it |
