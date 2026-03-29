---
document_type: story
story_id: STORY-021
epic_id: EPIC-02
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-013]
blocks: []
behavioral_contracts: [BC-2.05.006, BC-2.05.007, BC-2.05.008]
verification_properties: [VP-001]
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-021: Roots, Logging, Completion & Protocol Utilities

## Narrative
- **As an** AI Platform Engineer
- **I want to** use logging level control, roots management, and completion/autocomplete
- **So that** I can debug server behavior and provide better UX when calling tools

## Acceptance Criteria

### AC-001 (traces to BC-2.05.006 postcondition — roots list)
`connection.list_roots()` returns the client's configured root paths. When roots change, `notifications/roots/list_changed` is sent to the server.
- **Test:** `test_BC_2_05_006_roots_list_and_change_notification()`

### AC-002 (traces to BC-2.05.007 postcondition — logging level)
`connection.set_log_level(level)` sends `logging/setLevel` request. Subsequent `notifications/message` log entries at or above the configured level are received and displayable.
- **Test:** `test_BC_2_05_007_logging_level_control()`

### AC-003 (traces to BC-2.05.008 postcondition — completion)
`connection.complete(reference, argument)` sends `completion/complete` and returns a `CompletionResult` with suggested completions for tool arguments and prompt variables.
- **Test:** `test_BC_2_05_008_completion_request()`

### AC-004 (traces to BC-2.05.007 — log display)
Log messages received from the server are routable to: TUI log panel (if active) or stderr (CLI mode).
- **Test:** `test_BC_2_05_007_log_message_display()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `list_roots()`, `set_log_level()`, `complete()` | `forge-core/src/protocol.rs` | Effectful |
| Log message routing | `forge-core/src/handler.rs` | Effectful |

## UX Screens
- N/A directly (log messages appear in traffic inspector)

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Server doesn't support logging | Err(E-PRO-003) capability guard |
| EC-002 | Completion with no suggestions | Empty completions list, no error |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Protocol calls | Effectful | RPC |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~700 |
| BC-2.05.006–008 | ~600 |
| **Total** | **~1,300** |
| Agent context window | 200K |
| **Budget usage** | **~0.7%** |

## Tasks

1. [ ] Write failing tests for all 4 ACs
2. [ ] Implement `list_roots()` + roots changed notification
3. [ ] Implement `set_log_level()` + log message routing
4. [ ] Implement `complete()`
5. [ ] Verify Red Gate
6. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-016 | Capability guard established | Always check before API call | |
| STORY-014 | Roots ListChanged notification from ClientHandler | Integration with handler | |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Capability guard for logging capability | STORY-013 | |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| (no new deps) | | | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-core/src/protocol.rs` | list_roots(), set_log_level(), complete() | YES (from STORY-016) |
