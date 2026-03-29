---
document_type: story
story_id: STORY-011
epic_id: EPIC-01
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 3
depends_on: [STORY-010]
blocks: []
behavioral_contracts: [BC-1.03.002]
verification_properties: [VP-015]
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-011: Named Session Persistence Across CLI Invocations

## Narrative
- **As a** CLI user or AI agent
- **I want to** name a session and resume it across separate CLI invocations
- **So that** I can maintain conversation context without reconnecting on every command

## Acceptance Criteria

### AC-001 (traces to BC-1.03.002 postcondition — named session stored)
`forge-mcp connect --session <name> <server>` creates a named session in the daemon. The session ID and connection handle are stored keyed by the provided name.
- **Test:** `test_BC_1_03_002_named_session_creation()`

### AC-002 (traces to BC-1.03.002 postcondition — session resume)
A subsequent `forge-mcp call --session <name> <server> <tool>` command reuses the named session without re-establishing connection. The request is routed to the existing `McpConnection`.
- **Test:** `test_BC_1_03_002_named_session_resume()`

### AC-003 (traces to BC-1.03.002 — session not found)
If `--session <name>` references a non-existent session, returns `Err(E-CON-003: session '<name>' not found)` with exit code 2.
- **Test:** `test_BC_1_03_002_session_not_found()`

### AC-004 (traces to BC-1.03.002 — session list)
`forge-mcp daemon sessions` lists all active named sessions with: session name, server name, connection state, created time, last-used time. Output is JSON.
- **Test:** `test_BC_1_03_002_session_list_json()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| Named session registry | `forge-daemon/src/pool.rs` | Effectful |
| Session list command | `forge-mcp/src/commands/daemon.rs` | Effectful |

## UX Screens
- N/A (CLI-only story)

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Session name with spaces | Allowed, quoted in CLI |
| EC-002 | Two sessions with same name | Second overwrites first with warning |
| EC-003 | Session connection drops while named | Session entry retained, state = Error |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Session registry | Effectful | Persistent state across invocations |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~700 |
| BC-1.03.002 | ~400 |
| SessionPool (STORY-010) | ~300 |
| **Total** | **~1,400** |
| Agent context window | 200K |
| **Budget usage** | **~0.7%** |

## Tasks

1. [ ] Write failing tests for all 4 ACs
2. [ ] Extend SessionPool to support named sessions
3. [ ] Add `--session` flag to connect and call subcommands
4. [ ] Implement session list endpoint in daemon
5. [ ] Verify Red Gate
6. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-010 | SessionPool keyed by server name | Extend to support user-provided name key | Named sessions are an overlay on the pool |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Session IDs unique (DI-003) | BC-1.03.002 | UUIDs |
| No session name conflicts without warning | BC-1.03.002 | Log E-CON warning |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| (same as STORY-010) | | | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-daemon/src/pool.rs` | Named session support added | YES (from STORY-010) |
| `crates/forge-mcp/src/commands/daemon.rs` | daemon sessions subcommand | NO — this story creates it |
