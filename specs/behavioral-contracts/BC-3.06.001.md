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
capability: "CAP-006"
lifecycle_status: active
introduced: v0.1.0
---

# BC-3.06.001 — Multi-Pane Adaptive Layout (80×24 to Ultra-Wide)

## Summary

The TUI dashboard renders a multi-pane layout using ratatui that adapts from the minimum terminal size (80×24) to ultra-wide displays. The layout comprises five regions: server browser (left), capability explorer (center-top), traffic inspector (center-bottom), health metrics (right), and status bar (bottom). At minimum size, non-essential panes collapse; at ultra-wide, all panes expand with proportional sizing.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | Terminal supports at least 80 columns × 24 rows |
| PRE-002 | ratatui backend is initialized and connected to a valid terminal handle |
| PRE-003 | At least one server is configured in the server registry |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | All five layout regions are rendered without overlap or clipping at ≥80×24 |
| POST-002 | Status bar is always visible at the bottom row regardless of terminal size |
| POST-003 | At 80×24, only essential panes (server browser, traffic inspector, status bar) are visible |
| POST-004 | At ≥160 columns, all five panes are visible with proportional widths |
| POST-005 | Pane borders use Unicode box-drawing characters (with ASCII fallback per BC-3.06.002) |
| POST-006 | Each pane has a title header identifying its function |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | No two panes overlap in rendered screen coordinates |
| INV-002 | The sum of all pane widths equals the terminal width (no gaps) |
| INV-003 | The sum of all pane heights equals the terminal height (no gaps) |
| INV-004 | The status bar occupies exactly 1 row at the bottom |
| INV-005 | Render completes within a single frame tick (≤16ms at 60fps) |

## Edge Cases

| ID | Scenario | Expected Behavior | Source |
|----|----------|-------------------|--------|
| EC-001 | Terminal width < 80 or height < 24 | Display error message E-TUI-004 ("Terminal too small: minimum 80×24 required, got {w}×{h}") and refuse to render layout | E-TUI-004 |
| EC-002 | Terminal resized during active render cycle | Complete current frame, recalculate layout on next frame tick; no panic or partial render | FM-014 |
| EC-003 | Terminal resized from ultra-wide to minimum during active use | Collapse panes gracefully in priority order: health metrics → capability explorer → server browser detail | — |
| EC-004 | Slow server response blocks data fetch | TUI continues rendering with stale/empty data; never blocks render loop waiting for server I/O | DEC-005 |
| EC-005 | All panes have zero content (no servers, no traffic) | Render empty panes with placeholder text ("No servers configured", "No traffic captured") | — |
| EC-006 | Extremely tall terminal (e.g., 80×200) | Traffic inspector and capability explorer expand vertically; no wasted space | — |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-HP-001 | Terminal 120×40, 3 servers configured | Five-pane layout rendered: server browser ~25% width, center panes ~50% width, health metrics ~25% width, status bar 1 row |
| TV-HP-002 | Terminal 80×24, 1 server configured | Three-pane layout: server browser (collapsed to list), traffic inspector, status bar |
| TV-HP-003 | Terminal 200×50 (ultra-wide) | All five panes visible with proportional sizing; health metrics shows full sparklines |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-EC-001 | Terminal 79×24 | E-TUI-004 error displayed, no layout rendered |
| TV-EC-002 | Terminal 80×23 | E-TUI-004 error displayed, no layout rendered |
| TV-EC-003 | Resize from 200×50 to 80×24 mid-render | Layout collapses gracefully on next frame; no visual artifacts |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-ERR-001 | ratatui backend fails to initialize | Application exits with E-TUI-001 ("Failed to initialize terminal backend: {reason}") |
| TV-ERR-002 | Terminal handle becomes invalid during operation | Attempt reconnect; if fails, exit with E-TUI-002 |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | For all terminal sizes (w, h) where w ≥ 80 and h ≥ 24: no pane overlap exists in rendered frame | Property-based test |
| VP-002 | For all resize events: layout recalculates within 1 frame tick without panic | Fuzz test |
| VP-003 | Render loop never blocks on I/O operations (server fetch, traffic capture) | Async property test |
| VP-004 | Status bar is present in 100% of rendered frames | Invariant test |

## Traceability

| Item | Reference |
|------|-----------|
| L2 Capability | CAP-006 (TUI Dashboard) |
| Domain Invariant | DI — layout consistency |
| Failure Mode | FM-014 (resize during render) |
| Decision | DEC-005 (slow server doesn't hang TUI) |
| Error Code | E-TUI-004 (terminal too small) |
| Related BCs | BC-3.06.002 (color/character fallback), BC-3.08.001–005 (pane content) |
