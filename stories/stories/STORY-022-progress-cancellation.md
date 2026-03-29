---
document_type: story
story_id: STORY-022
epic_id: EPIC-02
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
behavioral_contracts: [BC-2.05.009, BC-2.05.010]
verification_properties: [VP-001, VP-002]
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-022: Progress Tracking, Cancellation & Error Distinction

## Narrative
- **As an** AI Platform Engineer
- **I want to** see progress on long-running tool calls and cancel them if needed
- **So that** I have visibility into slow operations and can interrupt them

## Acceptance Criteria

### AC-001 (traces to BC-2.05.009 postcondition — progress notification)
When a tool call receives `notifications/progress`, the progress token, current, and total values are emitted via an event channel. TUI can display a progress bar.
- **Test:** `test_BC_2_05_009_progress_notification_received()`

### AC-002 (traces to BC-2.05.009 postcondition — cancellation)
`connection.cancel_request(id)` sends `$/cancel` notification for the given request ID. Subsequent progress notifications for the cancelled request are silently ignored (E-PRO-006).
- **Test:** `test_BC_2_05_009_cancel_request()`

### AC-003 (traces to BC-2.05.009 — orphaned progress ignored)
Progress notifications for unknown or cancelled request IDs are silently dropped with `E-PRO-006` warning (degraded).
- **Test:** `test_BC_2_05_009_orphaned_progress_ignored()`

### AC-004 (traces to BC-2.05.010 postcondition — tool error vs protocol error)
`call_tool()` returning a result with `isError: true` is classified as a successful RPC call returning an error result (i.e., `Ok(ToolResult { isError: true })`). Only malformed JSON-RPC or transport failures return `Err(...)`. (VP-002 correctness.)
- **Test:** `test_BC_2_05_010_tool_error_is_ok_result()`

### AC-005 (traces to BC-2.05.010 — error display)
Tool errors (`isError: true`) are displayed in the TUI with a distinct error indicator (red `✗`) vs success results (green `✓`). Not as a panic or crash.
- **Test:** `test_BC_2_05_010_error_result_display_not_panic()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `cancel_request()` | `forge-core/src/protocol.rs` | Effectful |
| Progress event routing | `forge-core/src/events.rs` | Effectful |
| Error classification | `forge-core/src/error.rs` | Pure |

## UX Screens
- SCR-006 (Tool Execution Dialog) — shows progress bar and error results

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Cancel after completion | No-op, ignored |
| EC-002 | Progress total=0 | Display as indeterminate progress |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Error classification | Pure | isError flag check |
| cancel_request() | Effectful | Sends RPC notification |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~800 |
| BC-2.05.009, BC-2.05.010 | ~700 |
| **Total** | **~1,500** |
| Agent context window | 200K |
| **Budget usage** | **~0.8%** |

## Tasks

1. [ ] Write failing tests for all 5 ACs
2. [ ] Implement progress notification routing via event channel
3. [ ] Implement `cancel_request()` with orphan-guard
4. [ ] Extend error classification for tool error vs protocol error
5. [ ] Verify Red Gate
6. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-016 | call_tool established with isError distinction | Extend error module | Progress token can be any JSON value, not just int |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Tool error is Ok(), not Err() (DI-020) | purity-boundary-map.md | VP-002 test |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| tokio | >= 1.38 | Event channel | `tokio::sync::broadcast` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-core/src/protocol.rs` | cancel_request() added | YES (from STORY-016) |
| `crates/forge-core/src/events.rs` | Progress event bus | NO — this story creates it |
| `crates/forge-core/src/error.rs` | Error classification | YES (from STORY-016) |
