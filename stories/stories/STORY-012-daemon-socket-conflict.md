---
document_type: story
story_id: STORY-012
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
behavioral_contracts: [BC-1.03.003]
verification_properties: []
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-012: Daemon Socket Conflict Detection & Recovery

## Narrative
- **As a** developer or AI agent running multiple Forge MCP instances
- **I want to** have socket conflicts detected and handled gracefully
- **So that** I don't end up with multiple daemons fighting over the same socket or stale socket files

## Acceptance Criteria

### AC-001 (traces to BC-1.03.003 postcondition — stale socket detection)
On startup, if a socket file exists but no daemon is listening, `forge-daemon start` detects the stale socket, removes it, and starts fresh. Emits log: "Removed stale daemon socket at <path>".
- **Test:** `test_BC_1_03_003_stale_socket_removed()`

### AC-002 (traces to BC-1.03.003 postcondition — live conflict detection)
If a socket exists and a daemon is actively listening, `forge-daemon start` detects the conflict, logs "Daemon already running at <path>", and exits with code 0 (success — existing daemon is healthy).
- **Test:** `test_BC_1_03_003_live_daemon_detected()`

### AC-003 (traces to BC-1.03.003 — forced restart)
`forge-daemon restart` sends a shutdown signal to the existing daemon, waits up to 5s for it to exit, then starts a new daemon.
- **Test:** `test_BC_1_03_003_daemon_restart()`

### AC-004 (traces to BC-1.03.003 — socket path configuration)
The daemon socket path is configurable via `FORGE_DAEMON_SOCKET` env var. Default is OS-appropriate: `$XDG_RUNTIME_DIR/forge-mcp.sock` on Linux, `$TMPDIR/forge-mcp.sock` on macOS, `\\.\pipe\forge-mcp` on Windows.
- **Test:** `test_BC_1_03_003_socket_path_configuration()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `daemon_socket_path()` | `forge-daemon/src/socket.rs` | Pure |
| Socket conflict detection | `forge-daemon/src/daemon.rs` | Effectful |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Socket file is directory | Log error, abort start |
| EC-002 | Permission denied to remove stale socket | Log error, abort with useful message |
| EC-003 | FORGE_DAEMON_SOCKET set to non-existent dir | Create parent dir or error |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `daemon_socket_path()` | Pure | Returns path from env/constants |
| Conflict detection | Effectful | Socket file system access |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~700 |
| BC-1.03.003 | ~400 |
| **Total** | **~1,100** |
| Agent context window | 200K |
| **Budget usage** | **~0.6%** |

## Tasks

1. [ ] Write failing tests for all 4 ACs
2. [ ] Implement socket path resolution (pure, OS-aware)
3. [ ] Implement stale socket detection and removal
4. [ ] Implement live daemon detection via connect probe
5. [ ] Implement daemon restart command
6. [ ] Verify Red Gate
7. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-010 | DaemonServer has socket listener | Extend to add conflict detection on start | Windows named pipes need different conflict check |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Socket path is configurable | BC-1.03.003 | FORGE_DAEMON_SOCKET env var |
| Pure path resolution | purity-boundary-map.md | `daemon_socket_path()` pure |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| tokio | >= 1.38 | Async socket connect probe | `tokio::net::UnixStream::connect` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-daemon/src/socket.rs` | Socket path resolution | NO — this story creates it |
| `crates/forge-daemon/src/daemon.rs` | Conflict detection added | YES (from STORY-010) |
