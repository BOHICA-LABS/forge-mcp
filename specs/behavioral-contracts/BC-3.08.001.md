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
subsystem: "TUI Dashboard"
capability: "CAP-008"
lifecycle_status: active
introduced: v0.1.0
---

# BC-3.08.001 — JSON-RPC Syntax-Highlighted Message Rendering

## Summary

The traffic inspector pane renders captured JSON-RPC messages with syntax highlighting. Each message displays the method name, a directional arrow (→ for client-to-server, ← for server-to-client), and a timestamp. JSON payloads are color-coded by value type (keys, strings, numbers, booleans, null). Large payloads are truncated in the list view with an expand-on-select interaction.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | Traffic inspector pane is visible in the layout (BC-3.06.001) |
| PRE-002 | At least one captured JSON-RPC message exists in the capture buffer (BC-4.09.001) |
| PRE-003 | Color tier has been detected (BC-3.06.002) |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Each message row shows: `[timestamp] [→/←] method_name` |
| POST-002 | JSON keys are rendered in one color, string values in another, numbers in a third, booleans/null in a fourth |
| POST-003 | Color assignments are consistent with the detected color tier (BC-3.06.002) |
| POST-004 | Messages exceeding the truncation threshold (default: 5 lines) show a "[+N lines]" indicator |
| POST-005 | Selecting a truncated message (Enter) expands it to show the full payload |
| POST-006 | Pressing Esc on an expanded message collapses it back to truncated view |
| POST-007 | Direction arrow and method name are always visible (never truncated) |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | Syntax highlighting never alters the semantic content of the JSON payload |
| INV-002 | All rendered messages maintain wire order (DI-006) |
| INV-003 | Truncation only affects the display; the full message is always retrievable |
| INV-004 | Color token types are mutually exclusive (each character span has exactly one color) |

## Edge Cases

| ID | Scenario | Expected Behavior | Source |
|----|----------|-------------------|--------|
| EC-001 | Message payload is not valid JSON | Render as plain text (no highlighting); mark with ⚠ indicator | — |
| EC-002 | Message payload is empty (notification with no params) | Render method name and direction only; no payload section | — |
| EC-003 | Payload contains deeply nested JSON (>20 levels) | Render up to depth limit; show "[nested: N levels deep]" for deeper content | — |
| EC-004 | Payload contains very long string value (>10KB) | Truncate string display to 256 chars with "…[N chars]" indicator | — |
| EC-005 | Hundreds of messages visible simultaneously | Virtual scrolling; only render visible rows (ratatui StatefulList) | — |
| EC-006 | Binary/non-UTF8 content in payload | Render as hex dump with note "Binary content (N bytes)" | — |
| EC-007 | 16-color mode with limited palette | Use bold/dim/underline attributes to distinguish token types | — |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-HP-001 | `{"jsonrpc":"2.0","method":"tools/list","id":1}` client→server | Row: `[14:23:05.123] → tools/list` with JSON syntax highlighting |
| TV-HP-002 | Response with 3-line result payload | Payload rendered inline with color-coded JSON; no truncation |
| TV-HP-003 | Response with 50-line result payload | Shows first 5 lines + `[+45 lines]` indicator; Enter expands full |
| TV-HP-004 | Notification `{"jsonrpc":"2.0","method":"notifications/progress"}` | Row: `[14:23:06.456] ← notifications/progress` (no id, no expand) |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-EC-001 | Malformed JSON: `{"method": "test", "params": {broken` | Rendered as plain text with ⚠ indicator |
| TV-EC-002 | Empty params notification | Method and direction shown; no payload area |
| TV-EC-003 | 500 messages in buffer, 20 visible | Only 20 rows rendered; scrolling reveals others |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-ERR-001 | Capture buffer returns error on read | Traffic inspector shows "Error reading capture buffer" message |
| TV-ERR-002 | Message with NaN/Infinity in JSON (non-standard) | Render as plain text with ⚠; log parse warning |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | For all valid JSON payloads: every character span has exactly one syntax color | Property-based test |
| VP-002 | For all messages: expanding and collapsing produces identical truncated view | Roundtrip test |
| VP-003 | Wire order (DI-006) is preserved in rendered message list | Invariant test |
| VP-004 | Virtual scrolling renders correct items for any scroll position | Property-based test |

## Traceability

| Item | Reference |
|------|-----------|
| L2 Capability | CAP-008 (TUI Data Display) |
| Domain Invariant | DI-006 (wire order preservation) |
| Related BCs | BC-3.06.002 (color tier), BC-4.09.001 (message capture), BC-3.08.005 (accessibility) |
