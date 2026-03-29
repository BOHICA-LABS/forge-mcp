---
document_type: story
story_id: STORY-053
epic_id: EPIC-07
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 3
depends_on: [STORY-052]
blocks: [STORY-054]
behavioral_contracts: [BC-7.18.002]
verification_properties: []
priority: P1
assumption_validations: []
risk_mitigations: []
---

# STORY-053: Security Finding Suppression Rules

## Narrative
- **As a** Security Team member
- **I want to** define suppression rules for known-acceptable findings
- **So that** recurring false positives don't crowd the audit report

## Acceptance Criteria

### AC-001 (traces to BC-7.18.002 postcondition — suppression rule definition)
Suppression rules are defined in `~/.config/forge-mcp/suppression.json` as: `[{ "rule": "filesystem", "server": "my-fs-server", "reason": "Intentional filesystem access", "expires": "2026-12-31" }]`. A finding matching `rule + server` is suppressed.
- **Test:** `test_BC_7_18_002_suppression_rule_applied()`

### AC-002 (traces to BC-7.18.002 postcondition — suppression logged)
Suppressed findings emit `E-SEC-003` warning: "Security finding suppressed: <id> — <reason>". They appear in the report as `status: suppressed`, not omitted.
- **Test:** `test_BC_7_18_002_suppression_logged()`

### AC-003 (traces to BC-7.18.002 — expiry)
Rules with `expires` date are automatically inactive after the expiry date. Expired rules emit `E-SEC-004` auto-suppress suggestion on next match.
- **Test:** `test_BC_7_18_002_suppression_expires()`

### AC-004 (traces to BC-7.18.002 — suppression overlay is pure)
`apply_suppressions(findings, rules) -> Vec<Finding>` is a pure function: applies suppression overlay to findings without modifying original state.
- **Test:** `test_BC_7_18_002_suppression_overlay_pure()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `apply_suppressions()` | `forge-security/src/suppression.rs` | Pure (DI-012) |
| Suppression rule loader | `forge-security/src/suppression.rs` | Effectful (file read) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Suppression file missing | No suppressions applied, no error |
| EC-002 | Malformed suppression JSON | Log parse error, no suppressions |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| apply_suppressions() | Pure (DI-012) | Takes findings + rules, returns findings |
| Rule loader | Effectful | File read |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~700 |
| BC-7.18.002 | ~400 |
| **Total** | **~1,100** |
| Agent context window | 200K |
| **Budget usage** | **~0.6%** |

## Tasks

1. [ ] Write failing tests for all 4 ACs
2. [ ] Define `SuppressionRule` struct
3. [ ] Implement suppression rule JSON loader
4. [ ] Implement `apply_suppressions()` pure overlay
5. [ ] Implement expiry handling
6. [ ] Verify Red Gate
7. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-052 | AuditReport has Vec<Finding> | apply_suppressions() runs before report generation | DI-012 requires pure overlay |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Pure suppression overlay (DI-012) | purity-boundary-map.md | apply_suppressions() no I/O |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| chrono | >= 0.4 | Expiry date comparison | `chrono::NaiveDate` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-security/src/suppression.rs` | SuppressionRule + apply_suppressions | NO — this story creates it |
