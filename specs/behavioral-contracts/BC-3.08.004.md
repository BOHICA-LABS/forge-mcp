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

# BC-3.08.004 — Capability Explorer (Tools/Resources/Prompts Tree)

## Summary

The capability explorer pane displays a tree view of the selected server's capabilities, grouped by type: Tools, Resources, and Prompts. Selecting a tool shows its input schema; selecting a resource shows its URI template; selecting a prompt shows its argument schema. The explorer is pagination-aware: if the server reports paginated results, the explorer shows the total count and loads additional pages on demand.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | Capability explorer pane is visible in the layout (BC-3.06.001) |
| PRE-002 | A server is selected in the server browser (BC-3.08.003) |
| PRE-003 | Selected server is in Connected state |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Tree root shows three groups: "Tools (N)", "Resources (N)", "Prompts (N)" with counts |
| POST-002 | Expanding a group shows the individual capabilities as child nodes |
| POST-003 | Selecting a tool displays its JSON Schema input definition in a detail area |
| POST-004 | Selecting a resource displays its URI template and description |
| POST-005 | Selecting a prompt displays its arguments and description |
| POST-006 | If capabilities are paginated, header shows "Tools (N of M total)" and offers "Load more" at end of list |
| POST-007 | Tree state (expanded/collapsed groups) persists when switching back to this server |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | Capability counts match the actual number of items received from the server |
| INV-002 | Tree groups are always in order: Tools, Resources, Prompts |
| INV-003 | Capability data displayed matches the server's tools/list, resources/list, prompts/list responses |
| INV-004 | Loading additional pages does not lose or reorder already-loaded items |

## Edge Cases

| ID | Scenario | Expected Behavior | Source |
|----|----------|-------------------|--------|
| EC-001 | Server has zero capabilities (no tools, resources, or prompts) | Show all three groups with count 0: "Tools (0)", "Resources (0)", "Prompts (0)" | — |
| EC-002 | Server disconnects while explorer is showing its capabilities | Display stale data with "(disconnected)" label; disable "Load more" | — |
| EC-003 | Tool schema is deeply nested or very large | Render schema to configurable depth limit (default 5 levels); truncate with indicator | — |
| EC-004 | Server returns 500+ tools (paginated) | Show first page immediately; "Load more (showing 50 of 500)" at bottom | — |
| EC-005 | Switch to different server while previous server's capabilities are still loading | Cancel pending requests for previous server; start loading new server's capabilities | — |
| EC-006 | Capability name contains special characters or emoji | Render as-is using terminal's character support | — |
| EC-007 | Server's capabilities change while explorer is open (tools added/removed) | Refresh on explicit user action (`:refresh` command or keybinding `r`); no auto-refresh to avoid disruption | — |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-HP-001 | Server with 5 tools, 3 resources, 2 prompts | Tree: `▶ Tools (5)`, `▶ Resources (3)`, `▶ Prompts (2)` |
| TV-HP-002 | Expand Tools group, select "read_file" tool | Child list shows 5 tools; detail area shows `read_file` schema: `{path: string (required), encoding: string (optional)}` |
| TV-HP-003 | Server with paginated tools (50 per page, 120 total) | `▶ Tools (50 of 120)` with "Load more" at end of expanded list |
| TV-HP-004 | Click "Load more" | Additional 50 tools loaded; `▶ Tools (100 of 120)` |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-EC-001 | Server with 0 tools, 0 resources, 0 prompts | Three groups all showing "(0)"; no expandable children |
| TV-EC-002 | Server disconnects mid-view | Tree shows stale data with "(disconnected)" indicator |
| TV-EC-003 | Switch server while loading | Loading spinner stops; new server's data begins loading |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-ERR-001 | tools/list request fails with server error | "Tools" group shows "Error loading tools: {reason}"; other groups load independently |
| TV-ERR-002 | Schema parsing fails for a tool | Tool listed normally; detail view shows "Schema unavailable: parse error" |
| TV-ERR-003 | "Load more" request fails | Show "Failed to load next page. Press Enter to retry." at list end |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | For all servers: displayed capability counts match server-reported counts | Property-based test |
| VP-002 | Pagination loading is idempotent: loading the same page twice does not duplicate items | Invariant test |
| VP-003 | Server switch cancels all pending requests for the previous server | State machine test |
| VP-004 | Tree navigation (expand/collapse/select) works correctly for N capabilities where N ∈ [0, 1000] | Stress test |

## Traceability

| Item | Reference |
|------|-----------|
| L2 Capability | CAP-008 (TUI Data Display) |
| Related BCs | BC-3.08.003 (server browser — provides selected server), BC-3.07.001 (keyboard nav for tree), BC-3.08.005 (accessibility) |
