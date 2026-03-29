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
capability: "CAP-021"
lifecycle_status: active
introduced: v0.1.0
---

# BC-9.21.002 — Drift Report with Actionable Detail

## Summary

Produces a structured drift report for each detected configuration difference.
Reports which editors are involved, what specifically differs (command, args,
env, URL, transport type), and assigns severity levels. Output is available
in both JSON (machine-readable) and human-readable formats.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | Cross-editor config comparison has been performed (BC-9.21.001) |
| PRE-002 | Drift results include per-server comparison data with specific field differences |
| PRE-003 | Output format has been specified (JSON, human-readable, or both) |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Each drift entry identifies the affected server name |
| POST-002 | Each drift entry lists which editors/sources are involved |
| POST-003 | Each drift entry specifies what differs: field name and values per source |
| POST-004 | Drift severity is assigned: `info` for cosmetic differences, `warning` for substantive |
| POST-005 | JSON output conforms to a defined schema |
| POST-006 | Human-readable output is formatted for terminal display |
| POST-007 | Report includes summary counts: total servers, in-sync, drifted, missing |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | Every drifted server has at least one specific field difference listed |
| INV-002 | Severity is deterministic: same difference always yields same severity |
| INV-003 | No config files are modified (DI-015) |
| INV-004 | JSON and human-readable outputs represent the same underlying data |

## Edge Cases

| ID | Scenario | Expected Behavior | Source |
|----|----------|-------------------|--------|
| EC-001 | Arg order differs but args are the same set | Severity: `info` (cosmetic — arg order may not matter) |  |
| EC-002 | Command differs (`npx` vs `uvx` for same server) | Severity: `warning` (substantive — different runtime) |  |
| EC-003 | Environment variables differ | Severity: `warning` (different env may change behavior) |  |
| EC-004 | Transport type differs (stdio vs HTTP) | Severity: `warning` (substantive architectural difference) |  |
| EC-005 | URL differs only in trailing slash | Severity: `info` (cosmetic) |  |
| EC-006 | No drift detected (all in sync) | Report with zero drift entries; summary shows all in-sync |  |
| EC-007 | Server has > 5 field differences | All differences listed; no truncation |  |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output (JSON) |
|----|-------|------------------------|
| TV-HP-001 | `sqlite` drifted: VS Code has `--db test.db`, Claude Desktop has `--db prod.db` | `{"server":"sqlite","status":"drifted","severity":"warning","differences":[{"field":"args","sources":{"vscode":["--db","test.db"],"claude-desktop":["--db","prod.db"]}}]}` |
| TV-HP-002 | `github` missing from Cursor | `{"server":"github","status":"missing","severity":"warning","present_in":["vscode","claude-desktop"],"missing_from":["cursor"]}` |
| TV-HP-003 | All servers in sync | `{"summary":{"total":3,"in_sync":3,"drifted":0,"missing":0},"drifts":[]}` |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-EC-001 | `sqlite` args: VS Code `["a","b"]`, Claude Desktop `["b","a"]` | Severity: `info`, difference field: `args` (order only) |
| TV-EC-002 | `api-server` URL: VS Code `http://localhost:3000/`, Cursor `http://localhost:3000` | Severity: `info` (trailing slash only) |
| TV-EC-003 | `my-server` transport: VS Code `stdio`, Cursor `sse` | Severity: `warning`, difference field: `transport` |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-ERR-001 | No comparison data available | Error: "no comparison data — run drift detection first" |
| TV-ERR-002 | Invalid output format requested | Error: "unsupported format: xml; supported: json, text" |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | JSON output validates against drift report schema | JSON schema validation |
| VP-002 | Severity assignment is consistent for same difference type | Property-based test with randomized inputs |
| VP-003 | JSON and human-readable outputs are semantically equivalent | Cross-format comparison test |
| VP-004 | Summary counts match individual drift entries | Automated invariant check |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-021 (Config Drift Detection) |
| Domain Invariants | DI-015 (read-only — never modify configs) |
| Edge Cases | — |
| Priority | P2 |
| NFRs | — |
