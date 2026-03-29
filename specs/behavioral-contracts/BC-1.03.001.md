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

# BC-1.03.001 — Daemon Lazy Start and Session Pooling

## Summary

The Forge daemon starts lazily on the first connection request if not already running. It pools MCP server connections so that concurrent TUI and CLI clients can share the same server sessions. The daemon shuts down after a configurable idle timeout.

## Preconditions

- PRE-001: A client (TUI or CLI) requests a connection to an MCP server.
- PRE-002: The daemon socket path is deterministic and known to all clients (e.g., `$XDG_RUNTIME_DIR/forge-mcp/daemon.sock` or platform equivalent).

## Postconditions

- POST-001: If no daemon is running, a new daemon process is started in the background before the connection request is fulfilled.
- POST-002: The daemon listens on a Unix domain socket (macOS/Linux) or named pipe (Windows).
- POST-003: MCP server connections are pooled: if server `X` is already connected in the daemon, a new client requesting server `X` reuses the existing connection.
- POST-004: Each client gets a unique client session ID for request routing, even when sharing a pooled server connection.
- POST-005: The daemon tracks the number of active client sessions. When the count drops to zero, an idle timer starts.
- POST-006: After idle timeout (configurable, default 300s) with zero active clients, the daemon performs graceful shutdown of all server connections and exits.
- POST-007: The daemon PID is written to a lock file alongside the socket for stale detection (BC-1.03.003).

## Invariants

- **DI-003**: Every session ID is unique across all clients and daemon restarts. Session IDs include a timestamp or random component.

## Edge Cases

| ID | Condition | Expected Behavior |
|----|-----------|-------------------|
| EC-001 | DEC-006: Daemon is already running when a new client starts | Client connects to existing daemon via socket. No new daemon spawned. |
| EC-002 | Two CLI clients connect simultaneously to the same server | Both share the pooled connection. Requests are multiplexed with unique request IDs. Responses are routed to the correct client session. |
| EC-003 | Daemon idle timeout fires but a client connects during shutdown | Abort shutdown. Reset idle timer. Service the new client. |
| EC-004 | Client disconnects ungracefully (SIGKILL, crash) | Daemon detects broken socket. Decrements active client count. If zero clients remain, starts idle timer. |
| EC-005 | Daemon idle timeout set to 0 | Daemon shuts down immediately when last client disconnects (no pooling benefit, but valid config). |
| EC-006 | 100+ concurrent client sessions | No hardcoded limit. Performance may degrade but functionality must be preserved. |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-001 | First CLI client requests connection, no daemon running | Daemon starts, socket created, client connects, server connection established |
| TV-002 | Second TUI client connects while first CLI is active | TUI connects to existing daemon. If requesting same server, reuses pooled connection. |
| TV-003 | Both clients disconnect, 300s passes | Daemon shuts down gracefully. Socket file removed. Lock file removed. |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-004 | Client connects during daemon idle shutdown | Shutdown aborted, client served, idle timer reset |
| TV-005 | Idle timeout = 0, single client connects then disconnects | Daemon shuts down immediately after client disconnect |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-006 | Socket path directory does not exist | Daemon creates directory with 0700 permissions, then creates socket |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | Session IDs are unique across 10,000 concurrent sessions | Property test: generate 10,000 IDs, assert uniqueness |
| VP-002 | Daemon shuts down within 5s of idle timeout expiration (when no clients) | Integration test with short idle timeout (1s) |
| VP-003 | Pooled connections correctly route responses to the requesting client | Integration test: two clients, interleaved requests, verify response routing |

## Traceability

- **L2 Capability**: CAP-003 (Daemon & Session Management)
- **Domain Invariant**: DI-003 (Session ID uniqueness)
- **Edge Cases**: DEC-006
- **Priority**: P0
