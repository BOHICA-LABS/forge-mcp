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
subsystem: "MCP Protocol Operations"
capability: "CAP-005"
lifecycle_status: active
introduced: v0.1.0
---

# BC-2.05.009 — Progress Tracking and Cancellation

## Summary

Tracks progress of long-running server operations via `notifications/progress` with `progressToken`. Supports cancellation of in-flight requests by sending `notifications/cancelled`. Displays progress in TUI and handles the interaction between progress and cancellation.

## Preconditions

- PRE-001: Connection is initialized.
- PRE-002: A request has been sent to the server with a `_meta.progressToken` field.

## Postconditions

- POST-001: When sending a request that may be long-running (e.g., `tools/call`), Forge includes a `_meta: {progressToken: "<unique_token>"}` in the request.
- POST-002: `notifications/progress` from the server are parsed: `{progressToken, progress, total (optional), message (optional)}`.
- POST-003: Progress is matched to the originating request via `progressToken`.
- POST-004: In TUI mode: progress is displayed as a progress bar or percentage (if `total` is provided), or as a spinner with message (if no `total`).
- POST-005: In CLI mode: progress is displayed on stderr as `[server] <message> <progress>/<total>` or `[server] <message> ...` (if no total).
- POST-006: `notifications/cancelled` is sent with `{requestId, reason (optional)}` to cancel a request.
- POST-007: After cancellation is sent, any subsequent `notifications/progress` for the cancelled `progressToken` are silently ignored.
- POST-008: The server may still complete the request after cancellation is sent (cancellation is cooperative). Forge accepts the response.

## Invariants

- None specific. Progress and cancellation are cooperative.

## Edge Cases

| ID | Condition | Expected Behavior |
|----|-----------|-------------------|
| EC-001 | DEC-024: Progress notification for a cancelled request | Silently ignored. No display update. No error. |
| EC-002 | Progress notification with `progressToken` that does not match any pending request | Silently ignored. Log debug message. |
| EC-003 | `progress` exceeds `total` (e.g., progress=150, total=100) | Clamp display to 100%. Log warning `E-PRT-010: Progress exceeds total for <token>`. |
| EC-004 | Multiple concurrent requests with progress | Each tracked independently by unique `progressToken`. TUI shows multiple progress indicators. |
| EC-005 | Cancel a request that has already completed | No-op. The cancellation notification is sent but the server has already responded. |
| EC-006 | Server never sends progress despite progressToken | No display update. Request completes normally. Not an error. |
| EC-007 | Progress notification with no `message` and no `total` | Display spinner with server name only. |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-001 | Request with `progressToken: "tok-1"`, server sends `progress: 50, total: 100, message: "Indexing"` | TUI: progress bar at 50%, label "Indexing". CLI: `[server] Indexing 50/100`. |
| TV-002 | User cancels request with `requestId: 42` | `notifications/cancelled {requestId: 42, reason: "User cancelled"}` sent to server. |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-003 | Progress notification for cancelled token "tok-1" | Ignored. No display update. |
| TV-004 | Progress with `progress: 200, total: 100` | Display 100%. Warning `E-PRT-010` logged. |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-005 | Cancel sent, server still responds with result | Response accepted and processed normally. Cancellation was cooperative. |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | Progress for cancelled requests is silently ignored (DEC-024) | Unit test: cancel → progress → verify no display update |
| VP-002 | Unique `progressToken` per request | Property test: 1000 concurrent requests → all tokens unique |
| VP-003 | Cancellation does not break request/response matching | Integration test: cancel → server responds → verify response processed |

## Traceability

- **L2 Capability**: CAP-005 (Core MCP Operations)
- **Edge Cases**: DEC-024
- **Priority**: P1
