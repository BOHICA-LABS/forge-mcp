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

# BC-3.08.003 — Server Browser with Status Badges

## Summary

The server browser pane lists all MCP servers from the configuration registry. Each server entry displays a status badge combining a symbol and text label (for accessibility per BC-3.08.005), the server name, transport type, and source editor. Status badges reflect the real-time connection state of each server.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | Server browser pane is visible in the layout (BC-3.06.001) |
| PRE-002 | Server registry is loaded from configuration |
| PRE-003 | Connection state is available for each registered server |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | All servers from the configuration registry are listed in the server browser |
| POST-002 | Each server entry shows: `[badge] server-name (transport) [source]` |
| POST-003 | Connected servers display: `● Connected [✓]` badge in green |
| POST-004 | Disconnected servers display: `○ Disconnected [—]` badge in gray |
| POST-005 | Error-state servers display: `✕ Error [!]` badge in red |
| POST-006 | Transport type is shown (stdio, sse, streamable-http) |
| POST-007 | Source editor is shown if available (e.g., "VS Code", "Cursor") |
| POST-008 | Status badges update within 1 second of connection state change |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | Every server in the registry appears in the browser (no silent omissions) |
| INV-002 | Badge color is always paired with a text/shape indicator (BC-3.08.005, NFR-015) |
| INV-003 | Server list order is stable (sorted alphabetically by name unless user-sorted) |
| INV-004 | Selecting a server in the browser updates the capability explorer (BC-3.08.004) |

## Edge Cases

| ID | Scenario | Expected Behavior | Source |
|----|----------|-------------------|--------|
| EC-001 | Zero servers in configuration | Display "No servers configured. Use :connect or add to config file." | — |
| EC-002 | Server transitions from Connected to Error | Badge updates from `● [✓]` green to `✕ [!]` red within 1 second | — |
| EC-003 | 100+ servers in registry | Scrollable list with virtual rendering; show total count in pane header | — |
| EC-004 | Server name is very long (>50 chars) | Truncate with "..." to fit column width; full name shown on selection | — |
| EC-005 | Configuration reloaded while browser is displayed | List updates to reflect new config; selection preserved if server still exists | — |
| EC-006 | Server source editor is unknown | Show "unknown" for source field | — |
| EC-007 | Multiple servers with same name | Display all; disambiguate with transport type or index | — |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-HP-001 | 3 servers: local-stdio (connected), remote-sse (disconnected), dev-http (error) | Three rows: `● Connected [✓] local-stdio (stdio)`, `○ Disconnected [—] remote-sse (sse)`, `✕ Error [!] dev-http (streamable-http)` |
| TV-HP-002 | Select second server with j/Enter | Capability explorer updates to show remote-sse's capabilities |
| TV-HP-003 | Server transitions connected → disconnected | Badge updates within 1s; no list reorder |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-EC-001 | Empty server registry | "No servers configured" message in pane |
| TV-EC-002 | Server name "my-very-long-development-server-name-for-testing-purposes" | Truncated to fit: "my-very-long-development-ser..." |
| TV-EC-003 | 150 servers registered | Scrollable list with header "Servers (150)" |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-ERR-001 | Configuration file is corrupted/unreadable | Show "Error loading server registry: {reason}" in pane |
| TV-ERR-002 | Connection state query times out for a server | Show `? Unknown [?]` badge in yellow; retry on next tick |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | For all servers in registry: server appears in browser list | Invariant test |
| VP-002 | For all status badges: text/shape label accompanies color (accessibility) | Property-based test |
| VP-003 | Status badge update latency ≤ 1 second after connection state change | Timing test |
| VP-004 | Server list renders correctly for N servers where N ∈ [0, 1000] | Stress test |

## Traceability

| Item | Reference |
|------|-----------|
| L2 Capability | CAP-008 (TUI Data Display) |
| Related BCs | BC-3.08.004 (capability explorer — updated on selection), BC-3.08.005 (accessibility), BC-3.07.001 (keyboard navigation) |
| NFR | NFR-015 (accessibility — no color-only indicators) |
