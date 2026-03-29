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

# BC-7.18.002 — User-Defined Finding Suppression Rules

## Summary

Users can define suppression rules in the config file to suppress known-acceptable
security findings. Suppression is an overlay: the original finding is preserved in
the audit report with `suppressed: true`, maintaining a complete audit trail.
Suppressions are traceable to the user action that created them.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | An audit is being executed |
| PRE-002 | Suppression rules are loaded from config file or CLI arguments |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Suppression rules can match by: rule ID, server name, tool name pattern, or finding pattern |
| POST-002 | A suppressed finding is marked `suppressed: true` in the report but NOT removed |
| POST-003 | The original finding content (severity, confidence, evidence) is unchanged after suppression |
| POST-004 | Each suppression rule includes: a `reason` field (human explanation) and `created_at` timestamp |
| POST-005 | Suppressed findings do NOT contribute to the exit code (exit 0 if all findings are suppressed) |
| POST-006 | The audit report includes `suppressed_count` in the summary |
| POST-007 | Suppression rules are logged at audit start (which rules are active) |

## Suppression Rule Schema

```toml
[[audit.suppress]]
rule = "ssrf-private-ip"          # Match by rule ID
server = "internal-service"        # Optional: only for this server
tool = "fetch_*"                   # Optional: glob pattern on tool name
reason = "Internal service expected to access private IPs"
created_at = "2026-03-29"
expires_at = "2026-06-29"          # Optional: auto-expire

[[audit.suppress]]
rule = "dangerous-tool-exec"
tool = "run_tests"
reason = "Test runner requires exec capability"
created_at = "2026-03-29"
```

## Suppression Match Logic

1. Rule ID is required for every suppression rule
2. Optional filters (server, tool) are AND-combined: all specified filters must match
3. Tool matching supports glob patterns (`*`, `?`)
4. Expired rules (past `expires_at`) are ignored during matching

## Invariants

| ID | Invariant |
|----|-----------|
| DI-012 | Suppression is overlay: original finding is preserved, not deleted |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Suppression rule with no matching findings | Rule is active but matches nothing; no effect |
| EC-002 | Multiple suppression rules match same finding | Finding suppressed once (not duplicated); first matching rule's reason is used |
| EC-003 | Suppression rule expires during audit | Rule was valid at audit start; finding remains suppressed for this audit run |
| EC-004 | Severity change after suppression (DEC-013) | If the underlying detection changes severity, the suppressed finding shows the NEW severity but remains suppressed |
| EC-005 | Suppress all findings from a server | Use `server = "server-name"` without tool filter; all rules matched for that server |
| EC-006 | Invalid suppression rule (missing `rule` field) | Config error: report on stderr, skip this rule, continue with valid rules |
| EC-007 | Glob pattern `*` matches all tools | Valid: suppresses all findings matching the rule ID across all tools |
| EC-008 | Suppression rule with `expires_at` in the past | Rule is expired; findings not suppressed; info message logged |
| EC-009 | CLI `--no-suppress` flag | All suppression rules are ignored for this run |
| EC-010 | CLI `--suppress rule-id` | Ad-hoc suppression for single run (not persisted to config) |

## Canonical Test Vectors

### Happy Path

| Findings | Suppression Rules | Report Outcome |
|----------|-------------------|----------------|
| 3 findings: F1 (ssrf, srv1), F2 (exec, srv1), F3 (ssrf, srv2) | `rule=ssrf, server=srv1` | F1 suppressed; F2, F3 unsuppressed |
| 2 findings: F1 (exec, tool=run_tests), F2 (exec, tool=deploy) | `rule=exec, tool=run_*` | F1 suppressed; F2 unsuppressed |
| 1 finding: F1 (ssrf, srv1) | `rule=ssrf, server=srv1, expires_at=2026-01-01` | F1 NOT suppressed (rule expired) |

### Edge Case

| Findings | Suppression Rules | Report Outcome |
|----------|-------------------|----------------|
| 5 findings, all suppressed | 5 matching rules | `total: 5, suppressed: 5`, exit 0 |
| 0 findings | 3 rules | `total: 0`, exit 0 (rules match nothing) |
| 2 findings, same rule, different servers | `rule=ssrf` (no server filter) | Both suppressed |

### Error

| Scenario | Expected Behavior |
|----------|-------------------|
| Suppression rule missing `rule` field | Warning: invalid rule skipped; audit continues |
| Suppression config is invalid TOML | Config error on stderr; exit 3 |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | Suppressed findings have `suppressed: true` and unchanged severity/confidence/evidence | Unit test |
| VP-002 | Suppressed findings do NOT affect exit code | Integration test |
| VP-003 | Expired rules do not suppress findings | Unit test |
| VP-004 | Glob patterns match correctly (`*`, `?`, character classes) | Unit test |
| VP-005 | `--no-suppress` ignores all suppression rules | Integration test |
| VP-006 | Suppression rules are logged at audit start | Integration test |
| VP-007 | Finding severity change does not break suppression | Unit test (DEC-013) |

## Traceability

| Source | Target |
|--------|--------|
| CAP-018 | This contract |
| DI-012 | Invariant (suppression is overlay) |
| DEC-013 | Edge case (severity change after suppression) |
| BC-7.18.001 | Audit report (suppressed findings appear in report) |
| BC-7.16.004 | Severity classification (severity may change independently of suppression) |
