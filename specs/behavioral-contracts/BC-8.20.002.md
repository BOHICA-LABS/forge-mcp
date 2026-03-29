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
subsystem: "Conformance Testing"
capability: "CAP-020"
lifecycle_status: active
introduced: v0.1.0
---

# BC-8.20.002 — JSON Conformance Report Output

## Summary

Generates a structured JSON conformance report with per-method results,
aggregate statistics, server metadata, and spec version information. Exit
code semantics follow DI-014: 0 if all checks pass, 1 if any check fails.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | Conformance test run has completed (fully or partially) |
| PRE-002 | Test results include per-method status, timing, and error details |
| PRE-003 | Server info (name, version) is available from `InitializeResult` |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Output is valid JSON parseable by any JSON parser |
| POST-002 | Report includes `summary` object with `total`, `passed`, `failed`, `skipped`, `errored` counts |
| POST-003 | Report includes `server` object with `name`, `version`, `protocolVersion` |
| POST-004 | Report includes `specVersion` field indicating which MCP spec version was tested against |
| POST-005 | Report includes `results` array with one entry per conformance check |
| POST-006 | Each result entry includes `method`, `status` (pass/fail/skip/error), `duration_ms`, and optional `message` |
| POST-007 | Exit code is 0 if `failed` + `errored` = 0; exit code is 1 otherwise |
| POST-008 | Report includes `timestamp` (ISO 8601) and `duration_ms` (total run time) |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | `summary.total` = `summary.passed` + `summary.failed` + `summary.skipped` + `summary.errored` |
| INV-002 | Exit code is deterministic given the same results |
| INV-003 | JSON output is a single valid document (not NDJSON) |
| INV-004 | Exit code 0 ↔ all checks passed; exit code 1 ↔ at least one failure or error (DI-014) |

## Edge Cases

| ID | Scenario | Expected Behavior | Source |
|----|----------|-------------------|--------|
| EC-001 | All checks pass | Exit code 0; `summary.failed` = 0, `summary.errored` = 0 |  |
| EC-002 | Some checks skipped (server doesn't support capability) | Skipped checks in results with reason; skips alone do NOT cause exit code 1 |  |
| EC-003 | Server disconnects mid-run | Partial results in JSON; uncompleted checks as `errored`; exit code 1 | DEC-014 |
| EC-004 | Zero checks executed | Valid JSON with empty `results` array; exit code 0 (no failures) |  |
| EC-005 | JSON output redirected to file vs stdout | Same content regardless of output destination |  |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-HP-001 | 10 checks all pass, server "test-server v1.0", spec "2025-03-26" | `{"summary":{"total":10,"passed":10,"failed":0,"skipped":0,"errored":0},"server":{"name":"test-server","version":"1.0"},"specVersion":"2025-03-26","results":[...]}`, exit code 0 |
| TV-HP-002 | 5 checks: 4 pass, 1 skip (no prompts capability) | `summary.skipped: 1`, skip entry has `"reason": "capability not advertised: prompts"`, exit code 0 |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-EC-001 | Server disconnects after 3 of 8 checks | `summary: {total: 8, passed: 2, failed: 1, skipped: 0, errored: 5}`, remaining 5 as errored with `"message": "server disconnected"`, exit code 1 |
| TV-EC-002 | All checks skipped (server advertises no capabilities) | `summary: {total: 20, passed: 0, failed: 0, skipped: 20, errored: 0}`, exit code 0 |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-ERR-001 | 1 check fails: `tools/call` returns invalid response format | `summary.failed: 1`, failure entry includes `"method": "tools/call"`, `"message": "response schema validation failed: missing 'content' field"`, exit code 1 |
| TV-ERR-002 | Internal test runner error during check | Entry with `"status": "error"`, `"message": "internal: ..."`, exit code 1 |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | JSON output validates against defined report schema | JSON schema validation |
| VP-002 | Summary counts are arithmetically consistent | Automated invariant check |
| VP-003 | Exit code matches summary (0 ↔ no failures/errors) | Automated exit code assertion |
| VP-004 | Timestamps are valid ISO 8601 | Format validation |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-020 (Conformance Report Formats) |
| Domain Invariants | DI-014 (exit code semantics) |
| Edge Cases | DEC-014 |
| Priority | P1 |
| NFRs | — |
