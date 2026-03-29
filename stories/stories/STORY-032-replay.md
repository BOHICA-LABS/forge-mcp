---
document_type: story
story_id: STORY-032
epic_id: EPIC-04
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-029]
blocks: []
behavioral_contracts: [BC-4.10.003]
verification_properties: [VP-006]
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-032: Message Sequence Replay Against Target Server

## Narrative
- **As an** AI Platform Engineer
- **I want to** replay a captured message sequence against a server
- **So that** I can reproduce specific scenarios for debugging or comparison

## Acceptance Criteria

### AC-001 (traces to BC-4.10.003 postcondition — replay sends messages)
`replay_sequence(connection, messages)` sends each message in the captured sequence (filtered to client→server direction only) to the target server and returns the corresponding responses.
- **Test:** `test_BC_4_10_003_replay_sends_messages()`

### AC-002 (traces to BC-4.10.003 — server not connected error)
If the target server is not connected, returns `Err(E-CAP-002: replay target server not connected)`.
- **Test:** `test_BC_4_10_003_disconnected_target_errors()`

### AC-003 (traces to BC-4.10.003 — response capture)
Responses from the replayed sequence are captured as new `MessageCaptured` events with `replay: true` flag, distinguishable from live traffic.
- **Test:** `test_BC_4_10_003_replay_responses_captured()`

### AC-004 (traces to BC-4.10.003 — DI-007 ordering)
Messages are replayed in temporal order (same sequence as original capture). No reordering.
- **Test:** `test_BC_4_10_003_replay_preserves_order()`

### AC-005 (traces to BC-4.10.003 — CLI replay)
`forge-mcp call <server> --replay <capture-file>` replays a JSON capture file against the server.
- **Test:** `test_BC_4_10_003_cli_replay_command()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `replay_sequence()` | `forge-traffic/src/replay.rs` | Effectful (sends RPC) |
| Replay-tagged events | `forge-core/src/events.rs` | Pure (add flag) |

## UX Screens
- SCR-004 (Traffic Inspector) — `R` key triggers replay of selected message

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Replay server-initiated messages | Skipped (only client→server replayed) |
| EC-002 | Response to replayed request times out | Err, continue to next message |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `replay_sequence()` | Effectful | Sends real RPC messages |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~700 |
| BC-4.10.003 | ~400 |
| **Total** | **~1,100** |
| Agent context window | 200K |
| **Budget usage** | **~0.6%** |

## Tasks

1. [ ] Write failing tests for all 5 ACs
2. [ ] Implement `replay_sequence()` (direction filter + sequential send)
3. [ ] Add `replay: bool` flag to `MessageCaptured`
4. [ ] Implement CLI replay command
5. [ ] Verify Red Gate
6. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-030 | Direction filter available | Filter to ClientToServer before replay | Notifications can't be replayed |
| STORY-027 | MessageCaptured has direction | Use direction field | |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Replay via forge-core transport (DI-007) | module-decomposition.md | Use connection.call_tool not raw sockets |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| (no new deps) | | | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-traffic/src/replay.rs` | replay_sequence() | NO — this story creates it |
