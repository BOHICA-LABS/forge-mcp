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

# BC-3.07.001 — Vi-Style Keyboard Navigation Across Panes

## Summary

The TUI dashboard supports vi-style keyboard navigation for all pane movement and item selection. Users can navigate between panes and within list/table content using hjkl keys, cycle panes with Tab/Shift-Tab, select items with Enter, and cancel/go back with Esc. The entire application is fully operable via keyboard alone (accessibility requirement).

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | TUI layout is rendered with at least one visible pane (BC-3.06.001) |
| PRE-002 | Keyboard input is connected to the terminal (stdin is a TTY) |
| PRE-003 | Application is not in search mode (/) or command mode (:) — see BC-3.07.002 |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Exactly one pane has focus at any time, indicated by a highlighted border |
| POST-002 | The focused pane's title shows a focus indicator (e.g., bold or marker) |
| POST-003 | hjkl moves focus directionally between adjacent panes |
| POST-004 | j/k moves the cursor down/up within a list or table in the focused pane |
| POST-005 | Enter selects the currently highlighted item in the focused pane |
| POST-006 | Esc returns to the previous context (deselect item → unfocus pane → no-op at root) |
| POST-007 | Tab cycles focus forward through panes in a consistent order; Shift-Tab cycles backward |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | Exactly one pane is focused at all times (never zero, never more than one) |
| INV-002 | Focus order for Tab cycling is deterministic and stable across frames |
| INV-003 | Every interactive element in the TUI is reachable by keyboard alone |
| INV-004 | No key binding conflicts exist between navigation mode and other modes |
| INV-005 | Key repeat (holding a key) produces repeated navigation events |

## Edge Cases

| ID | Scenario | Expected Behavior | Source |
|----|----------|-------------------|--------|
| EC-001 | h pressed when focus is on leftmost pane | No-op; focus stays on current pane | — |
| EC-002 | l pressed when focus is on rightmost pane | No-op; focus stays on current pane | — |
| EC-003 | j pressed at bottom of a list | No-op; cursor stays at last item (no wrap) | — |
| EC-004 | k pressed at top of a list | No-op; cursor stays at first item (no wrap) | — |
| EC-005 | Tab pressed when only one pane is visible (80×24 collapsed) | No-op or cycle to status bar if interactive | — |
| EC-006 | Enter pressed on empty list | No-op; no selection event emitted | — |
| EC-007 | Rapid key repeat (>30 keys/sec) | All events processed in order; no dropped inputs | — |
| EC-008 | Esc pressed at root level (no selection, no sub-context) | No-op | — |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-HP-001 | Focus on server browser, press `l` | Focus moves to capability explorer (center pane) |
| TV-HP-002 | Focus on capability explorer, press `j` three times | Cursor moves down three items in the capability list |
| TV-HP-003 | Cursor on a tool in capability explorer, press `Enter` | Tool schema detail view opens |
| TV-HP-004 | In tool detail view, press `Esc` | Returns to capability list; cursor position preserved |
| TV-HP-005 | Press `Tab` five times from server browser | Focus cycles through all panes and returns to server browser |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-EC-001 | Focus on server browser (leftmost), press `h` | No change; focus remains on server browser |
| TV-EC-002 | Cursor at bottom of 50-item list, press `j` | No change; cursor remains on item 50 |
| TV-EC-003 | 80×24 terminal with collapsed panes, press `Tab` | Focus moves to next visible pane only |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-ERR-001 | stdin disconnected during operation | Application detects lost input and exits gracefully with E-TUI-003 |
| TV-ERR-002 | Unrecognized key code received | Ignored silently; no error displayed |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | For all key sequences: exactly one pane is focused after processing | Invariant test |
| VP-002 | For all navigation keys at boundary positions: no panic or out-of-bounds access | Fuzz test |
| VP-003 | Tab cycling visits all visible panes exactly once before returning to start | Property-based test |
| VP-004 | All interactive elements are reachable from initial state via keyboard-only input | Reachability test |

## Traceability

| Item | Reference |
|------|-----------|
| L2 Capability | CAP-007 (TUI Interaction) |
| Related BCs | BC-3.07.002 (search/command mode), BC-3.07.003 (mouse supplement), BC-3.06.001 (layout) |
| NFR | NFR-015 (accessibility — keyboard-only operation) |
