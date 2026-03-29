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
subsystem: "Config Drift Detection"
capability: "CAP-022"
lifecycle_status: active
introduced: v0.1.0
---

# BC-9.22.001 — Reconciliation Workflow Guidance

## Summary

Provides interactive guidance for resolving detected configuration drift.
Presents options for each drifted server: accept source A's config, accept
source B's config, or keep both (with warning). This is strictly advisory —
no configuration files are modified (DI-015). The user acts on the guidance
manually.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | Drift detection has been performed and at least one drift found |
| PRE-002 | Drift report (BC-9.21.002) is available with specific field differences |
| PRE-003 | User has invoked the reconciliation workflow |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Each drifted server is presented with reconciliation options |
| POST-002 | Options include: accept source A, accept source B, keep both (with warning) |
| POST-003 | For "accept" options: the recommended config values are shown (for manual copy) |
| POST-004 | For "keep both": a warning explains the behavioral implications of the difference |
| POST-005 | No config files have been written to, created, or modified (DI-015) |
| POST-006 | User choices are recorded for reporting purposes |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | Config files are NEVER modified by the reconciliation workflow (DI-015) |
| INV-002 | Every drifted server gets at least two reconciliation options |
| INV-003 | "Keep both" option always includes a warning about behavioral implications |
| INV-004 | Recommended config values shown match actual source values exactly |

## Edge Cases

| ID | Scenario | Expected Behavior | Source |
|----|----------|-------------------|--------|
| EC-001 | Drift involves > 2 sources (e.g., 3 editors all different) | Options expand: accept source A, accept source B, accept source C, keep all |  |
| EC-002 | User selects "accept source A" but source A config has other unrelated issues | Guidance only covers the drifted fields; does not validate overall config correctness |  |
| EC-003 | No drift detected (all in sync) | Reconciliation not needed; inform user "all servers in sync" |  |
| EC-004 | Drift is info-severity only (e.g., arg order) | Still presented but noted as low-impact; "keep both" may be recommended default |  |
| EC-005 | Server exists in 5+ sources with 3 different configs | Group sources by config value; present 3 options (one per unique config) |  |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-HP-001 | `sqlite` drifted: VS Code `--db test.db`, Claude Desktop `--db prod.db` | Options: (1) Use VS Code config: `args: ["mcp-server-sqlite","--db","test.db"]`, (2) Use Claude Desktop config: `args: ["mcp-server-sqlite","--db","prod.db"]`, (3) Keep both ⚠️ "different databases will be accessed by different editors" |
| TV-HP-002 | `github` missing from Cursor | Options: (1) Add to Cursor (show config to copy), (2) Remove from VS Code and Claude Desktop, (3) Keep as-is ⚠️ "github server unavailable in Cursor" |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-EC-001 | 3 editors, 3 different configs for `api-server` | Options: (1) Use VS Code config, (2) Use Cursor config, (3) Use Claude Desktop config, (4) Keep all ⚠️ with per-source warning |
| TV-EC-002 | All servers in sync | Message: "No drift detected. All N servers are in sync across M sources." |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-ERR-001 | Reconciliation invoked before drift detection | Error: "run drift detection first" |
| TV-ERR-002 | Drift report is stale (configs changed since detection) | Warning: "drift report may be stale; re-run detection for accurate results" |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | No config file is modified during reconciliation workflow | File system monitoring / hash comparison |
| VP-002 | Every drifted server receives reconciliation guidance | Automated completeness check |
| VP-003 | Recommended config values match actual source values exactly | Value comparison test |
| VP-004 | "Keep both" always includes a behavioral warning | Content assertion on output |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 (Drift Reconciliation Guidance) |
| Domain Invariants | DI-015 (read-only — never modify configs) |
| Edge Cases | — |
| Priority | P2 |
| NFRs | — |
