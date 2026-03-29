---
document_type: story
story_id: STORY-045
epic_id: EPIC-03
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-037, STORY-016, STORY-017, STORY-018]
blocks: []
behavioral_contracts: [BC-3.08.004, BC-3.08.005]
verification_properties: []
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-045: Capability Explorer (Tools/Resources/Prompts Tree)

## Narrative
- **As an** AI Platform Engineer
- **I want to** browse all capabilities of a connected server in a tabbed view
- **So that** I can see what tools, resources, and prompts are available and execute them

## Acceptance Criteria

### AC-001 (traces to BC-3.08.004 postcondition — tab-based explorer)
SCR-003 displays three tabs: Tools, Resources, Prompts. Each tab shows a table of the server's capabilities. `[`/`]` and `1`/`2`/`3` switch tabs.
- **Test:** `test_BC_3_08_004_tab_navigation()`

### AC-002 (traces to BC-3.08.004 postcondition — tools table)
Tools tab shows: name, description (truncated to 60 chars), first required argument (if any). Total count shown in tab title: "Tools (42)".
- **Test:** `test_BC_3_08_004_tools_table()`

### AC-003 (traces to BC-3.08.004 postcondition — tool execution)
Pressing `e` on a selected tool opens SCR-006 (Tool Execution Dialog) pre-filled with the tool's input schema for argument entry.
- **Test:** `test_BC_3_08_004_tool_execution_trigger()`

### AC-004 (traces to BC-3.08.004 postcondition — accessibility)
Each row in the tools table has sufficient text context that capability is identifiable without color. Severity/type indicators use text labels not just colors. (BC-3.08.005.)
- **Test:** Monochrome mode render test

### AC-005 (traces to BC-3.08.004 — empty state)
When a server doesn't advertise a capability (e.g., no resources), the tab shows "Server does not expose resources" empty state.
- **Test:** `test_BC_3_08_004_empty_capability_state()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `CapabilityExplorer` widget | `forge-tui/src/widgets/capability_explorer.rs` | Pure |
| `TabBar` widget | `forge-tui/src/widgets/tab_bar.rs` | Pure |
| Tool execution trigger | `forge-tui/src/app.rs` | Effectful |

## UX Screens
- SCR-003 (Capability Browser)
- SCR-006 (Tool Execution Dialog)
- FLOW-002 (Tool Discovery & Execution)

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | 1000+ tools | Virtual scrolling, only visible rows rendered |
| EC-002 | Tool with no description | Empty description column |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| CapabilityExplorer | Pure | Renders from Vec<Tool>/Vec<Resource>/Vec<Prompt> |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~800 |
| BC-3.08.004, BC-3.08.005 | ~600 |
| UX-INDEX.md tab bar widget | ~400 |
| **Total** | **~1,800** |
| Agent context window | 200K |
| **Budget usage** | **~0.9%** |

## Tasks

1. [ ] Write failing tests for all 5 ACs
2. [ ] Implement TabBar widget
3. [ ] Implement CapabilityExplorer with 3 tabs
4. [ ] Implement tool execution trigger → SCR-006
5. [ ] Implement empty state display
6. [ ] Verify accessibility (monochrome mode)
7. [ ] Verify Red Gate
8. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-016 | Tool list returns Vec<Tool> | Feed directly to widget | list_tools() may be slow; show loading indicator |
| STORY-039 | `e` key added for tool execution | TuiAction::ExecuteTool | |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Accessibility: no color-only (BC-3.08.005, NFR-015) | UX-INDEX.md | Text labels in all rows |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| ratatui | >= 0.28 | Table, Tabs widgets | `ratatui::widgets::{Table, Tabs}` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-tui/src/widgets/capability_explorer.rs` | CapabilityExplorer | NO — this story creates it |
| `crates/forge-tui/src/widgets/tab_bar.rs` | TabBar | NO |
| `crates/forge-tui/src/widgets/tool_execution.rs` | Tool execution dialog | NO |
