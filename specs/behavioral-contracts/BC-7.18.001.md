---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-03-29T11:25:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: ""
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: "Security Auditing"
capability: "CAP-018"
lifecycle_status: active
introduced: v0.1.0
---

# BC-7.18.001 — Structured Security Audit Report Generation

## Summary

The audit subsystem generates structured reports in both JSON and human-readable
formats. Reports include: a findings list with severity/confidence/evidence,
severity distribution summary, OWASP AST10 coverage metrics, and per-server
breakdown. The JSON schema matches the interface-definitions supplement.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | An audit has been executed against one or more servers |
| PRE-002 | Findings have been collected from detection contracts (BC-7.16.*, BC-7.17.*) |
| PRE-003 | Classification has been applied (BC-7.16.004) |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | JSON report is emitted on stdout when `--json` flag is present |
| POST-002 | Human-readable report is emitted on stdout when `--json` is not present |
| POST-003 | Report includes: `summary` (total findings, severity distribution), `findings` (list), `ast10_coverage`, `servers` (per-server breakdown) |
| POST-004 | Each finding in the report includes: id, rule, severity, confidence, description, evidence, tool, server, ast10_categories |
| POST-005 | Severity distribution: counts per level (critical, high, medium, low, info) |
| POST-006 | AST10 coverage: for each of the 6 runtime-applicable categories, whether at least one rule covers it |
| POST-007 | Report includes `timestamp`, `version` (forge-mcp version), and `duration_ms` (audit execution time) |
| POST-008 | Suppressed findings are included in the report with `suppressed: true` (not hidden) |
| POST-009 | JSON report validates against the schema in interface-definitions supplement |

## JSON Report Schema

```json
{
  "version": "0.1.0",
  "timestamp": "2026-03-29T11:25:00Z",
  "duration_ms": 1250,
  "summary": {
    "total_findings": 12,
    "by_severity": {
      "critical": 2,
      "high": 3,
      "medium": 5,
      "low": 1,
      "info": 1
    },
    "suppressed_count": 2,
    "servers_audited": 3
  },
  "ast10_coverage": {
    "AST01": {"covered": true, "rules": ["dangerous-tool-pattern", "network-outbound"]},
    "AST02": {"covered": true, "rules": ["ssrf-detection"]},
    "AST03": {"covered": true, "rules": ["permission-escalation", "dangerous-tool-pattern"]},
    "AST04": {"covered": true, "rules": ["schema-drift"]},
    "AST05": {"covered": true, "rules": ["auth-validation"]},
    "AST06": {"covered": true, "rules": ["root-enforcement"]},
    "coverage_pct": 100
  },
  "findings": [
    {
      "id": "F-001",
      "rule": "dangerous-tool-exec",
      "severity": "critical",
      "confidence": 1.0,
      "description": "Tool 'run_shell' has code execution capability",
      "evidence": {
        "tool": "run_shell",
        "match": "name contains 'shell'",
        "field": null
      },
      "server": "my-server",
      "ast10": ["AST01"],
      "suppressed": false
    }
  ],
  "servers": [
    {
      "name": "my-server",
      "transport": "stdio",
      "tools_scanned": 5,
      "findings_count": 3
    }
  ]
}
```

## Invariants

| ID | Invariant |
|----|-----------|
| DI-010 | Every finding in the report includes evidence |
| DI-012 | Suppressed findings are preserved in the report (overlay, not deletion) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Zero findings | Report generated with `total_findings: 0`, empty findings array, exit 0 |
| EC-002 | All findings suppressed | Report shows `total_findings: N`, `suppressed_count: N`; all findings have `suppressed: true`; exit 0 (no unsuppressed findings) |
| EC-003 | 1000+ findings | Report includes all findings; no truncation |
| EC-004 | Mixed suppressed and unsuppressed findings | Both present in report; only unsuppressed count toward exit code determination |
| EC-005 | Server unreachable during audit | Report includes server with `error: "connection_failed"`; no findings for that server |
| EC-006 | Human-readable format (no `--json`) | Table/text format with color-coded severities on stderr-capable terminals |
| EC-007 | `--json` combined with `--quiet` | JSON report only, no progress on stderr |
| EC-008 | Report for single server vs multi-server | Same schema; `servers` array has 1 or N entries |

## Canonical Test Vectors

### Happy Path

| Input | Expected Report Summary | Exit Code |
|-------|------------------------|-----------|
| `forge-mcp audit srv1 --json` (3 findings: 1 critical, 2 medium) | `total: 3, critical: 1, medium: 2` | 4 |
| `forge-mcp audit srv1 srv2 --json` (0 findings) | `total: 0` | 0 |
| `forge-mcp audit srv1 --json` (2 findings, both suppressed) | `total: 2, suppressed: 2` | 0 |

### Edge Case

| Input | Expected Behavior |
|-------|-------------------|
| `forge-mcp audit srv1` (no --json) | Human-readable table on stdout |
| `forge-mcp audit srv1 --json \| jq .summary.total_findings` | Integer |
| `forge-mcp audit unreachable --json` | Report with server error entry | 

### Error

| Scenario | Expected Behavior | Exit Code |
|----------|-------------------|-----------|
| All servers unreachable | Report with all servers errored; no findings | 2 |
| Internal error during report generation | Error JSON on stdout | 100 |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | JSON report validates against the defined schema | Schema validation test |
| VP-002 | `summary.total_findings` equals `findings.length` | Consistency test |
| VP-003 | `summary.by_severity` counts match actual finding severities | Consistency test |
| VP-004 | Suppressed findings have `suppressed: true` | Unit test |
| VP-005 | AST10 coverage percentage is accurate | Unit test |
| VP-006 | Zero-finding audit produces valid report with exit code 0 | Integration test |

## Traceability

| Source | Target |
|--------|--------|
| CAP-018 | This contract |
| DI-010 | Invariant (evidence in findings) |
| DI-012 | Invariant (suppression is overlay) |
| BC-7.16.001 | Dangerous tool findings (input) |
| BC-7.16.002 | SSRF findings (input) |
| BC-7.16.003 | Schema drift findings (input) |
| BC-7.16.004 | Severity classification (applied to findings) |
| BC-7.17.001 | Permission escalation findings (input) |
| BC-7.17.002 | Root enforcement findings (input) |
| BC-7.17.003 | Auth validation findings (input) |
| BC-7.18.002 | Suppression rules (applied before report) |
| BC-7.18.003 | AST10 coverage (reported in ast10_coverage section) |
