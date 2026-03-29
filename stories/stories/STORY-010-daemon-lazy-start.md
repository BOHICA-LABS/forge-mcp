---
document_type: story
story_id: STORY-010
epic_id: EPIC-01
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-007, STORY-008, STORY-009]
blocks: [STORY-011, STORY-012]
behavioral_contracts: [BC-1.03.001]
verification_properties: [VP-015]
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-010: Daemon Lazy Start & Session Pooling

## Narrative
- **As an** AI agent or CLI user
- **I want to** have connections reused automatically across multiple CLI invocations
- **So that** I avoid reconnection overhead and achieve < 50ms cold start for subsequent commands

## Acceptance Criteria

### AC-001 (traces to BC-1.03.001 postcondition — daemon starts on demand)
When the first CLI command is invoked and no daemon is running, `forge-mcp daemon` is auto-started in the background. The CLI waits up to 3s for the daemon socket to become available, then proceeds.
- **Test:** `test_BC_1_03_001_daemon_lazy_start()`

### AC-002 (traces to BC-1.03.001 postcondition — session pool)
The daemon maintains a pool of `McpConnection` objects keyed by server name. When a CLI command requests a server, the daemon returns the existing connection if alive, or establishes a new one.
- **Test:** `test_BC_1_03_001_session_pool_reuse()`

### AC-003 (traces to BC-1.03.001 — idle timeout)
Pooled connections idle for more than 5 minutes (configurable via `--idle-timeout`) are closed and removed from the pool.
- **Test:** `test_BC_1_03_001_idle_timeout_closes_connection()`

### AC-004 (traces to BC-1.03.001 — concurrent requests)
Multiple CLI invocations simultaneously requesting the same server receive the same pooled connection (multiplexed). No two invocations establish separate connections to the same server. (Relates to VP-015.)
- **Test:** `test_BC_1_03_001_concurrent_requests_multiplexed()`

### AC-005 (traces to BC-1.03.001 — NFR-001 cold start)
After the daemon is running with a pooled connection, `forge-mcp list <server>` executes in < 50ms wall time (measured by `hyperfine`). (Traces to NFR-001.)
- **Test:** Benchmark test with hyperfine in CI

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `DaemonServer` | `forge-daemon/src/daemon.rs` | Effectful (socket) |
| `SessionPool` | `forge-daemon/src/pool.rs` | Effectful (connection mgmt) |
| Session ID generation | `forge-daemon/src/session.rs` | Pure (DI-003) |

## UX Screens
- Status bar shows "[Daemon: OK]"

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Daemon fails to start within 3s | CLI falls back to direct connection |
| EC-002 | Server disconnects while in pool | Pool evicts entry; next request creates new connection |
| EC-003 | Maximum pool size reached | LRU eviction of oldest idle connection |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Session ID generation | Pure (DI-003) | UUID generation, no I/O |
| SessionPool | Effectful | Socket + connection management |
| DaemonServer | Effectful | Unix socket listener |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~1,000 |
| BC-1.03.001 | ~600 |
| McpConnection from STORY-007 | ~300 |
| **Total** | **~1,900** |
| Agent context window | 200K |
| **Budget usage** | **~1%** |

## Tasks

1. [ ] Write failing tests for all 5 ACs
2. [ ] Define daemon IPC protocol (Unix socket / Windows named pipe)
3. [ ] Implement `DaemonServer` with socket listener
4. [ ] Implement `SessionPool` with keyed connection map
5. [ ] Implement lazy start logic in CLI (detect daemon, spawn if missing)
6. [ ] Implement idle timeout with cleanup task
7. [ ] Add proptest for VP-015 session multiplexing
8. [ ] Verify NFR-001 with hyperfine benchmark
9. [ ] Verify Red Gate
10. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-007 | McpConnection is the connection handle | Pool stores McpConnection | Windows uses named pipes not Unix sockets |
| STORY-009 | reconnect() available | Use when pooled conn enters Error state | |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| forge-daemon is L3 (depends on forge-core + forge-discovery) | dependency-graph.md | No imports from L2 |
| DI-003: session IDs unique across concurrent sessions | purity-boundary-map.md | Pure UUID generation |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| tokio | >= 1.38 | Async socket + timers | `tokio::net::UnixListener` |
| uuid | >= 1.0 | Session ID generation | `Uuid::new_v4()` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-daemon/src/daemon.rs` | DaemonServer | NO — this story creates it |
| `crates/forge-daemon/src/pool.rs` | SessionPool | NO — this story creates it |
| `crates/forge-daemon/src/session.rs` | Session ID generation | NO |
| `crates/forge-daemon/tests/daemon_tests.rs` | Integration tests | NO |
