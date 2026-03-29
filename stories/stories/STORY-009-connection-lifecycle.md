---
document_type: story
story_id: STORY-009
epic_id: EPIC-01
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 3
depends_on: [STORY-007, STORY-008]
blocks: [STORY-010]
behavioral_contracts: [BC-1.02.003]
verification_properties: [VP-013]
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-009: Connection Lifecycle Management (Keepalive & Shutdown)

## Narrative
- **As an** AI Platform Engineer using the TUI or CLI
- **I want to** have connections stay alive during active use and shut down cleanly when I'm done
- **So that** I don't experience dropped connections or zombie processes

## Acceptance Criteria

### AC-001 (traces to BC-1.02.003 postcondition — keepalive)
When a connection is idle for more than 60s (configurable), it is kept alive via a protocol-level ping or by periodic lightweight requests. Connection state remains `Connected`.
- **Test:** `test_BC_1_02_003_keepalive_maintains_connection()`

### AC-002 (traces to BC-1.02.003 postcondition — graceful shutdown)
`connection.close()` sends a JSON-RPC shutdown notification to the server before closing the transport. For stdio, the subprocess is allowed 5s to exit gracefully before `SIGKILL`.
- **Test:** `test_BC_1_02_003_graceful_shutdown()`

### AC-003 (traces to BC-1.02.003 postcondition — connection state transitions)
Connection state machine follows the valid transitions: `Connecting → Connected`, `Connected → Disconnecting → Disconnected`, `Connected → Error(reason)`, `Disconnected → Connecting` (reconnect). No invalid transitions are possible.
- **Test:** `test_BC_1_02_003_state_machine_valid_transitions()` (Kani proof for VP-013)

### AC-004 (traces to BC-1.02.003 — error state recovery)
A connection in `Error` state can be transitioned back to `Connecting` via `connection.reconnect()`. The previous error is cleared.
- **Test:** `test_BC_1_02_003_reconnect_from_error()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `McpConnection.close()` | `forge-core/src/connection.rs` | Effectful |
| `McpConnection.reconnect()` | `forge-core/src/connection.rs` | Effectful |
| State machine transitions | `forge-core/src/connection.rs` | Pure core |

## UX Screens
- SCR-002 — badge shows Disconnecting state

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Server doesn't respond to shutdown | Force close after 5s |
| EC-002 | Close called on already-disconnected connection | No-op, no error |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| State machine | Pure core | Transition table, no I/O |
| close(), reconnect() | Effectful | Network/process I/O |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~700 |
| BC-1.02.003 | ~500 |
| VP-013 proof sketch | ~400 |
| **Total** | **~1,600** |
| Agent context window | 200K |
| **Budget usage** | **~0.8%** |

## Tasks

1. [ ] Write failing tests for all 4 ACs
2. [ ] Implement keepalive mechanism in McpConnection
3. [ ] Implement graceful shutdown with timeout
4. [ ] Implement reconnect() method
5. [ ] Write Kani proof harness for VP-013 state machine
6. [ ] Verify Red Gate
7. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-007 | McpConnection and state machine skeleton | Continue extending | Stdio needs SIGKILL fallback; HTTP just drops session |
| STORY-008 | HTTP session loss triggers re-init | Reuse reconnect() pattern | |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| State machine pure core | purity-boundary-map.md | No tokio calls in state transition logic |
| rmcp for shutdown notification | AD-002 | Use rmcp shutdown, not custom message |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| tokio | >= 1.38 | Async timers for keepalive | `tokio::time::interval` |
| kani | dev | VP-013 proof | `#[kani::proof]` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-core/src/connection.rs` | Add close(), reconnect(), keepalive | YES (from STORY-007) |
| `crates/forge-core/proofs/connection_state.rs` | Kani proof for VP-013 | NO — this story creates it |
