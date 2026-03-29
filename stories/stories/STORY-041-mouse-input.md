---
document_type: story
story_id: STORY-041
epic_id: EPIC-03
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 3
depends_on: [STORY-039]
blocks: []
behavioral_contracts: [BC-3.07.003]
verification_properties: []
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-041: Mouse Supplementary Input

## Narrative
- **As an** AI Platform Engineer
- **I want to** use the mouse as a supplementary navigation tool in the TUI
- **So that** I can click on items and scroll without remembering all keybindings

## Acceptance Criteria

### AC-001 (traces to BC-3.07.003 postcondition — click to focus pane)
Clicking within a pane transfers keyboard focus to that pane (same as Tab navigation). The focused pane's border changes to double-line.
- **Test:** `test_BC_3_07_003_click_focuses_pane()`

### AC-002 (traces to BC-3.07.003 postcondition — click to select row)
Clicking on a row in a list widget selects that row (same as pressing Enter on it).
- **Test:** `test_BC_3_07_003_click_selects_row()`

### AC-003 (traces to BC-3.07.003 postcondition — scroll wheel)
Mouse scroll wheel up/down scrolls the focused widget (same as k/j).
- **Test:** `test_BC_3_07_003_scroll_wheel_navigation()`

### AC-004 (traces to BC-3.07.003 — mouse disable flag)
`--mouse false` CLI flag disables mouse entirely. Mouse events are not captured in this mode. Keyboard-only operation continues normally.
- **Test:** `test_BC_3_07_003_mouse_disabled_flag()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| Mouse event handler | `forge-tui/src/input.rs` | Pure (maps mouse → TuiAction) |
| Mouse mode config | `forge-tui/src/config.rs` | Pure |

## UX Screens
- All screens (supplementary)

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Mouse click outside all panes | No action |
| EC-002 | Mouse scroll in empty list | No-op |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Mouse event handler | Pure | Maps crossterm MouseEvent → TuiAction |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~600 |
| BC-3.07.003 | ~300 |
| **Total** | **~900** |
| Agent context window | 200K |
| **Budget usage** | **~0.5%** |

## Tasks

1. [ ] Write failing tests for all 4 ACs
2. [ ] Implement mouse event → TuiAction mapping
3. [ ] Implement mouse enable/disable config
4. [ ] Enable crossterm mouse capture when enabled
5. [ ] Verify Red Gate
6. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-039 | TuiAction enum covers all interactions | Mouse actions reuse same enum | crossterm mouse must be explicitly enabled |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Keyboard must work without mouse | BC-3.07.001 | Mouse is supplementary only |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| crossterm | >= 0.27 | Mouse event support | `crossterm::event::EnableMouseCapture` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-tui/src/input.rs` | Mouse event handling added | YES (from STORY-039) |
