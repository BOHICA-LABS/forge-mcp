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

# BC-1.02.003 — Connection Lifecycle Management

## Summary

Manages the full lifecycle of an established MCP connection: keepalive monitoring via ping, graceful shutdown on disconnect, crash/timeout detection, and automatic reconnection with exponential backoff. For HTTP transport, handles session re-establishment when the `Mcp-Session-Id` becomes invalid.

## Preconditions

- PRE-001: A connection has been successfully established (via BC-1.02.001 or BC-1.02.002).
- PRE-002: The connection is registered in the session's connection pool with state `Connected`.

## Postconditions

- POST-001: Keepalive pings are sent at a configurable interval (default: 30s) to detect unresponsive servers.
- POST-002: If a ping fails or times out, the connection state transitions to `Unhealthy`.
- POST-003: On explicit disconnect (user-initiated or session close), a graceful shutdown sequence is executed: pending requests are drained (up to 5s timeout), then the transport is closed.
- POST-004: For stdio transport: the child process receives SIGTERM, then SIGKILL after 5s if still running.
- POST-005: For HTTP transport: the connection is closed. No server-side cleanup is needed (HTTP is stateless at transport level).
- POST-006: On unexpected disconnection (crash, network failure), the connection state transitions to `Disconnected` and automatic reconnection begins.
- POST-007: Reconnection uses exponential backoff: 1s, 2s, 4s, 8s, 16s, 32s, capped at 60s. Max attempts: 10.
- POST-008: For HTTP transport, if the server returns HTTP 404 with the stored `Mcp-Session-Id`, the session is re-established (new initialize handshake) per FM-021.
- POST-009: Connection state transitions are emitted as events for the TUI/CLI to display.

## Invariants

- **DI-004**: Transport layer is managed exclusively by rmcp. Forge manages lifecycle decisions (when to reconnect) but delegates transport operations to rmcp.

## Edge Cases

| ID | Condition | Expected Behavior |
|----|-----------|-------------------|
| EC-001 | FM-002: Stdio server process crashes (unexpected exit) | Detect via process exit signal. Transition to `Disconnected`. Begin reconnection (re-spawn process). |
| EC-002 | FM-003: Server stops responding but process is alive (hung) | Detected by ping timeout. Transition to `Unhealthy`. After 3 consecutive ping failures, transition to `Disconnected` and attempt reconnection (kill + restart for stdio). |
| EC-003 | FM-021: HTTP server returns 404 for stored session ID | Discard stored `Mcp-Session-Id`. Perform new initialize handshake. If new handshake succeeds, transition back to `Connected`. |
| EC-004 | Reconnection exhausts max attempts (10) | Transition to `Failed`. Emit `E-CON-011: Reconnection failed after 10 attempts for server <name>`. No further automatic reconnection. Manual reconnect required. |
| EC-005 | Disconnect requested during active tool call | Wait up to 5s for in-flight requests to complete. If not completed, cancel them and proceed with disconnect. |
| EC-006 | Server disconnects and reconnects rapidly (flapping) | Exponential backoff prevents tight reconnect loops. After 3 disconnects within 60s, emit warning `E-CON-012: Server <name> is flapping`. |
| EC-007 | Network partition (HTTP transport) | Detected by failed ping or failed request. Reconnection attempts will fail until network is restored. Backoff continues. |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-001 | Connected server, 30s passes | Ping sent, pong received, state remains `Connected` |
| TV-002 | User requests disconnect of healthy connection | Graceful shutdown: drain pending → close transport → state = `Disconnected`. For stdio: SIGTERM sent to child. |
| TV-003 | Stdio server crashes, reconnect succeeds on first attempt | State: `Connected` → `Disconnected` → `Reconnecting` → `Connected`. Total downtime ≈ 1s (first backoff). |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-004 | HTTP server returns 404 for session ID | New initialize handshake, new session ID stored, state returns to `Connected` |
| TV-005 | 10 consecutive reconnection failures | State = `Failed`. Event E-CON-011 emitted. No further auto-reconnect. |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-006 | Stdio child process killed by OOM (SIGKILL, exit 137) | Detected, state → `Disconnected`, reconnection begins. stderr captured for diagnostics. |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | No orphan child processes after disconnect (stdio) | Integration test: disconnect, wait 10s, verify no orphan processes with same command |
| VP-002 | Exponential backoff intervals follow the defined schedule | Unit test: mock clock, verify reconnect timing |
| VP-003 | Connection state machine has no invalid transitions | State machine model check: `Connected`, `Unhealthy`, `Disconnected`, `Reconnecting`, `Failed` — verify all transitions |

## Traceability

- **L2 Capability**: CAP-002 (Transport Connection)
- **Domain Invariant**: DI-004
- **Failure Modes**: FM-002, FM-003, FM-021
- **Priority**: P0
