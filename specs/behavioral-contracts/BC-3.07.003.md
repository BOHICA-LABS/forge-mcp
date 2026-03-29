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

# BC-3.07.003 — Mouse Supplementary Input

## Summary

The TUI dashboard supports mouse input as a supplementary interaction method. Mouse clicks select items in lists and tables, and the scroll wheel scrolls content within panes. Mouse input is strictly supplementary — every action achievable via mouse is also achievable via keyboard (BC-3.07.001). Mouse support can be enabled or disabled via configuration.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | TUI layout is rendered (BC-3.06.001) |
| PRE-002 | Mouse support is enabled in configuration (`tui.mouse_enabled = true`, default: true) |
| PRE-003 | Terminal supports mouse reporting (xterm-style mouse protocol) |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Left-click on an item in a list/table selects that item and moves focus to the containing pane |
| POST-002 | Scroll wheel up/down scrolls the content of the pane under the cursor |
| POST-003 | Left-click on a pane header moves focus to that pane without selecting an item |
| POST-004 | Mouse events outside any pane boundary are ignored |
| POST-005 | When mouse is disabled via config, no mouse capture escape sequences are emitted |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | Every action achievable via mouse is also achievable via keyboard |
| INV-002 | Mouse interaction never enters a state unreachable by keyboard |
| INV-003 | Mouse and keyboard inputs can be freely interleaved without state corruption |
| INV-004 | Disabling mouse at runtime does not affect current selection or focus state |

## Edge Cases

| ID | Scenario | Expected Behavior | Source |
|----|----------|-------------------|--------|
| EC-001 | Click on the border between two panes | Focus moves to the pane whose content area is closer to the click coordinate | — |
| EC-002 | Click on the status bar | No-op (status bar is not interactive via mouse) | — |
| EC-003 | Scroll wheel on a pane with no scrollable content | No-op | — |
| EC-004 | Mouse enabled but terminal doesn't support mouse reporting | Mouse events are simply never received; keyboard works normally | — |
| EC-005 | Click during search mode (BC-3.07.002) | Click cancels search mode and processes the click as normal navigation | — |
| EC-006 | Right-click anywhere | No-op (no context menu in v1) | — |
| EC-007 | Mouse disabled mid-session via config reload | Mouse capture sequences disabled on next frame; existing focus preserved |  — |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-HP-001 | Left-click on third item in server browser list | Server browser gains focus; third item is selected/highlighted |
| TV-HP-002 | Scroll wheel down in traffic inspector | Traffic list scrolls down; visible items shift |
| TV-HP-003 | Click on capability explorer header | Capability explorer gains focus; no item selected |
| TV-HP-004 | Click server → scroll traffic → press `j` | All three inputs processed correctly in sequence (interleaved mouse/keyboard) |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-EC-001 | Click on pane border | Focus moves to nearest pane content area |
| TV-EC-002 | Scroll on health metrics sparkline | No-op (sparklines are not scrollable) |
| TV-EC-003 | `tui.mouse_enabled = false` in config | No mouse capture; terminal mouse works normally (copy/paste) |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-ERR-001 | Mouse event with coordinates outside terminal bounds | Ignored; no panic or error |
| TV-ERR-002 | Malformed mouse escape sequence from terminal | Ignored; logged at debug level |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | For all mouse-reachable states: an equivalent keyboard-only path exists | Reachability test |
| VP-002 | Interleaved mouse and keyboard input sequences never produce inconsistent state | Fuzz test |
| VP-003 | When mouse is disabled: zero mouse capture escape sequences in output | Invariant test |
| VP-004 | Click coordinates always map to the correct pane (no off-by-one in hit testing) | Property-based test |

## Traceability

| Item | Reference |
|------|-----------|
| L2 Capability | CAP-007 (TUI Interaction) |
| Related BCs | BC-3.07.001 (keyboard navigation — primary), BC-3.07.002 (search/command mode) |
| NFR | NFR-015 (accessibility — mouse is supplementary, not required) |
