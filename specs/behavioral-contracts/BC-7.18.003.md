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

# BC-7.18.003 — OWASP AST10 Coverage Mapping

## Summary

Each security detection rule is mapped to one or more OWASP Agentic Security
Threats Top 10 (AST10) categories. The audit report includes a coverage metric
showing which of the 6 runtime-detectable categories are covered. The target is
≥ 83% coverage (≥ 5 of 6 runtime-applicable categories).

## OWASP AST10 Categories

| ID | Category | Runtime-Detectable | forge-mcp Coverage |
|----|----------|-------------------|-------------------|
| AST01 | Malicious Skills/Tools | Yes | BC-7.16.001 (dangerous tool patterns, network) |
| AST02 | Excessive Data Exposure | Yes | BC-7.16.002 (SSRF / private data exfiltration) |
| AST03 | Over-Privileged Skills | Yes | BC-7.16.001 (broad access), BC-7.17.001 (escalation) |
| AST04 | Insecure Metadata | Yes | BC-7.16.003 (schema drift) |
| AST05 | Inadequate Authentication | Yes | BC-7.17.003 (auth validation) |
| AST06 | Weak Isolation | Yes | BC-7.17.002 (root enforcement) |
| AST07 | Tool/Skill Update Drift | No (process) | BC-7.16.003 (partial: runtime hash comparison) |
| AST08 | Data Poisoning | No (process) | Out of scope for v1 |
| AST09 | Supply Chain Compromise | No (process) | Out of scope for v1 |
| AST10 | Unmonitored Agent Behavior | No (process) | Out of scope for v1 |

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | The security rule registry is loaded with rule-to-AST10 mappings |
| PRE-002 | At least one audit has been executed |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Each security detection rule has at least one AST10 category mapping |
| POST-002 | The audit report includes an `ast10_coverage` section listing each runtime-detectable category |
| POST-003 | For each category, the report shows: whether it's covered (`true`/`false`) and which rules cover it |
| POST-004 | Coverage percentage is calculated as: `covered_runtime_categories / 6 * 100` |
| POST-005 | The target coverage of ≥ 83% (≥ 5 of 6) is reported as a pass/fail metric |
| POST-006 | Process-only categories (AST07–AST10) are listed as "not applicable (process-only)" |
| POST-007 | AST07 has partial runtime coverage via BC-7.16.003 (hash comparison); reported as "partial" |

## Rule-to-AST10 Mapping Registry

| Rule ID | Rule Name | AST10 Categories |
|---------|-----------|------------------|
| dangerous-tool-exec | Code execution detection | AST01 |
| dangerous-tool-fs | Filesystem access detection | AST03 |
| dangerous-tool-network | Network access detection | AST01 |
| ssrf-metadata | Cloud metadata SSRF | AST02 |
| ssrf-private-ip | Private IP SSRF | AST02 |
| schema-drift | Schema drift detection | AST04, AST07 |
| permission-escalation | Capability escalation | AST03 |
| root-enforcement | Root boundary verification | AST06 |
| auth-leak | Credential leak detection | AST05 |
| auth-transport | Transport auth validation | AST05 |

## Invariants

| ID | Invariant |
|----|-----------|
| NFR-006 | AST10 coverage ≥ 83% of runtime-applicable categories (≥ 5 of 6) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | All 6 runtime categories covered | Coverage: 100%, status: pass |
| EC-002 | Only 4 of 6 runtime categories covered | Coverage: 67%, status: fail (below 83%) |
| EC-003 | A rule covers multiple AST10 categories | Each category gets credit; rule counted once per category |
| EC-004 | No rules loaded (empty rule registry) | Coverage: 0%, status: fail |
| EC-005 | New AST10 categories added in future spec | Coverage denominator stays at 6 (runtime-only) until registry updated |
| EC-006 | Custom user rules with AST10 mappings | Included in coverage calculation |
| EC-007 | AST07 (Update Drift) treated as process-only | Listed as "partial" coverage due to BC-7.16.003 runtime hash check |

## Canonical Test Vectors

### Happy Path — Full Coverage

| Active Rules | Coverage |
|-------------|----------|
| dangerous-tool-*, ssrf-*, schema-drift, permission-escalation, root-enforcement, auth-* | 6/6 = 100%, pass |

### Edge Case — Partial Coverage

| Active Rules | Covered Categories | Coverage |
|-------------|-------------------|----------|
| dangerous-tool-exec, ssrf-metadata, schema-drift, auth-leak | AST01, AST02, AST04, AST05 | 4/6 = 67%, fail |
| dangerous-tool-exec, ssrf-metadata, schema-drift, permission-escalation, auth-leak | AST01, AST02, AST03, AST04, AST05 | 5/6 = 83%, pass |

### Error

| Scenario | Expected Behavior |
|----------|-------------------|
| Rule registry cannot be loaded | Warning: AST10 coverage unknown; report shows "unable to compute" |
| Rule has no AST10 mapping | Warning: unmapped rule; does not contribute to coverage |

## AST10 Coverage Report Section

### JSON Format

```json
{
  "ast10_coverage": {
    "AST01": {"covered": true, "rules": ["dangerous-tool-exec", "dangerous-tool-network"]},
    "AST02": {"covered": true, "rules": ["ssrf-metadata", "ssrf-private-ip"]},
    "AST03": {"covered": true, "rules": ["dangerous-tool-fs", "permission-escalation"]},
    "AST04": {"covered": true, "rules": ["schema-drift"]},
    "AST05": {"covered": true, "rules": ["auth-leak", "auth-transport"]},
    "AST06": {"covered": true, "rules": ["root-enforcement"]},
    "AST07": {"covered": "partial", "rules": ["schema-drift"], "note": "Runtime hash comparison; full drift requires process controls"},
    "AST08": {"covered": false, "rules": [], "note": "Process-only; out of scope for runtime detection"},
    "AST09": {"covered": false, "rules": [], "note": "Process-only; out of scope for runtime detection"},
    "AST10": {"covered": false, "rules": [], "note": "Process-only; out of scope for runtime detection"},
    "runtime_coverage_pct": 100,
    "runtime_target_pct": 83,
    "runtime_pass": true
  }
}
```

### Human-Readable Format

```
OWASP AST10 Coverage
═══════════════════════════════════════
AST01  Malicious Skills        ✔ covered  (dangerous-tool-exec, dangerous-tool-network)
AST02  Excessive Data Exposure ✔ covered  (ssrf-metadata, ssrf-private-ip)
AST03  Over-Privileged Skills  ✔ covered  (dangerous-tool-fs, permission-escalation)
AST04  Insecure Metadata       ✔ covered  (schema-drift)
AST05  Inadequate Auth         ✔ covered  (auth-leak, auth-transport)
AST06  Weak Isolation          ✔ covered  (root-enforcement)
AST07  Update Drift            ~ partial  (schema-drift — runtime hash only)
AST08  Data Poisoning          — n/a      (process-only)
AST09  Supply Chain            — n/a      (process-only)
AST10  Unmonitored Behavior    — n/a      (process-only)
───────────────────────────────────────
Runtime coverage: 6/6 (100%) ✔ PASS (target: ≥83%)
```

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | All 6 runtime-detectable AST10 categories have at least one mapped rule | Registry test |
| VP-002 | Coverage percentage calculation is `covered_count / 6 * 100` | Unit test |
| VP-003 | Coverage pass/fail threshold is ≥ 83% (≥ 5 of 6) | Unit test |
| VP-004 | Process-only categories (AST07–AST10) do not count toward coverage denominator | Unit test |
| VP-005 | Adding a rule with an AST10 mapping updates coverage calculation | Integration test |
| VP-006 | Removing a rule that was the sole coverage for a category decreases coverage | Integration test |

## Traceability

| Source | Target |
|--------|--------|
| CAP-018 | This contract |
| NFR-006 | Non-functional requirement (≥ 83% coverage target) |
| BC-7.16.001 | Covers AST01, AST03 |
| BC-7.16.002 | Covers AST02 |
| BC-7.16.003 | Covers AST04, partial AST07 |
| BC-7.17.001 | Covers AST03 |
| BC-7.17.002 | Covers AST06 |
| BC-7.17.003 | Covers AST05 |
| BC-7.18.001 | Audit report (coverage section) |
