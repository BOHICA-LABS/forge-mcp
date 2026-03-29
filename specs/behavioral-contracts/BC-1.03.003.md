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

# BC-1.03.003 — Daemon Socket Conflict Detection and Recovery

## Summary

Detects when the daemon socket address is already in use on startup, determines whether the existing daemon is healthy, and provides recovery options: connect to the healthy daemon, or kill-and-restart an unresponsive one. Handles stale lock files left by crashed daemons.

## Preconditions

- PRE-001: A client has requested a connection, triggering daemon lazy start (BC-1.03.001).
- PRE-002: The daemon socket path is already occupied (bind fails with EADDRINUSE or equivalent).

## Postconditions

- POST-001: If a lock file exists, the PID in the lock file is checked:
  - If the PID is alive and responds to a health check on the socket → connect to existing daemon (no conflict).
  - If the PID is alive but does not respond to health check within 5s → report `E-DAE-001: Existing daemon (PID <N>) is unresponsive`.
  - If the PID is not alive → stale lock file. Remove lock file and socket file, then start new daemon.
- POST-002: If no lock file exists but socket file exists → treat as stale. Remove socket file and start new daemon.
- POST-003: Health check is a lightweight ping on the daemon control socket (not an MCP ping). Response must arrive within 5s.
- POST-004: When offering kill-and-restart for an unresponsive daemon:
  - In interactive mode (TUI/terminal): prompt user for confirmation.
  - In non-interactive mode (piped CLI): return error with instructions.
  - With `--force` flag: kill without prompting.
- POST-005: Kill sequence: SIGTERM → wait 5s → SIGKILL if still alive.
- POST-006: After successful recovery, the new daemon starts normally per BC-1.03.001.

## Invariants

- **DI-003**: The new daemon generates fresh session IDs. No session state from the old daemon is recovered.

## Edge Cases

| ID | Condition | Expected Behavior |
|----|-----------|-------------------|
| EC-001 | FM-005: Socket file exists, lock file exists, PID alive and healthy | Not a conflict. Connect to existing daemon. |
| EC-002 | Socket file exists, lock file exists, PID alive but unresponsive | Conflict. Offer kill-and-restart (interactive) or error (non-interactive). |
| EC-003 | Socket file exists, lock file exists, PID dead | Stale state. Clean up lock + socket, start new daemon. |
| EC-004 | Socket file exists, no lock file | Stale socket from crash without lock file. Remove socket, start new daemon. |
| EC-005 | Lock file contains non-numeric PID | Treat as corrupt. Remove lock file. Attempt to connect to socket. If fails, remove socket and start new daemon. |
| EC-006 | Race condition: two clients trigger daemon start simultaneously | First client creates lock file atomically (O_CREAT | O_EXCL). Second client sees lock file, checks PID, connects to the daemon started by first client. |
| EC-007 | Permission denied removing stale socket/lock file | Return `Err(E-DAE-002: Cannot clean up stale daemon files: permission denied)`. Manual cleanup required. |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-001 | Socket exists, lock PID alive, health check passes | Client connects to existing daemon. No new daemon started. |
| TV-002 | Socket exists, lock PID dead (stale) | Lock + socket removed. New daemon starts. Client connects. |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-003 | Socket exists, lock PID alive, health check times out (5s), `--force` flag | Old daemon killed (SIGTERM → SIGKILL). New daemon starts. |
| TV-004 | Two clients race to start daemon | One wins (creates lock atomically). Other connects to the winner's daemon. No duplicate daemons. |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-005 | Socket exists, PID alive, unresponsive, non-interactive mode, no `--force` | `Err(E-DAE-001: Existing daemon (PID 12345) is unresponsive. Use --force to kill and restart.)` |
| TV-006 | Cannot remove stale socket (permission denied) | `Err(E-DAE-002)` |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | Stale lock files are always cleaned up when PID is dead | Integration test: create stale lock, verify cleanup |
| VP-002 | No duplicate daemons can run simultaneously | Concurrency test: 10 clients race to start daemon, verify exactly 1 daemon running |
| VP-003 | Kill sequence respects SIGTERM → SIGKILL escalation | Integration test with unresponsive mock daemon |

## Traceability

- **L2 Capability**: CAP-003 (Daemon & Session Management)
- **Domain Invariant**: DI-003
- **Failure Modes**: FM-005
- **Priority**: P1
