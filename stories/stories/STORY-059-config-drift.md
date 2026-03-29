---
document_type: story
story_id: STORY-059
epic_id: EPIC-09
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 8
depends_on: [STORY-006]
blocks: [STORY-060]
behavioral_contracts: [BC-9.21.001, BC-9.21.002]
verification_properties: []
priority: P1
assumption_validations: []
risk_mitigations: []
---

# STORY-059: Cross-Editor Config Comparison & Drift Report

## Narrative
- **As a** DevEx Engineer managing MCP configs across teams
- **I want to** see differences between MCP server configurations across editors
- **So that** I can identify and fix silent config drift before it causes problems

## Acceptance Criteria

### AC-001 (traces to BC-9.21.001 postcondition — cross-editor comparison)
`compare_configs(registry_a, registry_b) -> DriftReport` compares two `ServerRegistry` instances (from different editors or environments) and identifies: servers present in A only, servers present in B only, servers present in both with different configs.
- **Test:** `test_BC_9_21_001_cross_editor_comparison()`

### AC-002 (traces to BC-9.21.001 postcondition — multi-source comparison)
`forge-mcp info --drift` compares all discovered config sources in the current `ServerRegistry.conflicts` list. Each conflict is a drift record.
- **Test:** `test_BC_9_21_001_multi_source_drift_command()`

### AC-003 (traces to BC-9.21.002 postcondition — drift report detail)
The `DriftReport` contains for each drifted server: server name, source A (editor, path, config), source B (editor, path, config), diff summary (which fields differ: command, url, args, env, enabled), severity (None|Cosmetic|Breaking).
- **Test:** `test_BC_9_21_002_drift_report_detail()`

### AC-004 (traces to BC-9.21.002 postcondition — breaking vs cosmetic)
`Breaking` drift: different transport (stdio vs HTTP), different command, different URL. `Cosmetic` drift: different args, different env vars, different enabled state. Classified in the report.
- **Test:** `test_BC_9_21_002_drift_severity_classification()`

### AC-005 (traces to BC-9.21.001 — SCR-009 side-by-side view)
`forge-mcp info --drift --tui` opens SCR-009 (Config Drift View) showing side-by-side config diff with highlighted changes. Changed fields shown in yellow/amber, absent fields shown in red.
- **Test:** TUI rendering test with TestBackend

### AC-006 (traces to BC-9.21.002 — JSON export)
`forge-mcp info --drift --format json` outputs `DriftReport` as JSON to stdout.
- **Test:** `test_BC_9_21_002_json_export()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `compare_configs()` | `forge-config/src/diff.rs` | Pure |
| `DriftReport` | `forge-config/src/types.rs` | Pure |
| `DriftSeverity` classifier | `forge-config/src/diff.rs` | Pure |
| SCR-009 TUI view | `forge-tui/src/widgets/config_drift.rs` | Pure (render) |

## UX Screens
- SCR-009 (Config Drift View)

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | All configs identical | DriftReport with zero diffs |
| EC-002 | Three-way comparison | Pairwise comparisons: A vs B, A vs C, B vs C |
| EC-003 | Server only in one source | Classified as "missing" not "drifted" |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| compare_configs() | Pure | Fully pure: forge-config has no effectful operations |
| DriftReport | Pure | Data structure |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~1,100 |
| BC-9.21.001, BC-9.21.002 | ~800 |
| **Total** | **~1,900** |
| Agent context window | 200K |
| **Budget usage** | **~1%** |

## Tasks

1. [ ] Write failing tests for all 6 ACs
2. [ ] Define `DriftReport`, `DriftRecord`, `DriftSeverity` types
3. [ ] Implement `compare_configs()` pure function
4. [ ] Implement drift severity classification
5. [ ] Implement CLI `--drift` flag in `forge-mcp info`
6. [ ] Implement SCR-009 TUI widget
7. [ ] Verify Red Gate
8. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-006 | ServerRegistry with ConflictRecord | ConflictRecord is the input to drift detection | forge-config is fully pure (purity-boundary-map) |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| forge-config is fully pure | purity-boundary-map.md | No I/O in any forge-config function |
| forge-config L2 depends on forge-discovery + forge-core | dependency-graph.md | |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| (no new deps) | | | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-config/src/types.rs` | DriftReport, DriftRecord, DriftSeverity | NO — this story creates it |
| `crates/forge-config/src/diff.rs` | compare_configs() | NO |
| `crates/forge-tui/src/widgets/config_drift.rs` | SCR-009 widget | NO |
