---
document_type: story
story_id: STORY-042
epic_id: EPIC-03
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-037, STORY-027]
blocks: []
behavioral_contracts: [BC-3.08.001]
verification_properties: []
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-042: JSON-RPC Syntax-Highlighted Message Rendering

## Narrative
- **As an** AI Platform Engineer
- **I want to** see JSON-RPC messages rendered with syntax highlighting in the TUI
- **So that** I can quickly read and understand message structure

## Acceptance Criteria

### AC-001 (traces to BC-3.08.001 postcondition — syntax highlighting)
JSON objects in the traffic inspector are rendered with syntax highlighting per UX-INDEX.md `syntax.*` tokens: keys in blue, strings in green, numbers in orange, booleans in purple, null in muted, brackets in cyan. In monochrome mode, all text is same color.
- **Test:** `test_BC_3_08_001_syntax_highlighting_tokens()`

### AC-002 (traces to BC-3.08.001 postcondition — 2-space indentation)
JSON is rendered with 2-space indentation in the expanded view. Collapsed view shows the raw single-line JSON (truncated to terminal width).
- **Test:** `test_BC_3_08_001_indentation_format()`

### AC-003 (traces to BC-3.08.001 postcondition — truncation with expand)
Long values are truncated with `…` (U+2026). Pressing `Enter`/`→` expands the value. `←` collapses it.
- **Test:** `test_BC_3_08_001_truncation_expand_collapse()`

### AC-004 (traces to BC-3.08.001 postcondition — scrollable)
`j`/`k` scroll line-by-line through the JSON viewer. The viewer shows a scrollbar indicator on the right.
- **Test:** `test_BC_3_08_001_scrollable_viewer()`

### AC-005 (traces to BC-3.08.001 — accessibility: no color-only)
Every highlighted element also has positional/structural distinction: keys are left-aligned followed by `:`, values follow `:` or `,`. Structure is parseable without color. (BC-3.08.005.)
- **Test:** Manual accessibility review

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `JsonViewer` widget | `forge-tui/src/widgets/json_viewer.rs` | Pure (render to Frame) |
| Syntax tokenizer | `forge-tui/src/syntax.rs` | Pure |

## UX Screens
- SCR-004 (Traffic Inspector) — message detail pane
- SCR-003 (Capability Browser) — schema display

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Non-JSON payload (notification with string result) | Render as plain text |
| EC-002 | Deeply nested JSON (20+ levels) | Render up to depth 10, truncate with "[...]" |
| EC-003 | Unicode in JSON values | Rendered correctly, no corruption |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| JsonViewer | Pure | Renders to ratatui Frame, no I/O |
| Syntax tokenizer | Pure | Token stream from JSON value |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~800 |
| BC-3.08.001 | ~500 |
| UX-INDEX.md widget contracts | ~600 |
| **Total** | **~1,900** |
| Agent context window | 200K |
| **Budget usage** | **~1%** |

## Tasks

1. [ ] Write failing tests for all 5 ACs
2. [ ] Implement syntax tokenizer (pure: JSON value → Vec<Token>)
3. [ ] Implement JsonViewer widget rendering
4. [ ] Implement truncation + expand/collapse
5. [ ] Implement scrollbar
6. [ ] Verify accessibility (no color-only)
7. [ ] Verify Red Gate
8. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-038 | ColorSystem determines rendering depth | Pass ColorSystem to syntax tokenizer | ratatui Span uses Style, not raw ANSI |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Color + glyph for accessibility (NFR-015) | nfr-catalog.md | Structural distinction without color |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| ratatui | >= 0.28 | Span, Style, Widget | `ratatui::widgets::*` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-tui/src/widgets/json_viewer.rs` | JsonViewer widget | NO — this story creates it |
| `crates/forge-tui/src/syntax.rs` | Syntax tokenizer | NO |
