---
document_type: story
story_id: STORY-044
epic_id: EPIC-03
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-037, STORY-006]
blocks: []
behavioral_contracts: [BC-3.08.003]
verification_properties: []
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-044: Server Browser with Status Badges

## Narrative
- **As an** AI Platform Engineer
- **I want to** see all discovered servers with their connection status in the TUI sidebar
- **So that** I can quickly connect/disconnect and see the health of each server

## Acceptance Criteria

### AC-001 (traces to BC-3.08.003 postcondition — server list with badges)
The server sidebar (SCR-002) shows each server from `ServerRegistry` with: server name, connection status badge (Connected ● CONN / Disconnected ○ DISC / Error ✗ ERR / Connecting ◌ BUSY) per UX-INDEX.md badge definitions.
- **Test:** `test_BC_3_08_003_server_list_badges()`

### AC-002 (traces to BC-3.08.003 — badge is glyph + text + color)
Every status badge has all three: a glyph (●/○/✗/◌), a text label (CONN/DISC/ERR/BUSY), AND a color. No color-only badge. (BC-3.08.005, NFR-015.)
- **Test:** `test_BC_3_08_003_badge_glyph_text_color()`

### AC-003 (traces to BC-3.08.003 — connect/disconnect)
Pressing `Enter` on a disconnected server connects it (triggers `connect_stdio` or `connect_http`). Pressing `Enter` on a connected server disconnects it. Badge updates in real-time.
- **Test:** `test_BC_3_08_003_connect_disconnect_action()`

### AC-004 (traces to BC-3.08.003 — inline search)
`/` in the server sidebar filters servers by name in real-time. Matching servers remain visible; non-matching are hidden.
- **Test:** `test_BC_3_08_003_server_search_filter()`

### AC-005 (traces to BC-3.08.003 — keyboard add/delete)
`n` opens "Add Server" dialog (manual entry). `d` prompts confirmation and removes the selected server from the registry.
- **Test:** `test_BC_3_08_003_add_delete_server()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| Server browser widget | `forge-tui/src/widgets/server_browser.rs` | Pure (render) |
| Badge renderer | `forge-tui/src/widgets/badge.rs` | Pure |
| Connect action handler | `forge-tui/src/app.rs` | Effectful |

## UX Screens
- SCR-002 (Server Sidebar)
- FLOW-001 (Server Connection)

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | No servers discovered | "No servers found" empty state message |
| EC-002 | ASCII terminal (no Unicode) | ● → *, ○ → o, ✗ → X, ◌ → . |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Server browser widget | Pure | Renders ServerRegistry, no I/O |
| Badge renderer | Pure | Status → styled text |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~800 |
| BC-3.08.003 | ~400 |
| UX-INDEX.md status badges | ~400 |
| **Total** | **~1,600** |
| Agent context window | 200K |
| **Budget usage** | **~0.8%** |

## Tasks

1. [ ] Write failing tests for all 5 ACs
2. [ ] Implement badge renderer (status → glyph+text+color)
3. [ ] Implement server browser list widget
4. [ ] Wire connect/disconnect actions to forge-core
5. [ ] Implement inline search
6. [ ] Implement add/delete actions
7. [ ] Verify Red Gate
8. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-006 | ServerRegistry is the data source | Pass registry to widget | Status requires live connection state from STORY-007 |
| STORY-039 | Navigation actions established | n/d added as new actions | |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Badge must have glyph+text+color (BC-3.08.005) | UX-INDEX.md | Never color-only |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| ratatui | >= 0.28 | List widget | `ratatui::widgets::List` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-tui/src/widgets/server_browser.rs` | Server list widget | NO — this story creates it |
| `crates/forge-tui/src/widgets/badge.rs` | Badge renderer | NO |
