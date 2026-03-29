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

# BC-4.10.001 — Traffic Filtering by Method, Direction, Time, and Content

## Summary

The traffic inspector provides composable filters that narrow the displayed message list without affecting the underlying capture buffer. Filters operate on four dimensions: method name (regex match), direction (client→server, server→client, or both), time range (absolute or relative), and content pattern (regex match against payload). Multiple filters combine with AND logic. Filters are applied to the view only — the capture buffer (BC-4.09.003) remains unmodified (DI-006).

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | Capture buffer contains at least one message (BC-4.09.001) |
| PRE-002 | Traffic inspector pane is visible or filter is being set via command mode (BC-3.07.002) |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Only messages matching ALL active filters are displayed |
| POST-002 | The capture buffer is not modified by any filter operation (DI-006) |
| POST-003 | Removing a filter immediately re-includes previously hidden messages |
| POST-004 | Active filters are displayed in the traffic inspector header (e.g., "Filters: method=tools/.*, dir=→") |
| POST-005 | Filter count indicator shows "Showing N of M messages" |
| POST-006 | Filters persist until explicitly cleared by the user |

## Filter Specifications

| Dimension | Syntax | Example | Match Logic |
|-----------|--------|---------|-------------|
| Method | `method:<regex>` | `method:tools/.*` | Regex match against JSON-RPC method name |
| Direction | `dir:<arrow>` | `dir:→` or `dir:←` | Exact match: → = client→server, ← = server→client |
| Time range | `time:<start>..<end>` | `time:14:00..14:30` | Messages with timestamp within the range (inclusive) |
| Content | `content:<regex>` | `content:error` | Regex match against full JSON payload (case-sensitive by default) |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | DI-006: Filtering never mutates the capture buffer |
| INV-002 | Multiple filters combine with AND logic (all must match for a message to appear) |
| INV-003 | Clearing all filters restores the full message list |
| INV-004 | Filter evaluation order does not affect results (commutativity of AND) |
| INV-005 | Filter application does not reorder messages (filtered view preserves wire order) |

## Edge Cases

| ID | Scenario | Expected Behavior | Source |
|----|----------|-------------------|--------|
| EC-001 | Filter matches zero messages | Display "No messages match current filters" with active filter summary | — |
| EC-002 | Invalid regex in method or content filter | Display "Invalid filter: {regex parse error}" in status bar; filter not applied | — |
| EC-003 | Time range where start > end | Display "Invalid time range: start must be before end"; filter not applied | — |
| EC-004 | Very broad filter (matches all messages) | Display all messages; functionally equivalent to no filter | — |
| EC-005 | New message arrives that matches active filter | New message appears in filtered view in real time | — |
| EC-006 | New message arrives that does not match active filter | New message captured to buffer but not shown in filtered view | — |
| EC-007 | Filter applied to 100,000+ message buffer | Filter evaluation completes within 100ms for responsive UI | — |
| EC-008 | Content filter with `.` (matches everything) | All messages shown (regex `.` matches any character in payload) | — |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-HP-001 | Buffer has 100 messages; filter `method:tools/.*` matches 30 | "Showing 30 of 100 messages"; header shows "Filters: method=tools/.*" |
| TV-HP-002 | Add `dir:→` to existing method filter | Results narrow further to only client→server tools/* calls |
| TV-HP-003 | `:filter time:14:00..14:05` | Only messages from 14:00:00 to 14:05:59 displayed |
| TV-HP-004 | `:filter content:"error"` | Only messages containing the string "error" in their payload |
| TV-HP-005 | Clear all filters | Full message list restored; "Showing 100 of 100 messages" |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-EC-001 | Filter `method:zzzzz` matches 0 messages | "No messages match current filters" |
| TV-EC-002 | Filter `method:[invalid` (bad regex) | Status bar: "Invalid filter: unclosed character class" |
| TV-EC-003 | Filter `time:15:00..14:00` | Status bar: "Invalid time range: start must be before end" |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-ERR-001 | Filter evaluation takes >500ms on large buffer | Display partial results with "Filtering..." indicator; complete in background | — |
| TV-ERR-002 | Regex causes catastrophic backtracking | Timeout regex evaluation after 100ms; report "Filter regex too complex" | — |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | For all filter combinations: filtered_set ⊆ full_buffer_set | Property-based test |
| VP-002 | DI-006: buffer contents unchanged before and after filter apply/remove cycle | Invariant test |
| VP-003 | Clearing all filters produces identical set to no-filter state | Equivalence test |
| VP-004 | AND composition: filter(A ∧ B) == filter(A) ∩ filter(B) | Property-based test |
| VP-005 | Filtered view preserves wire order from original capture | Sequence test |

## Traceability

| Item | Reference |
|------|-----------|
| L2 Capability | CAP-010 (Traffic Filtering & Replay) |
| Domain Invariant | DI-006 (wire order / capture immutability) |
| Related BCs | BC-4.09.001 (capture buffer — data source), BC-4.10.002 (full-text search), BC-3.07.002 (`:filter` command) |
