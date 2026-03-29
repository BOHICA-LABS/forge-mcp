---
document_type: story
story_id: STORY-038
epic_id: EPIC-03
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 3
depends_on: [STORY-037]
blocks: []
behavioral_contracts: [BC-3.06.002]
verification_properties: []
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-038: Color System Auto-Detection & Degradation

## Narrative
- **As an** AI Platform Engineer on various terminals
- **I want to** have Forge MCP automatically detect and use the best available color depth
- **So that** the TUI looks great on modern terminals and still works on minimal ones

## Acceptance Criteria

### AC-001 (traces to BC-3.06.002 postcondition — truecolor detection)
When `$COLORTERM` == "truecolor" or "24bit", uses 24-bit color palette with full hex color tokens from UX-INDEX.md.
- **Test:** `test_BC_3_06_002_truecolor_detected()`

### AC-002 (traces to BC-3.06.002 — 256-color detection)
When `$TERM` contains "256color", uses 256-color xterm palette mappings from UX-INDEX.md.
- **Test:** `test_BC_3_06_002_256color_detected()`

### AC-003 (traces to BC-3.06.002 — 16-color fallback)
When `$TERM` is "xterm", "screen", etc. (no 256/truecolor indicator), uses 16-color ANSI palette with bold/dim modifiers.
- **Test:** `test_BC_3_06_002_16color_fallback()`

### AC-004 (traces to BC-3.06.002 — monochrome fallback)
When `$NO_COLOR` is set or `--color never`, uses monochrome mode. All color tokens map to plain white/black. Status indicators use glyph+text only.
- **Test:** `test_BC_3_06_002_no_color_monochrome()`

### AC-005 (traces to BC-3.06.002 — detection in < 1ms)
Color detection reads env vars synchronously. Completes in < 1ms. Detected once at startup, not per-frame.
- **Test:** Timing assertion in startup benchmark

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `detect_color_mode()` | `forge-tui/src/color.rs` | Pure (reads from injected env) |
| `ColorSystem` enum | `forge-tui/src/color.rs` | Pure |
| Color token resolver | `forge-tui/src/color.rs` | Pure |

## UX Screens
- All screens use color system

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Both $NO_COLOR and $COLORTERM set | $NO_COLOR wins |
| EC-002 | --color always flag | Force truecolor regardless of env |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| detect_color_mode() | Pure (with injected env) | Takes env map as parameter |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~600 |
| BC-3.06.002 | ~400 |
| UX-INDEX.md color palette | ~600 |
| **Total** | **~1,600** |
| Agent context window | 200K |
| **Budget usage** | **~0.8%** |

## Tasks

1. [ ] Write failing tests for all 5 ACs
2. [ ] Define `ColorSystem` enum (Truecolor, Color256, Color16, Monochrome)
3. [ ] Implement `detect_color_mode()` with injected env map
4. [ ] Implement color token resolver for all 4 color depths
5. [ ] Verify Red Gate
6. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-037 | TuiState holds color system | Pass ColorSystem to all widget renders | $COLORTERM vs $TERM detection order matters |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Color detection in < 1ms | UX-INDEX.md | Synchronous env read, no I/O |
| NO_COLOR respected | UX-INDEX.md | Always check NO_COLOR first |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| ratatui | >= 0.28 | Color types | `ratatui::style::Color` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-tui/src/color.rs` | ColorSystem + detection | NO — this story creates it |
