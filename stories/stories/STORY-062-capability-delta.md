---
document_type: story
story_id: STORY-062
epic_id: EPIC-10
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 3
depends_on: [STORY-013]
blocks: []
behavioral_contracts: [BC-10.24.001]
verification_properties: []
priority: P2
assumption_validations: []
risk_mitigations: []
---

# STORY-062: Capability Set Delta Comparison

## Narrative
- **As an** MCP Server Author
- **I want to** compare capability sets between two server instances
- **So that** I can see what features were added or removed

## Acceptance Criteria

### AC-001 (traces to BC-10.24.001 postcondition — capability delta)
`diff_capabilities(caps_a, caps_b) -> CapabilityDelta` shows which capabilities are: in A only (removed), in B only (added), in both (common).
- **Test:** `test_BC_10_24_001_capability_delta()`

### AC-002 (traces to BC-10.24.001 — included in diff output)
`forge-mcp diff <server-a> <server-b>` includes capability delta in its output alongside tool schema diff.
- **Test:** `test_BC_10_24_001_in_diff_output()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `diff_capabilities()` | `forge-config/src/schema_diff.rs` | Pure |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| diff_capabilities() | Pure | Set operations on capabilities |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~400 |
| BC-10.24.001 | ~300 |
| **Total** | **~700** |
| Agent context window | 200K |
| **Budget usage** | **~0.4%** |

## Tasks

1. [ ] Write failing tests for both ACs
2. [ ] Implement `diff_capabilities()`
3. [ ] Wire into CLI diff command (STORY-061)
4. [ ] Verify Red Gate
5. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-061 | schema_diff.rs created | Add diff_capabilities() to same file | |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| forge-config fully pure | purity-boundary-map.md | |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| (no new deps) | | | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-config/src/schema_diff.rs` | diff_capabilities() added | YES (from STORY-061) |
