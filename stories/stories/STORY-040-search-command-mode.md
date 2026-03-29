---
document_type: story
story_id: STORY-040
epic_id: EPIC-03
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-039]
blocks: []
behavioral_contracts: [BC-3.07.002]
verification_properties: [VP-012]
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-040: Search & Command Mode (/ and :)

## Narrative
- **As an** AI Platform Engineer
- **I want to** press `/` to filter content inline and `:` to access command actions
- **So that** I can quickly find items and perform global actions without leaving the keyboard

## Acceptance Criteria

### AC-001 (traces to BC-3.07.002 postcondition — search mode)
Pressing `/` in normal mode enters search mode. A search bar appears at the bottom of the focused pane. Typing filters the current pane's content in real-time (matches highlighted). `Esc` clears filter and returns to normal mode.
- **Test:** `test_BC_3_07_002_search_mode_inline_filter()`

### AC-002 (traces to BC-3.07.002 postcondition — command mode)
Pressing `:` in normal mode opens command mode. A command input bar appears. Supported commands: `:q` (quit), `:connect <server>`, `:disconnect`, `:clear` (clear traffic), `:audit <server>`. `Esc` cancels command mode.
- **Test:** `test_BC_3_07_002_command_mode_dispatch()`

### AC-003 (traces to BC-3.07.002 — search accessible from any pane)
`/` works from sidebar, content area, and metrics panel. Search is always scoped to the currently focused pane.
- **Test:** `test_BC_3_07_002_search_any_pane()`

### AC-004 (traces to BC-3.07.002 — search preserves pane navigation)
While in search mode, `j`/`k` still navigate through filtered results. `Enter` selects the focused filtered item.
- **Test:** `test_BC_3_07_002_search_mode_navigation()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| Search mode state | `forge-tui/src/state.rs` | Pure |
| Command mode parser | `forge-tui/src/commands.rs` | Pure |
| Command dispatcher | `forge-tui/src/app.rs` | Effectful |

## UX Screens
- All screens (search is global)

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Search with no matches | All items hidden, "No results" shown |
| EC-002 | Unknown command | Show error in command bar |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Search/command mode state | Pure | Mode flags in TuiState |
| Command parser | Pure | Text → TuiCommand enum |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~700 |
| BC-3.07.002 | ~400 |
| **Total** | **~1,100** |
| Agent context window | 200K |
| **Budget usage** | **~0.6%** |

## Tasks

1. [ ] Write failing tests for all 4 ACs
2. [ ] Extend TuiState with search_query and command_input fields
3. [ ] Implement command parser (text → TuiCommand)
4. [ ] Wire command dispatcher to TUI actions
5. [ ] Implement real-time filtering via filter_messages / search in panes
6. [ ] Verify Red Gate
7. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-039 | TuiAction enum established | Add SearchInput, CommandInput actions | Search state must be cleared on pane switch |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Pure command parser | purity-boundary-map.md | No I/O in parse_command() |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| (no new deps) | | | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-tui/src/state.rs` | Search + command mode fields | YES (from STORY-037) |
| `crates/forge-tui/src/commands.rs` | Command parser | NO — this story creates it |
