---
document_type: story
story_id: STORY-015
epic_id: EPIC-02
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
behavioral_contracts: [BC-2.04.003]
verification_properties: [VP-001]
priority: P0
assumption_validations: []
risk_mitigations: [R-002]
---

# STORY-015: Graceful Degradation with Older Spec Versions

## Narrative
- **As an** AI Platform Engineer running older MCP servers
- **I want to** have Forge MCP work with servers implementing older MCP spec versions
- **So that** I can inspect legacy servers without upgrading them

## Acceptance Criteria

### AC-001 (traces to BC-2.04.003 postcondition — version detection)
After `initialize`, the negotiated protocol version is stored on `McpConnection`. Both `2024-11-05` and `2025-11-25` versions are supported. (Traces to NFR-011.)
- **Test:** `test_BC_2_04_003_version_detection()`

### AC-002 (traces to BC-2.04.003 — version mismatch warning)
When the server responds with a protocol version differing from the client's proposed version, emits warning `E-CON-006` but continues. The negotiated version governs subsequent behavior.
- **Test:** `test_BC_2_04_003_version_mismatch_warning()`

### AC-003 (traces to BC-2.04.003 — method gating by version)
Methods not available in older spec versions (e.g., `elicitation/create` absent in `2024-11-05`) return `Err(E-PRO-003)` rather than being sent. Version-specific method availability is enforced via the capability guard from STORY-013.
- **Test:** `test_BC_2_04_003_method_gated_by_version()`

### AC-004 (traces to BC-2.04.003 — unknown version rejected)
If the server responds with a version string that is neither `2024-11-05` nor `2025-11-25`, logs a warning but continues with the highest-known spec semantics.
- **Test:** `test_BC_2_04_003_unknown_version_continues()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `SpecVersion` enum | `forge-core/src/types.rs` | Pure |
| Version-method gate | `forge-core/src/connection.rs` | Pure |

## UX Screens
- N/A — protocol story

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Server sends version "2024-11-05" | Full degradation to 2024 method set |
| EC-002 | Future version "2026-01-01" | Log warning, use 2025-11-25 semantics |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| SpecVersion | Pure | Enum with comparison |
| Version-method gate | Pure | Lookup table, no I/O |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~600 |
| BC-2.04.003 | ~400 |
| NFR-011 | ~200 |
| **Total** | **~1,200** |
| Agent context window | 200K |
| **Budget usage** | **~0.6%** |

## Tasks

1. [ ] Write failing tests for all 4 ACs
2. [ ] Define `SpecVersion` enum
3. [ ] Implement version parsing from initialize response
4. [ ] Add version-based method gate (extends capability guard from STORY-013)
5. [ ] Verify Red Gate
6. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-013 | Capability guard established | Extend with version dimension | Version is a second gate after capability |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Support ≥ 2 spec versions (NFR-011) | nfr-catalog.md | Test with both version mock servers |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| (no new deps) | | | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-core/src/types.rs` | SpecVersion enum | YES — add to types |
| `crates/forge-core/src/connection.rs` | Version gate added | YES (from STORY-013) |
