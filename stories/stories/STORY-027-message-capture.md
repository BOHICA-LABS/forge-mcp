---
document_type: story
story_id: STORY-027
epic_id: EPIC-04
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-013]
blocks: [STORY-028, STORY-029, STORY-030, STORY-031, STORY-032, STORY-033, STORY-034, STORY-047, STORY-048, STORY-049]
behavioral_contracts: [BC-4.09.001]
verification_properties: [VP-005, VP-006]
priority: P0
assumption_validations: []
risk_mitigations: [R-010]
---

# STORY-027: Transparent JSON-RPC Message Capture

## Narrative
- **As an** AI Platform Engineer using the traffic inspector
- **I want to** have all MCP messages captured transparently as they flow
- **So that** I can see exactly what the server receives and sends

## Acceptance Criteria

### AC-001 (traces to BC-4.09.001 postcondition — transparent capture)
Every JSON-RPC message passing through `forge-core` (both directions: client→server and server→client) emits a `MessageCaptured` event containing: `id` (UUID), `direction` (ClientToServer | ServerToClient), `method: Option<String>`, `payload: serde_json::Value`, `timestamp: Instant`.
- **Test:** `test_BC_4_09_001_all_messages_captured()`

### AC-002 (traces to BC-4.09.001 postcondition — content integrity)
The captured payload is byte-identical to the wire payload. No reformatting, compression, or field omission. (VP-006: message capture preserves content.)
- **Test:** `test_BC_4_09_001_payload_content_integrity()` (proptest with arbitrary messages)

### AC-003 (traces to BC-4.09.001 — event bus)
`MessageCaptured` events are broadcast on a Tokio broadcast channel. Multiple consumers (forge-traffic, forge-health, forge-security, forge-tui) can subscribe independently.
- **Test:** `test_BC_4_09_001_multiple_consumers()`

### AC-004 (traces to BC-4.09.001 — server performance impact)
Capture adds < 1% additional latency to server operations (NFR-004). Capture is passive observation only — no synthetic probes.
- **Test:** Benchmark test measuring overhead

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `MessageCaptured` event | `forge-core/src/events.rs` | Pure (data type) |
| Message intercept hook | `forge-core/src/connection.rs` | Effectful |
| Broadcast channel | `forge-core/src/events.rs` | Effectful |

## UX Screens
- SCR-004 (Traffic Inspector)
- FLOW-003 (Traffic Inspection)

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | No consumers subscribed | Events broadcast to 0 receivers; no error, no backpressure |
| EC-002 | Very large payload (>1MB) | Captured as-is, no truncation |
| EC-003 | Capture channel full | Oldest event dropped (lagged receiver) |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `MessageCaptured` struct | Pure | Data type only |
| Broadcast channel | Effectful | Async channel state |
| Message intercept hook | Effectful | Intercept in rmcp I/O path |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~900 |
| BC-4.09.001 | ~600 |
| forge-core events.rs (STORY-022) | ~300 |
| **Total** | **~1,800** |
| Agent context window | 200K |
| **Budget usage** | **~0.9%** |

## Tasks

1. [ ] Write failing tests for all 4 ACs
2. [ ] Extend `MessageCaptured` event type in forge-core events.rs
3. [ ] Add message intercept hook in connection.rs (both directions)
4. [ ] Set up broadcast channel for capture events
5. [ ] Add proptest for VP-006 (content integrity)
6. [ ] Benchmark overhead (NFR-004)
7. [ ] Verify Red Gate
8. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-022 | events.rs created for progress | Extend same events.rs | broadcast channel has lagged receiver semantics |
| STORY-013 | connection.rs is the rmcp wrapper | Hook into protocol calls there | Must capture ALL messages, not just tool calls |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| AD-004: event-driven architecture | ARCH-INDEX.md | Use broadcast channel not direct coupling |
| Passive observation only (NFR-004) | nfr-catalog.md | No synthetic probe messages |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| tokio | >= 1.38 | Broadcast channel | `tokio::sync::broadcast` |
| proptest | dev | VP-006 content integrity | `proptest::proptest!` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-core/src/events.rs` | MessageCaptured event + broadcast | YES (from STORY-022) |
| `crates/forge-core/src/connection.rs` | Message intercept hook | YES — extended here |
