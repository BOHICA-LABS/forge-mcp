---
document_type: story
story_id: STORY-052
epic_id: EPIC-07
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 8
depends_on: [STORY-047, STORY-048, STORY-049, STORY-050, STORY-051]
blocks: [STORY-053, STORY-054]
behavioral_contracts: [BC-7.18.001, BC-7.18.003]
verification_properties: [VP-009, VP-010, VP-011]
priority: P1
assumption_validations: []
risk_mitigations: [R-004, R-011]
---

# STORY-052: Security Audit Report Generation & OWASP AST10 Mapping

## Narrative
- **As a** Security Team member
- **I want to** generate a structured security audit report with OWASP AST10 mapping
- **So that** I can communicate findings to stakeholders and demonstrate compliance

## Acceptance Criteria

### AC-001 (traces to BC-7.18.001 postcondition — structured report)
`generate_audit_report(Vec<SecurityFinding>) -> AuditReport` produces a structured report with: `server_name`, `audit_timestamp`, `findings: Vec<SecurityFinding>` (sorted by severity), `summary: { critical: u32, high: u32, medium: u32, low: u32 }`, `owasp_coverage: HashMap<String, Vec<String>>`.
- **Test:** `test_BC_7_18_001_structured_report_generated()`

### AC-002 (traces to BC-7.18.001 postcondition — JSON export)
`forge-mcp audit <server>` outputs the `AuditReport` as JSON on stdout. Exit code 4 when any Critical or High finding exists. Exit code 0 when all findings are Medium or Low.
- **Test:** `test_BC_7_18_001_cli_audit_json_output()`

### AC-003 (traces to BC-7.18.003 postcondition — OWASP AST10 mapping)
The report's `owasp_coverage` field maps each OWASP AST10 category to finding IDs: AST01-AST10. Categories with no findings are present as empty arrays. (NFR-006: ≥83% = ≥5 of 6 runtime-applicable categories covered.)
- **Test:** `test_BC_7_18_003_owasp_coverage_map()`

### AC-004 (traces to BC-7.18.003 — runtime category coverage)
The implemented rule set covers: AST03 (Permission Escalation), AST06 (Root Enforcement), AST07 (SSRF), AST08 (Auth), AST09 (Schema Drift), AST10 (Rug Pull). This is 6 of 6 runtime-applicable categories (100%).
- **Test:** `test_BC_7_18_003_runtime_category_coverage()`

### AC-005 (traces to BC-7.18.001 — report precision gate)
Report is only generated when precision confidence is > 80% overall (NFR-005). Low-confidence findings are downgraded one severity level in the report summary.
- **Test:** `test_BC_7_18_001_precision_threshold()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `generate_audit_report()` | `forge-security/src/report.rs` | Pure |
| `AuditReport` | `forge-security/src/report.rs` | Pure |
| CLI audit command | `forge-mcp/src/commands/audit.rs` | Effectful |

## UX Screens
- SCR-007 (Security Audit View)
- FLOW-005 (Security Audit Workflow)

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | No findings | Report with empty findings, exit 0 |
| EC-002 | All findings are Low | Exit 0 |
| EC-003 | Mix of Critical and Low | Exit 4 |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| generate_audit_report() | Pure | Aggregates Vec<Finding> → AuditReport |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~1,000 |
| BC-7.18.001, BC-7.18.003 | ~800 |
| NFR-005, NFR-006 | ~400 |
| **Total** | **~2,200** |
| Agent context window | 200K |
| **Budget usage** | **~1.1%** |

## Tasks

1. [ ] Write failing tests for all 5 ACs
2. [ ] Define `AuditReport` struct
3. [ ] Implement `generate_audit_report()` pure function
4. [ ] Implement OWASP AST10 category mapping
5. [ ] Add precision gate logic
6. [ ] Implement `forge-mcp audit` CLI command
7. [ ] Verify exit code 4 for Critical/High findings
8. [ ] Verify Red Gate
9. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-047 | SecurityFinding type + owasp tag | Aggregate all findings here | Need ALL detectors running before generating report |
| STORY-023 | Exit code 4 defined for security | Implement in CLI audit command | |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| OWASP ≥83% coverage (NFR-006) | nfr-catalog.md | Audit category mapping |
| Report is pure function | purity-boundary-map.md | No I/O in generate_audit_report |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| serde | >= 1.0 | Report JSON serialization | `#[derive(Serialize)]` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-security/src/report.rs` | AuditReport + generate | NO — this story creates it |
| `crates/forge-mcp/src/commands/audit.rs` | CLI audit command | NO |
