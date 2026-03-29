---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-03-29T11:25:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: ""
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: "Server Discovery & Connection Management"
capability: "CAP-002"
lifecycle_status: active
introduced: v0.1.0
---

# BC-1.02.001 — Stdio Transport Connection Establishment

## Summary

Establishes a connection to an MCP server via stdio transport. Spawns the server process with the configured command, arguments, and environment variables, then connects stdin/stdout pipes through the rmcp library and completes the MCP initialize/initialized handshake.

## Preconditions

- PRE-001: A `ServerEntry` with `transport: Stdio` exists in the unified registry.
- PRE-002: The `ServerEntry` has a non-empty `command` field.
- PRE-003: The server is marked as `enabled: true`.
- PRE-004: No existing active connection to this server name exists in the session (or reconnection is explicitly requested).

## Postconditions

- POST-001: A child process is spawned with the configured `command`, `args`, and `env`.
- POST-002: Environment variables from `env` are merged with the current process environment. Config `env` values override process env for duplicate keys.
- POST-003: The rmcp `ClientTransport` is connected to the child process's stdin (write) and stdout (read).
- POST-004: The MCP `initialize` request is sent and a valid `InitializeResult` is received containing `serverInfo`, `capabilities`, and `protocolVersion`.
- POST-005: The `notifications/initialized` notification is sent after successful initialize.
- POST-006: The connection is registered in the session's connection pool with state `Connected`.
- POST-007: stderr of the child process is captured and forwarded to the logging subsystem (not mixed with MCP protocol traffic).

## Invariants

- **DI-001**: Capability negotiation (initialize/initialized) MUST complete before any other MCP method is called.
- **DI-004**: Transport layer is managed exclusively by rmcp. Forge does not directly read/write stdin/stdout bytes.

## Edge Cases

| ID | Condition | Expected Behavior |
|----|-----------|-------------------|
| EC-001 | FM-002: Server process crashes immediately on start (exit code != 0 within 1s) | Return `Err(E-CON-001: Server process exited with code <N> during startup)`. Capture stderr for error message. |
| EC-002 | Command binary not found in PATH | Return `Err(E-CON-002: Command not found: <command>)`. |
| EC-003 | Permission denied executing command | Return `Err(E-CON-003: Permission denied: <command>)`. |
| EC-004 | Server starts but does not respond to initialize within timeout (30s default) | Return `Err(E-CON-004: Initialize timeout after <N>s)`. Kill child process. |
| EC-005 | Server responds to initialize with unsupported protocol version | Accept connection but log warning `E-CON-006`. Graceful degradation handled by BC-2.04.003. |
| EC-006 | `env` contains `PATH` override | Merge: config `PATH` replaces (not appends to) process `PATH`. |
| EC-007 | Command is a relative path (e.g., `./server`) | Resolve relative to current working directory. If CWD is ambiguous, resolve relative to project root if available. |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-001 | `command: "npx", args: ["-y", "@modelcontextprotocol/server-filesystem", "/tmp"]` | Process spawned, initialize handshake completes, connection state = `Connected`, `serverInfo.name` = "filesystem" |
| TV-002 | `command: "node", args: ["server.js"], env: {"API_KEY": "test123"}` | Process env includes `API_KEY=test123`, connection established |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-003 | `command: "nonexistent-binary"` | `Err(E-CON-002: Command not found: nonexistent-binary)` |
| TV-004 | Server process writes to stderr during startup | stderr captured in log, connection still succeeds if initialize handshake completes |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-005 | Server exits with code 1 immediately | `Err(E-CON-001: Server process exited with code 1 during startup)` with stderr content |
| TV-006 | Server hangs (never responds to initialize) | After 30s timeout: `Err(E-CON-004)`, child process killed |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | Child process is always killed on connection failure | Integration test: verify no orphan processes after error paths |
| VP-002 | stderr is never mixed with stdout MCP traffic | Integration test: server writes to stderr, verify MCP messages unaffected |
| VP-003 | Environment variable merging is deterministic | Unit test: config env + process env → verify merge result |

## Traceability

- **L2 Capability**: CAP-002 (Transport Connection)
- **Domain Invariants**: DI-001, DI-004
- **Failure Modes**: FM-002
- **Priority**: P0
