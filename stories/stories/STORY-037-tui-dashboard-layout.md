---
document_type: story
story_id: STORY-037
epic_id: EPIC-03
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 8
depends_on: [STORY-006, STORY-013]
blocks: [STORY-038, STORY-039, STORY-040, STORY-041, STORY-042, STORY-043, STORY-044, STORY-045]
behavioral_contracts: [BC-3.06.001]
verification_properties: [VP-012]
priority: P0
assumption_validations: []
risk_mitigations: [R-005]
---

# STORY-037: TUI Main Dashboard Layout & Adaptive Panes

## Narrative
- **As an** AI Platform Engineer
- **I want to** launch the Forge MCP TUI and see an organized, adaptive dashboard
- **So that** I can monitor multiple servers and dimensions in one terminal window

> **SR-003 addressed here:** TUI frame rate testing uses `ratatui::backend::TestBackend` to count `draw()` calls per second with a synthetic event stream, avoiding real terminal dependency in CI.

## Acceptance Criteria

### AC-001 (traces to BC-3.06.001 postcondition — 80×24 minimum layout)
TUI renders without crash at 80×24 terminal size. Three-pane layout: left sidebar (22% width), center content area (54%), right metrics panel (24%). Below 80×24 shows "Terminal too small" message and no panes.
- **Test:** `test_BC_3_06_001_minimum_terminal_size()` using TestBackend

### AC-002 (traces to BC-3.06.001 postcondition — narrow layout)
At 80–100 cols, right health panel collapses into a tab within the content area. Left sidebar stays at 20 cols fixed.
- **Test:** `test_BC_3_06_001_narrow_layout_collapse()`

### AC-003 (traces to BC-3.06.001 postcondition — wide layout)
At > 160 cols, sidebar expands to 30 cols. Content area grows proportionally. Health panel expands to 28 cols.
- **Test:** `test_BC_3_06_001_wide_layout_expand()`

### AC-004 (traces to BC-3.06.001 — SIGWINCH resize)
When terminal is resized (SIGWINCH signal), TUI re-renders with new dimensions immediately. No stale layout from previous size.
- **Test:** `test_BC_3_06_001_resize_rerender()`

### AC-005 (traces to BC-3.06.001 — status bar always visible)
Status bar is always rendered at the bottom row: "[Daemon: OK] [Capture: ON] [nn msgs] [latency: NNms] [q:quit ?:help]". Never hidden regardless of terminal size.
- **Test:** `test_BC_3_06_001_status_bar_always_visible()`

### AC-006 (traces to BC-3.06.001 — 60fps with TestBackend — SR-003)
Using `ratatui::backend::TestBackend`, simulate 100 synthetic events/second and assert ≥ 60 `draw()` calls/second for 1 second. (NFR-002 validation method per SR-003.)
- **Test:** `test_BC_3_06_001_60fps_test_backend()` — SR-003 implementation

### AC-007 (traces to BC-3.06.001 — TUI state machine)
`TuiState` pure state machine manages: active pane (Sidebar|Content|Metrics), current mode (Normal|Search|Command|Input), active tab (Capabilities|Traffic|Health|Security). All states reachable, no dead states. (VP-012.)
- **Test:** Proptest for VP-012: all valid state transitions reachable

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `TuiState` state machine | `forge-tui/src/state.rs` | Pure |
| Layout calculator | `forge-tui/src/layout.rs` | Pure |
| TUI event loop | `forge-tui/src/app.rs` | Effectful |
| Terminal I/O | `forge-tui/src/terminal.rs` | Effectful |

## UX Screens
- SCR-001 (Main Dashboard Layout)
- Status bar (global, all screens)

## Design System Components

| Component | Contract | Variants | Required States | Async States |
|-----------|----------|----------|----------------|-------------|
| Panel widget | UX-INDEX.md §Panel | focused, unfocused | focused, unfocused | N/A |
| Status bar | UX-INDEX.md §StatusBar | normal, warning | connected, disconnected | N/A |
| Tab bar | UX-INDEX.md §TabBar | 4 tabs | active, inactive | N/A |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Terminal < 80×24 | "Terminal too small" full-screen message |
| EC-002 | Ultra-wide (220×60) | Panes expand proportionally, no overflow |
| EC-003 | SIGWINCH with ongoing render | Render completes, then resize applied |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| TuiState | Pure | State transitions, no I/O |
| Layout calculator | Pure | Math on terminal dimensions |
| TUI event loop | Effectful | crossterm terminal I/O |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~1,200 |
| BC-3.06.001 | ~700 |
| UX-INDEX.md (layout section) | ~800 |
| VP-012 | ~300 |
| ratatui TestBackend API | ~400 |
| **Total** | **~3,400** |
| Agent context window | 200K |
| **Budget usage** | **~1.7%** |

## Tasks

1. [ ] Write failing tests using TestBackend (all 7 ACs)
2. [ ] Define `TuiState` pure state machine
3. [ ] Implement layout calculator (pure math)
4. [ ] Implement 3-pane layout with responsive collapse
5. [ ] Implement status bar widget
6. [ ] Implement resize handling (SIGWINCH)
7. [ ] Implement 60fps event loop using TestBackend for CI (SR-003)
8. [ ] Add proptest for VP-012 TUI state machine
9. [ ] Wire up TUI entry point in forge-mcp `tui` subcommand
10. [ ] Verify Red Gate
11. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-006 | ServerRegistry available | Pass to TUI as data source | crossterm must be initialized after clap parsing |
| STORY-013 | ServerCapabilities available | Pass connection state to TUI | ratatui requires terminal in raw mode |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| ratatui + crossterm (AD-007) | ARCH-INDEX.md | No other TUI framework |
| TUI state machine pure core | purity-boundary-map.md | TuiState has no crossterm types |
| NFR-002: 60fps with < 5MB RSS | nfr-catalog.md | TestBackend benchmark in CI |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| ratatui | >= 0.28 | TUI framework (AD-007) | `use ratatui::{Terminal, Frame}` |
| crossterm | >= 0.27 | Terminal backend (AD-007) | `use crossterm::terminal::*` |
| tokio | >= 1.38 | Async event loop | `tokio::select!` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-tui/src/state.rs` | TuiState pure state machine | NO — this story creates it |
| `crates/forge-tui/src/layout.rs` | Layout calculator | NO |
| `crates/forge-tui/src/app.rs` | TUI event loop | NO |
| `crates/forge-tui/src/terminal.rs` | Terminal setup/teardown | NO |
| `crates/forge-tui/src/widgets/panel.rs` | Panel widget | NO |
| `crates/forge-tui/proofs/tui_state.rs` | VP-012 proptest | NO |
| `crates/forge-mcp/src/commands/tui.rs` | `forge-mcp tui` subcommand | NO |
