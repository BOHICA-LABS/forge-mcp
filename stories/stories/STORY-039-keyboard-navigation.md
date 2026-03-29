---
document_type: story
story_id: STORY-039
epic_id: EPIC-03
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-037]
blocks: [STORY-040, STORY-041]
behavioral_contracts: [BC-3.07.001]
verification_properties: [VP-012]
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-039: Vi-Style Keyboard Navigation

## Narrative
- **As an** AI Platform Engineer using the TUI
- **I want to** navigate panes and items using vi-style keys
- **So that** I can operate the TUI efficiently without reaching for the mouse

## Acceptance Criteria

### AC-001 (traces to BC-3.07.001 postcondition — hjkl navigation)
In normal mode: `h`/`←` shifts focus to left pane, `l`/`→` to right pane, `j`/`↓` moves down within focused pane, `k`/`↑` moves up within focused pane.
- **Test:** `test_BC_3_07_001_hjkl_navigation()`

### AC-002 (traces to BC-3.07.001 postcondition — Tab pane cycling)
`Tab` cycles focus forward through panes. `Shift+Tab` cycles backward. Focus indicator (double-line border) updates on each Tab press.
- **Test:** `test_BC_3_07_001_tab_pane_cycling()`

### AC-003 (traces to BC-3.07.001 postcondition — g/G shortcuts)
`g` / `Home` jumps to first item in focused list. `G` / `End` jumps to last item.
- **Test:** `test_BC_3_07_001_g_G_shortcuts()`

### AC-004 (traces to BC-3.07.001 postcondition — Enter and Esc)
`Enter` activates/selects the currently focused item. `Esc` cancels/closes current modal, or exits search mode, or exits command mode.
- **Test:** `test_BC_3_07_001_enter_esc_actions()`

### AC-005 (traces to BC-3.07.001 — q exits with confirmation)
`q`/`Q` in normal mode shows a confirmation prompt: "Quit Forge MCP? [y/N]". Pressing `y` exits. Pressing `N` or `Esc` cancels.
- **Test:** `test_BC_3_07_001_quit_confirmation()`

### AC-006 (traces to BC-3.07.001 — r refresh)
`r` in normal mode refreshes the current pane data (re-fetches from daemon).
- **Test:** `test_BC_3_07_001_refresh_key()`

### AC-007 (traces to BC-3.07.001 — Ctrl+C immediate exit)
`Ctrl+C` exits immediately without confirmation. Terminal restored before exit. (BC-3.07.001.)
- **Test:** `test_BC_3_07_001_ctrlc_immediate_exit()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| Key event handler | `forge-tui/src/input.rs` | Pure (maps key → TuiAction) |
| TuiAction enum | `forge-tui/src/state.rs` | Pure |
| Action dispatcher | `forge-tui/src/app.rs` | Effectful |

## UX Screens
- All screens use global keybindings
- SCR-001 (Main Dashboard) — pane focus

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | h on leftmost pane | No-op, no wrap |
| EC-002 | k on first item | No-op, no wrap to bottom |
| EC-003 | Tab with only 1 pane | No-op |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Key event handler | Pure | Maps crossterm Key → TuiAction enum |
| TuiAction dispatcher | Effectful | Mutates TuiState, triggers renders |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~900 |
| BC-3.07.001 | ~500 |
| UX-INDEX.md global keybindings | ~600 |
| **Total** | **~2,000** |
| Agent context window | 200K |
| **Budget usage** | **~1%** |

## Tasks

1. [ ] Write failing tests for all 7 ACs
2. [ ] Define `TuiAction` enum with all navigation actions
3. [ ] Implement `key_to_action()` pure mapping function
4. [ ] Implement action dispatcher (state mutation)
5. [ ] Implement quit confirmation modal
6. [ ] Implement Ctrl+C signal handler
7. [ ] Verify Red Gate
8. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-037 | TuiState has active_pane | Navigation mutates active_pane | crossterm key events are synchronous; must drain on resize |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Pure key mapping | purity-boundary-map.md | key_to_action() has no I/O |
| Full keyboard-only operation | BC-3.07.001, NFR-015 | Test with no mouse events |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| crossterm | >= 0.27 | Key event types | `crossterm::event::{KeyCode, KeyModifiers}` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-tui/src/input.rs` | Key → TuiAction mapping | NO — this story creates it |
| `crates/forge-tui/src/state.rs` | TuiAction enum | YES (from STORY-037) — add actions |
