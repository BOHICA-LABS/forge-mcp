---
document_type: story
story_id: STORY-058
epic_id: EPIC-08
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-055, STORY-056, STORY-057]
blocks: []
behavioral_contracts: [BC-8.20.001, BC-8.20.002]
verification_properties: []
priority: P1
assumption_validations: []
risk_mitigations: []
---

# STORY-058: Conformance Report Output (JUnit XML + JSON)

## Narrative
- **As an** MCP Server Author integrating with CI/CD
- **I want to** get conformance test results in JUnit XML and JSON formats
- **So that** I can integrate with GitHub Actions, GitLab CI, and Jenkins

## Acceptance Criteria

### AC-001 (traces to BC-8.20.001 postcondition — JUnit XML output)
`forge-mcp test <server> --format junit` outputs valid JUnit XML to stdout. Format: `<testsuite name="MCP Conformance" tests="N" failures="M" errors="0">` with one `<testcase>` per conformance test. Failed tests include `<failure message="...">`.
- **Test:** `test_BC_8_20_001_junit_xml_valid()`

### AC-002 (traces to BC-8.20.001 — CI integration)
JUnit XML schema validates against the standard JUnit 4 XSD. Compatible with GitHub Actions test summary, GitLab CI artifacts, and Jenkins JUnit plugin.
- **Test:** `test_BC_8_20_001_junit_schema_valid()`

### AC-003 (traces to BC-8.20.002 postcondition — JSON report)
`forge-mcp test <server> --format json` outputs a JSON conformance report: `{ "server": {...}, "results": [...], "summary": { "total": N, "passed": P, "failed": F, "skipped": S, "coverage_pct": X.X } }`.
- **Test:** `test_BC_8_20_002_json_report_structure()`

### AC-004 (traces to BC-8.20.001 — exit code 1 on failures)
When any conformance test fails, exit code is 1 (per BC-5.11.003). CI pipelines use this for gating.
- **Test:** `test_BC_8_20_001_exit_1_on_failure()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| JUnit XML generator | `forge-conformance/src/junit_output.rs` | Pure |
| JSON reporter | `forge-conformance/src/json_output.rs` | Pure |
| CLI test command | `forge-mcp/src/commands/test.rs` | Effectful |

## UX Screens
- SCR-008 (Conformance Test Runner) — shows progress then results

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | All tests pass | Exit 0, 0 failures in JUnit |
| EC-002 | JUnit with special chars in names | XML-escaped properly |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| JUnit/JSON generators | Pure | ConformanceResults → string output |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~800 |
| BC-8.20.001, BC-8.20.002 | ~600 |
| **Total** | **~1,400** |
| Agent context window | 200K |
| **Budget usage** | **~0.7%** |

## Tasks

1. [ ] Write failing tests for all 4 ACs
2. [ ] Implement JUnit XML generator (pure)
3. [ ] Implement JSON reporter (pure)
4. [ ] Add `--format junit|json|text` flag to `forge-mcp test`
5. [ ] Verify XML validates against JUnit 4 XSD
6. [ ] Verify exit code 1 on failures
7. [ ] Verify Red Gate
8. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-055 | ConformanceResult type | Serialize to XML and JSON | JUnit XML needs exact element order for CI tools |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Exit code 1 for test failures | BC-5.11.003 | Conformance test uses exit code 1 (not 4) |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| quick-xml | >= 0.31 | JUnit XML generation | `quick_xml::Writer` |
| serde | >= 1.0 | JSON output | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-conformance/src/junit_output.rs` | JUnit XML generator | NO — this story creates it |
| `crates/forge-conformance/src/json_output.rs` | JSON reporter | NO |
| `crates/forge-mcp/src/commands/test.rs` | CLI test command | NO |
