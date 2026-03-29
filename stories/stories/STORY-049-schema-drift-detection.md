---
document_type: story
story_id: STORY-049
epic_id: EPIC-07
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-027]
blocks: [STORY-052]
behavioral_contracts: [BC-7.16.003]
verification_properties: [VP-011]
priority: P1
assumption_validations: []
risk_mitigations: [R-014]
---

# STORY-049: Schema Drift / Rug Pull Detection

## Narrative
- **As a** Security Team member
- **I want to** detect when an MCP server's tool schemas change between connections
- **So that** I can identify potential rug pull attacks where a server's behavior changes after trust is established

## Acceptance Criteria

### AC-001 (traces to BC-7.16.003 postcondition — baseline hash storage)
On first `tools/list` response for a server, compute and store a SHA-256 hash of the serialized tool schema set. Hash stored per server in daemon session context.
- **Test:** `test_BC_7_16_003_baseline_hash_stored()`

### AC-002 (traces to BC-7.16.003 postcondition — drift detection on reconnect)
On subsequent `tools/list` responses (reconnect or `list_changed` notification), recompute hash and compare to baseline. If different, emit Finding `category: SchemaDrift`, `severity: Critical` and `E-SEC-005` warning. (NFR-007.)
- **Test:** `test_BC_7_16_003_drift_detected_on_reconnect()` (proptest for VP-011)

### AC-003 (traces to BC-7.16.003 — drift report detail)
The SchemaDrift finding includes: which tools were added, removed, or had description/schema changes. Human-readable diff summary.
- **Test:** `test_BC_7_16_003_drift_report_detail()`

### AC-004 (traces to BC-7.16.003 — hash verified every connection)
Hash comparison fires on EVERY `tools/list` response, not just reconnects. (NFR-007 schema metadata integrity.)
- **Test:** `test_BC_7_16_003_hash_every_connection()`

### AC-005 (traces to BC-7.16.003 — no false positives for identical schemas)
Identical tool schemas (same tools, same descriptions) produce identical hashes across different server instances. No false drift findings.
- **Test:** `test_BC_7_16_003_identical_schema_no_false_positive()` (VP-011 proptest)

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `SchemaDriftDetector` | `forge-security/src/schema_drift.rs` | Pure (comparison) |
| Baseline hash storage | `forge-daemon/src/session.rs` | Effectful (per-session) |

## UX Screens
- SCR-007 (Security Audit View) — shows [DRIFT] alert

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Server adds a new benign tool | SchemaDrift finding with High severity (not Critical) |
| EC-002 | Tool renamed (same schema) | Drift finding — names are part of hash |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Hash computation | Pure | Deterministic hash of schema |
| Drift comparison | Pure | hash_a != hash_b |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~900 |
| BC-7.16.003 | ~600 |
| VP-011 | ~300 |
| **Total** | **~1,800** |
| Agent context window | 200K |
| **Budget usage** | **~0.9%** |

## Tasks

1. [ ] Write failing tests for all 5 ACs
2. [ ] Implement schema hash computation (deterministic serialization + SHA-256)
3. [ ] Implement SchemaDriftDetector (pure comparison)
4. [ ] Store baseline in daemon session context
5. [ ] Implement drift detail diff (added/removed/changed tools)
6. [ ] Add proptest for VP-011 drift detection completeness
7. [ ] Verify Red Gate
8. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-047 | SecurityFinding type established | SchemaDrift is a finding variant | Hash must be over canonically serialized JSON |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Hash on every connection (NFR-007) | nfr-catalog.md | Not just reconnects |
| Schema comparison is pure | purity-boundary-map.md | No I/O in hash comparison |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| sha2 | >= 0.10 | SHA-256 hash | `sha2::Sha256` |
| proptest | dev | VP-011 | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-security/src/schema_drift.rs` | SchemaDriftDetector | NO — this story creates it |
| `crates/forge-security/proofs/schema_drift.rs` | VP-011 proptest | NO |
