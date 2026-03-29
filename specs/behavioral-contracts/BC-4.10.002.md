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
subsystem: "Traffic Inspection"
capability: "CAP-010"
lifecycle_status: active
introduced: v0.1.0
---

# BC-4.10.002 — Full-Text Payload Search

## Summary

The traffic inspection subsystem supports regex-based full-text search across all captured JSON-RPC message payloads. Matches are highlighted in the traffic inspector pane. The search supports case-insensitive mode. Search operates on the raw captured payload bytes and can be combined with filters (BC-4.10.001).

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | Capture buffer contains at least one message (BC-4.09.001) |
| PRE-002 | Traffic inspector pane is visible |
| PRE-003 | User has activated search mode (`/` in traffic inspector pane, per BC-3.07.002) |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | All messages containing a regex match in their payload are shown |
| POST-002 | Matched substrings are visually highlighted (bold + color per BC-3.08.005) |
| POST-003 | Match count is displayed: "N matches in M messages" |
| POST-004 | j/k or n/N navigates between matches (next/previous match) |
| POST-005 | Case-insensitive mode is togglable (default: case-sensitive; toggle via `/i` prefix or keybinding) |
| POST-006 | Search results combine with active filters (AND logic) — only filtered messages are searched |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | Search never modifies the capture buffer or filter state |
| INV-002 | Search results are a subset of the currently filtered view (or full buffer if no filters active) |
| INV-003 | Highlight rendering does not alter the underlying payload display |
| INV-004 | Match navigation (n/N) wraps around at boundaries (last match → first match) |

## Edge Cases

| ID | Scenario | Expected Behavior | Source |
|----|----------|-------------------|--------|
| EC-001 | Search query matches zero payloads | Display "No matches for: {query}" in status bar | — |
| EC-002 | Invalid regex (e.g., `[unclosed`) | Display "Invalid search pattern: {parse error}" in status bar; search not applied | — |
| EC-003 | Search in empty buffer | Display "No messages to search" | — |
| EC-004 | Query matches within a truncated message display | Highlight visible portion; expanding message shows all highlights | — |
| EC-005 | Very common pattern matching thousands of times | Show first N highlights (N = visible rows); total count accurate | — |
| EC-006 | Search regex with catastrophic backtracking potential | Timeout after 100ms per message; skip message and continue; show "Search timeout on N messages" | — |
| EC-007 | Binary/non-UTF8 content in payload | Search operates on UTF-8 representation; non-UTF8 bytes are not matched | — |
| EC-008 | Empty search query (just pressing Enter on `/`) | Cancel search mode; no filter applied | — |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-HP-001 | Search `/error` in buffer of 100 messages, 5 contain "error" | "5 matches in 5 messages"; matched messages shown with "error" highlighted |
| TV-HP-002 | Press `n` from first match | Navigate to second match; highlight moves |
| TV-HP-003 | Press `N` from first match | Navigate to last match (wrap-around) |
| TV-HP-004 | Toggle case-insensitive: `/i Error` | Matches "error", "Error", "ERROR" etc. |
| TV-HP-005 | Search `/tools/.*list` (regex) | Matches messages with methods like "tools/list", "tools/template_list" |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-EC-001 | Search `/xyzzy` with no matches | "No matches for: xyzzy" |
| TV-EC-002 | Search `/[bad` (invalid regex) | "Invalid search pattern: unclosed character class" |
| TV-EC-003 | Active filter + search: filter shows 50 messages, search matches 3 of those | "3 matches in 3 messages" (out of 50 filtered, not full buffer) |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-ERR-001 | Regex `(a+)+$` on large payload (catastrophic backtracking) | Timeout after 100ms; message skipped; "Search timeout on 1 messages" |
| TV-ERR-002 | Search while buffer is being written to (concurrent access) | Consistent read; search sees a snapshot (no torn reads) |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | For all search queries: result_set ⊆ filtered_view ⊆ full_buffer | Property-based test |
| VP-002 | Case-insensitive search matches are a superset of case-sensitive matches for same query | Property-based test |
| VP-003 | Match navigation visits every match exactly once before wrapping | Sequence test |
| VP-004 | Search with regex timeout does not crash or hang the UI | Fuzz test |

## Traceability

| Item | Reference |
|------|-----------|
| L2 Capability | CAP-010 (Traffic Filtering & Replay) |
| Related BCs | BC-4.10.001 (filtering — combined with search), BC-3.07.002 (search mode activation), BC-3.08.001 (message rendering — highlight integration), BC-3.08.005 (accessibility — highlight uses non-color) |
