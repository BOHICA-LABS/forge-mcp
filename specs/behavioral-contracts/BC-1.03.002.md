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
capability: "CAP-003"
lifecycle_status: active
introduced: v0.1.0
---

# BC-1.03.002 — Named Session Persistence Across CLI Invocations

## Summary

Named sessions persist beyond the lifetime of a single CLI process. A user can start a session with `forge session create --name my-session`, disconnect the CLI, and later reconnect with `forge session attach my-session` to find the same MCP server connections active (kept alive by the daemon). This enables warm startup for repeated agent interactions.

## Preconditions

- PRE-001: The Forge daemon is running (started lazily by BC-1.03.001 if needed).
- PRE-002: A session name is provided (non-empty, alphanumeric + hyphens, max 64 chars).

## Postconditions

- POST-001: `session create --name <N>` creates a named session in the daemon's session registry with the given name.
- POST-002: The session holds references to all MCP server connections established during the session.
- POST-003: When the creating CLI process exits, the session remains in the daemon. Server connections remain alive.
- POST-004: `session attach <N>` reconnects a CLI process to the named session. The client receives the current state of all server connections (names, health status, capabilities).
- POST-005: `session list` returns all named sessions with: name, creation time, last activity time, connected server count, active client count.
- POST-006: `session destroy <N>` gracefully shuts down all server connections in the session and removes it from the registry.
- POST-007: Session names are unique within a daemon instance. Attempting to create a duplicate returns `Err(E-SES-001: Session name already exists: <N>)`.

## Invariants

- **DI-003**: Session IDs (internal UUIDs) are unique. Session names (user-facing) are also unique within a daemon instance.

## Edge Cases

| ID | Condition | Expected Behavior |
|----|-----------|-------------------|
| EC-001 | CLI attaches to a session where a server has crashed since last use | Session is returned with the server in `Disconnected` state. Auto-reconnection may be in progress (BC-1.02.003). Client sees current state. |
| EC-002 | Daemon restarts (crash or manual) — all sessions lost | Sessions are in-memory only. After daemon restart, `session list` returns empty. CLI receives `Err(E-SES-002: Session not found)` on attach. User must create new sessions. |
| EC-003 | Session name contains invalid characters (spaces, `/`, etc.) | Return `Err(E-SES-003: Invalid session name. Must match [a-zA-Z0-9-]{1,64})`. |
| EC-004 | Two CLI clients attach to the same named session simultaneously | Both clients share the session. Both see the same server connections. Requests from either client are served. |
| EC-005 | Session with no server connections (created but never used) | Valid state. Session persists. `session list` shows 0 connected servers. |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-001 | `forge session create --name agent-loop`, connect to server, exit CLI | Session persists in daemon. `forge session list` shows `agent-loop` with 1 server. |
| TV-002 | `forge session attach agent-loop` (after CLI exit) | CLI reconnects. Server connection is live. Can immediately invoke tools. |
| TV-003 | `forge session destroy agent-loop` | All server connections in session closed. Session removed from registry. |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-004 | `forge session create --name agent-loop` when `agent-loop` already exists | `Err(E-SES-001: Session name already exists: agent-loop)` |
| TV-005 | `forge session attach old-session` after daemon restart | `Err(E-SES-002: Session not found: old-session)` |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-006 | `forge session create --name "my session"` (space in name) | `Err(E-SES-003: Invalid session name)` |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | Server connections survive CLI process exit | Integration test: create session, connect server, kill CLI, verify server still responding via new CLI attach |
| VP-002 | Session names are unique (no duplicates) | Unit test: create, attempt duplicate, verify error |
| VP-003 | `session destroy` cleans up all server connections | Integration test: destroy session, verify no orphan server processes |

## Traceability

- **L2 Capability**: CAP-003 (Daemon & Session Management)
- **Domain Invariant**: DI-003 (Session ID uniqueness)
- **Priority**: P1
