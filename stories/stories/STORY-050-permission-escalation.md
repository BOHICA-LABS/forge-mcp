---
document_type: story
story_id: STORY-050
epic_id: EPIC-07
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-047]
blocks: [STORY-052]
behavioral_contracts: [BC-7.17.001, BC-7.17.002]
verification_properties: [VP-009]
priority: P1
assumption_validations: []
risk_mitigations: [R-004]
---

# STORY-050: Permission Escalation & Root Enforcement Detection

## Narrative
- **As a** Security Team member
- **I want to** detect permission escalation patterns and root boundary violations
- **So that** I can identify servers that try to access resources outside their declared scope

## Acceptance Criteria

### AC-001 (traces to BC-7.17.001 postcondition — privilege escalation patterns)
Detects tool call arguments or results containing privilege escalation indicators: `sudo`, `su`, `setuid`, `chmod 777`, `chown root`, `runas`, escalated path patterns (`/etc/passwd`, `/etc/shadow`, `/root/`). Emits Finding with `category: PermissionEscalation`, `severity: Critical`.
- **Test:** `test_BC_7_17_001_privilege_escalation_patterns()`

### AC-002 (traces to BC-7.17.002 postcondition — root enforcement)
When Forge MCP has advertised root paths (via `roots/list`), and a tool call accesses a path OUTSIDE the declared roots, emits Finding `category: RootBoundaryViolation`, `severity: High`.
- **Test:** `test_BC_7_17_002_root_boundary_violation()`

### AC-003 (traces to BC-7.17.002 — root enforcement only when roots declared)
If no root paths are configured, root boundary checking is disabled. No false positives from unrestricted servers.
- **Test:** `test_BC_7_17_002_no_roots_no_check()`

### AC-004 (traces to BC-7.17.001 — OWASP AST03 mapping)
Permission escalation findings are tagged `owasp: AST03` (Over-Privileged Skills). This mapping contributes to NFR-006 (≥83% AST10 coverage).
- **Test:** `test_BC_7_17_001_ast03_tag()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `EscalationDetector` | `forge-security/src/escalation.rs` | Pure |
| Root path checker | `forge-security/src/roots.rs` | Pure |

## UX Screens
- SCR-007 (Security Audit View)

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Tool name "sudo_helper" (not actual sudo) | Low confidence finding |
| EC-002 | Path within root but traversal attempt (../../../etc) | Root boundary violation finding |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| EscalationDetector | Pure | Pattern matching |
| Root path checker | Pure | Path prefix comparison |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~800 |
| BC-7.17.001, BC-7.17.002 | ~600 |
| **Total** | **~1,400** |
| Agent context window | 200K |
| **Budget usage** | **~0.7%** |

## Tasks

1. [ ] Write failing tests for all 4 ACs
2. [ ] Implement EscalationDetector with privilege patterns
3. [ ] Implement root path boundary checker
4. [ ] Add OWASP AST03 tag
5. [ ] Verify Red Gate
6. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-047 | RuleEngine pattern established | Add escalation patterns to engine | Path comparison must normalize (resolve ..) |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Pure detection | purity-boundary-map.md | No I/O in detector |
| OWASP AST10 coverage (NFR-006) | nfr-catalog.md | Tag each finding with AST category |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| (no new deps) | | | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-security/src/escalation.rs` | EscalationDetector | NO — this story creates it |
| `crates/forge-security/src/roots.rs` | Root path checker | NO |
