---
document_type: story
story_id: STORY-054
epic_id: EPIC-07
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-052, STORY-053, STORY-037]
blocks: []
behavioral_contracts: [BC-7.18.001]
verification_properties: []
priority: P1
assumption_validations: []
risk_mitigations: []
---

# STORY-054: Security Audit TUI View (SCR-007)

## Narrative
- **As an** AI Platform Engineer using the TUI
- **I want to** see security findings in a dedicated TUI view with severity indicators
- **So that** I can review and act on security issues during an interactive session

## Acceptance Criteria

### AC-001 (traces to BC-7.18.001 postcondition — findings list)
SCR-007 shows a scrollable table of findings sorted by severity (Critical first). Each row: `[CRIT]`/`[HIGH]`/`[MED]`/`[LOW]` badge, category, server name, brief description (truncated 60 chars). Badges use color + text (accessibility).
- **Test:** `test_BC_7_18_001_findings_table()`

### AC-002 (traces to BC-7.18.001 postcondition — finding detail)
Pressing `Enter` on a finding opens a detail view showing full description, OWASP category, confidence, affected tool name, and suppression option.
- **Test:** `test_BC_7_18_001_finding_detail_view()`

### AC-003 (traces to BC-7.18.001 — export)
Pressing `E` in the security view exports the current audit report as JSON to `~/.local/share/forge-mcp/audit-<timestamp>.json` and shows confirmation.
- **Test:** `test_BC_7_18_001_export_report()`

### AC-004 (traces to BC-7.18.001 — start audit)
Pressing `s` starts a new audit for the selected server (clears current findings, re-runs all detectors on buffered traffic).
- **Test:** `test_BC_7_18_001_start_audit_action()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `SecurityAuditView` widget | `forge-tui/src/widgets/security_view.rs` | Pure (render) |
| Export action | `forge-tui/src/app.rs` | Effectful |

## UX Screens
- SCR-007 (Security Audit View)
- FLOW-005 (Security Audit Workflow)

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | No findings | "No security findings" empty state |
| EC-002 | 500+ findings | Virtual scroll only visible rows |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| SecurityAuditView | Pure | Renders from Vec<Finding> |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~700 |
| BC-7.18.001 | ~400 |
| UX-INDEX.md severity indicators | ~400 |
| **Total** | **~1,500** |
| Agent context window | 200K |
| **Budget usage** | **~0.8%** |

## Tasks

1. [ ] Write failing tests for all 4 ACs
2. [ ] Implement SecurityAuditView widget
3. [ ] Implement finding detail expansion
4. [ ] Implement export to file
5. [ ] Implement start-audit action
6. [ ] Verify severity badges use text labels
7. [ ] Verify Red Gate
8. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-044 | Badge rendering established | Reuse badge renderer for severity badges | [CRIT] badges need highest contrast |
| STORY-052 | AuditReport JSON structure | Serialize for export | |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| No color-only severity indicators (NFR-015) | nfr-catalog.md | [CRIT]/[HIGH] text labels required |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| ratatui | >= 0.28 | Table widget | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-tui/src/widgets/security_view.rs` | SecurityAuditView | NO — this story creates it |
