---
document_type: story
story_id: STORY-007
epic_id: EPIC-01
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-002, STORY-004]
blocks: [STORY-009, STORY-010, STORY-013]
behavioral_contracts: [BC-1.02.001]
verification_properties: [VP-013]
priority: P0
assumption_validations: []
risk_mitigations: [R-003]
---

# STORY-007: Stdio Transport Connection Establishment

## Narrative
- **As an** AI Platform Engineer
- **I want to** connect to an MCP server running as a subprocess via stdio
- **So that** I can inspect its tools, resources, and prompts

## Acceptance Criteria

### AC-001 (traces to BC-1.02.001 postcondition — connection established)
`connect_stdio(entry: &ServerEntry) -> Result<McpConnection>` spawns the server process from `StdioConfig.command`+`args`, establishes JSON-RPC framing over stdin/stdout via rmcp, and returns a live `McpConnection` handle.
- **Test:** `test_BC_1_02_001_stdio_connect_success()`

### AC-002 (traces to BC-1.02.001 — env var expansion)
Environment variables in `StdioConfig.env` are expanded at connection time using the process environment as the base. The expanded values are passed to the child process environment.
- **Test:** `test_BC_1_02_001_env_var_expansion()`

### AC-003 (traces to BC-1.02.001 — process exit detection)
When the server process exits unexpectedly, the connection transitions to `Disconnected` state and returns `Err(E-CON-002: server process exited with code <n>)`.
- **Test:** `test_BC_1_02_001_process_exit_detected()`

### AC-004 (traces to BC-1.02.001 — connection timeout)
If the server process does not respond to `initialize` within 30s (configurable), returns `Err(E-CON-003: timeout after <n>s)`.
- **Test:** `test_BC_1_02_001_connection_timeout()`

### AC-005 (traces to BC-1.02.001 — rmcp API used exclusively)
Connection uses only `rmcp` for transport framing. No custom JSON-RPC framing code. (Traces to AD-002, NFR-014.)
- **Test:** Code review assertion; no `serde_json::from_str` on raw stdin/stdout

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `connect_stdio()` | `forge-core/src/transport.rs` | Effectful (subprocess) |
| `McpConnection` | `forge-core/src/connection.rs` | Effectful (network state) |
| Connection state machine | `forge-core/src/connection.rs` | Pure core |

## UX Screens
- SCR-002 (Server Sidebar) — shows connection state
- FLOW-001 (Server Connection)

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Command not found in PATH | Err(E-CON-002 with "not found") |
| EC-002 | Process starts but sends invalid JSON-RPC | Err(E-PRO-001) |
| EC-003 | Very slow server initialization | Timeout fires at configured limit |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `connect_stdio()` | Effectful | Spawns process, I/O |
| Connection state machine | Pure core | State transitions w/o I/O |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~1,000 |
| BC-1.02.001 (referenced) | ~600 |
| rmcp transport API | ~400 |
| forge-test-server (STORY-002) | ~300 |
| **Total** | **~2,300** |
| Agent context window | 200K |
| **Budget usage** | **~1.2%** |

## Tasks

1. [ ] Write failing tests (uses forge-test-server from STORY-002)
2. [ ] Define `McpConnection` type in forge-core
3. [ ] Define connection state machine (Disconnected→Connecting→Connected→Error)
4. [ ] Implement `connect_stdio()` using rmcp stdio transport
5. [ ] Implement env var expansion at connection time
6. [ ] Implement process-exit detection
7. [ ] Implement connection timeout
8. [ ] Add Kani proof sketch for VP-013 connection state machine
9. [ ] Verify Red Gate
10. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-002 | forge-test-server spawns as subprocess | Use same spawn pattern | Subprocess stdout must be piped (not inherited) for rmcp |
| STORY-005 | StdioConfig has command, args, env | env not expanded until now | Shell variable syntax varies by OS |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| rmcp exclusive for transport (AD-002, DI-004) | ARCH-INDEX.md | No raw TCP/stdin parsing |
| McpConnection in forge-core (L0) | dependency-graph.md | Not in forge-daemon |
| Timeout configurable | BC-1.02.001 | Not hardcoded 30s |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| rmcp | workspace | Stdio transport | `rmcp::client::ClientBuilder` |
| tokio | >= 1.38 | Async subprocess | `tokio::process::Command` |
| tokio-util | >= 0.7 | I/O framing | codec utilities |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-core/src/transport.rs` | connect_stdio(), connect_http() | NO — this story creates it |
| `crates/forge-core/src/connection.rs` | McpConnection, state machine | NO — this story creates it |
| `crates/forge-core/tests/transport_tests.rs` | Integration tests with mock server | NO |
