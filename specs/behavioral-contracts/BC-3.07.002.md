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
capability: "CAP-007"
lifecycle_status: active
introduced: v0.1.0
---

# BC-3.07.002 — Search and Command Mode (/ and :)

## Summary

The TUI dashboard provides two modal input modes activated by single-character triggers: search mode (`/`) performs incremental filtering within the currently focused pane, and command mode (`:`) opens a command palette for executing application-level commands. Both modes capture keyboard input into a text prompt and override normal navigation until dismissed.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | TUI is in normal navigation mode (not already in search or command mode) |
| PRE-002 | At least one pane is focused (BC-3.07.001) |
| PRE-003 | For search mode: focused pane contains searchable content (list or table) |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | `/` activates search mode: a search prompt appears at the bottom of the focused pane |
| POST-002 | Search is incremental: results filter as each character is typed |
| POST-003 | Matching items are highlighted in the focused pane's list/table |
| POST-004 | `:` activates command mode: a command prompt appears in the status bar area |
| POST-005 | Enter in search mode applies the filter and returns to navigation mode with results |
| POST-006 | Enter in command mode executes the typed command and returns to navigation mode |
| POST-007 | Esc in either mode cancels the input, discards the query, and returns to navigation mode |
| POST-008 | The search/command prompt shows the current input text in real time |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | At most one modal input mode is active at any time (search XOR command XOR normal) |
| INV-002 | Navigation keys (hjkl, Tab) are not processed while in search or command mode |
| INV-003 | Esc always exits the current modal mode and returns to normal navigation |
| INV-004 | Search filtering never mutates the underlying data; it only affects the view |

## Edge Cases

| ID | Scenario | Expected Behavior | Source |
|----|----------|-------------------|--------|
| EC-001 | `/` pressed when focused pane has no searchable content (e.g., sparkline chart) | No-op; remain in navigation mode | — |
| EC-002 | Search query matches zero items | Display "No matches" in the pane; list shows empty | — |
| EC-003 | `:connect` with no argument | Display usage hint: ":connect <server-name>" | — |
| EC-004 | Unknown command entered (e.g., `:foobar`) | Display "Unknown command: foobar" in status bar for 3 seconds | — |
| EC-005 | Very long search query (>200 chars) | Truncate display to fit prompt width; full query is used for matching | — |
| EC-006 | `:quit` while unsaved state exists | No unsaved state concept in TUI (stateless view); quit immediately | — |
| EC-007 | `/` pressed while already in search mode | No-op; `/` character is treated as literal search input | — |

## Supported Commands

| Command | Arguments | Description |
|---------|-----------|-------------|
| `:connect` | `<server-name>` | Connect to a named server from the registry |
| `:disconnect` | `[server-name]` | Disconnect from current or named server |
| `:quit` / `:q` | — | Exit the TUI application |
| `:filter` | `<expression>` | Apply traffic filter (see BC-4.10.001) |
| `:export` | `<format> <path>` | Export captured traffic (json, csv) to file path |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-HP-001 | Focus on server browser, type `/local` | Server list filters to show only servers matching "local"; matches highlighted |
| TV-HP-002 | In search mode, press Enter | Search filter applied; navigation mode restored; filtered list remains |
| TV-HP-003 | Type `:connect my-server` then Enter | Connection initiated to "my-server"; status bar shows connecting state |
| TV-HP-004 | Type `:quit` then Enter | TUI exits cleanly |
| TV-HP-005 | Type `:export json /tmp/traffic.json` then Enter | Captured traffic exported to specified path; confirmation in status bar |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-EC-001 | `/` on sparkline pane | No-op; remains in navigation mode |
| TV-EC-002 | Search "zzzzz" in server list with no matches | "No matches" displayed; empty list |
| TV-EC-003 | `:connect` with no argument | Status bar shows ":connect <server-name>" usage hint |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-ERR-001 | `:connect nonexistent-server` | Status bar shows "Server not found: nonexistent-server" |
| TV-ERR-002 | `:export json /root/noperm.json` (no write permission) | Status bar shows "Export failed: permission denied: /root/noperm.json" |
| TV-ERR-003 | `:foobar` | Status bar shows "Unknown command: foobar" for 3 seconds |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | For all valid search strings: filtered results are a subset of unfiltered results | Property-based test |
| VP-002 | For all mode transitions: Esc always returns to normal navigation mode | State machine test |
| VP-003 | Incremental search produces same results as full-string search for the same final query | Equivalence test |
| VP-004 | All supported commands are parseable and dispatchable without panic | Fuzz test |

## Traceability

| Item | Reference |
|------|-----------|
| L2 Capability | CAP-007 (TUI Interaction) |
| Related BCs | BC-3.07.001 (navigation mode), BC-3.07.003 (mouse input), BC-4.10.001 (traffic filtering) |
