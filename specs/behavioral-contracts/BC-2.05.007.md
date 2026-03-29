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

# BC-2.05.007 — Logging Level Control and Message Display

## Summary

Sends `logging/setLevel` requests to MCP servers to control their log output verbosity. Handles incoming `notifications/message` from servers and routes them to the appropriate display: TUI log panel or stderr in CLI mode.

## Preconditions

- PRE-001: Connection is initialized and `capabilities.logging` is present in negotiated capabilities.
- PRE-002: For `logging/setLevel`: a valid log level is specified.

## Postconditions

- POST-001: `logging/setLevel` sends `{level}` where level is one of: `debug`, `info`, `notice`, `warning`, `error`, `critical`, `alert`, `emergency`.
- POST-002: The server adjusts its logging output. Only messages at or above the configured level are sent.
- POST-003: Incoming `notifications/message` are parsed: `{level, logger (optional), data}`.
- POST-004: In TUI mode: log messages are displayed in a dedicated log panel, color-coded by severity:
  - debug/info: default color
  - notice/warning: yellow
  - error/critical/alert/emergency: red
- POST-005: In CLI mode: log messages are written to stderr, prefixed with `[<server_name>] [<level>] <data>`.
- POST-006: Log messages are also captured in the traffic inspector as notification events.

## Invariants

- None specific. Logging is best-effort display.

## Edge Cases

| ID | Condition | Expected Behavior |
|----|-----------|-------------------|
| EC-001 | Server does not support logging (no `capabilities.logging`) | `logging/setLevel` returns `Err(E-CAP-001)`. Incoming `notifications/message` are still processed (they may arrive regardless). |
| EC-002 | Invalid log level string | Return `Err(E-PRT-008: Invalid log level: <level>. Must be one of: debug, info, notice, warning, error, critical, alert, emergency)`. |
| EC-003 | `notifications/message` with `data` as non-string (object, array) | Serialize `data` as JSON string for display. |
| EC-004 | High-frequency log messages (>100/s) | Display all in TUI (scrolling). No rate limiting on display. Traffic inspector may apply sampling for storage. |
| EC-005 | `notifications/message` with missing `level` field | Default to `info`. Log warning about malformed notification. |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-001 | `logging/setLevel {level: "warning"}` | Request sent to server. Server acknowledges (empty result). |
| TV-002 | Server sends `notifications/message {level: "error", logger: "db", data: "Connection lost"}` | TUI: red text `[db] Connection lost`. CLI: stderr `[server-name] [error] Connection lost`. |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-003 | `logging/setLevel {level: "invalid"}` | `Err(E-PRT-008)` |
| TV-004 | `notifications/message {data: {"key": "value"}}` (object data, no level) | Displayed as: `[info] {"key": "value"}` |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-005 | `logging/setLevel` when server lacks logging capability | `Err(E-CAP-001: Server does not support logging)` |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | All 8 log levels are accepted by `logging/setLevel` | Unit test: iterate all levels, verify no error |
| VP-002 | Log messages appear in both TUI display and traffic inspector | Integration test: send notification, verify both outputs |
| VP-003 | CLI mode writes to stderr, not stdout | Integration test: capture stderr/stdout separately, verify log in stderr |

## Traceability

- **L2 Capability**: CAP-005 (Core MCP Operations)
- **Priority**: P2
