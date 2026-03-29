---
document_type: story
story_id: STORY-060
epic_id: EPIC-09
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 3
depends_on: [STORY-059]
blocks: []
behavioral_contracts: [BC-9.22.001]
verification_properties: []
priority: P2
assumption_validations: []
risk_mitigations: []
---

# STORY-060: Config Drift Reconciliation Workflow Guidance

## Narrative
- **As a** DevEx Engineer
- **I want to** receive actionable guidance on how to reconcile detected config drift
- **So that** I know exactly what to change and where

## Acceptance Criteria

### AC-001 (traces to BC-9.22.001 postcondition — reconciliation steps)
For each `DriftRecord` with `severity: Breaking`, the report includes `reconciliation_steps: Vec<String>` with human-readable guidance: "In Cursor (~/.cursor/mcp.json), change 'command' from 'node' to 'npx -y my-server' to match Claude Desktop config."
- **Test:** `test_BC_9_22_001_reconciliation_steps_breaking()`

### AC-002 (traces to BC-9.22.001 — advisory only, no write-back)
Reconciliation steps are purely advisory text. Forge MCP NEVER modifies config files. (DI-015.)
- **Test:** `test_BC_9_22_001_no_file_modification()`

### AC-003 (traces to BC-9.22.001 — CLI guidance output)
`forge-mcp info --drift --reconcile` outputs the reconciliation guidance as part of the drift report JSON, or as human-readable text with `--format text`.
- **Test:** `test_BC_9_22_001_cli_guidance_output()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `generate_reconciliation()` | `forge-config/src/reconcile.rs` | Pure |

## UX Screens
- SCR-009 (Config Drift View) — shows guidance inline

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Cosmetic drift only | No reconciliation steps (cosmetic is advisory) |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| generate_reconciliation() | Pure | DriftRecord → Vec<String> |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~600 |
| BC-9.22.001 | ~300 |
| **Total** | **~900** |
| Agent context window | 200K |
| **Budget usage** | **~0.5%** |

## Tasks

1. [ ] Write failing tests for all 3 ACs
2. [ ] Implement `generate_reconciliation()` pure function
3. [ ] Add guidance to DriftReport
4. [ ] Add --reconcile flag to CLI
5. [ ] Verify DI-015 (no file writes) via test
6. [ ] Verify Red Gate
7. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-059 | DriftRecord has source A/B and diff | Generate steps from diff | Guidance must be specific, not generic |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| DI-015: no config writes | BC-9.22.001 | Advisory only assertion |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| (no new deps) | | | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-config/src/reconcile.rs` | generate_reconciliation() | NO — this story creates it |
