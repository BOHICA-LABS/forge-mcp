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

# BC-8.20.001 — JUnit XML Output Generation

## Summary

Generates valid JUnit XML output from conformance test results. Each
conformance check becomes one `<testcase>` element with timing, failure
messages, and skip reasons. Output conforms to the standard JUnit XML schema
used by CI systems (Jenkins, GitHub Actions, GitLab CI).

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | Conformance test run has completed (fully or partially) |
| PRE-002 | Test results include per-check status (pass/fail/skip/error), timing, and messages |
| PRE-003 | Output path for JUnit XML file is specified or defaults to stdout |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Output is valid XML conforming to JUnit XML schema |
| POST-002 | One `<testcase>` element per conformance check |
| POST-003 | Failed checks include `<failure>` element with descriptive message |
| POST-004 | Skipped checks include `<skipped>` element with reason |
| POST-005 | Errored checks (e.g., disconnect) include `<error>` element, distinct from `<failure>` |
| POST-006 | `<testsuite>` element includes `tests`, `failures`, `errors`, `skipped`, and `time` attributes |
| POST-007 | Each `<testcase>` includes `time` attribute with execution duration in seconds |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | `tests` attribute = count of `<testcase>` elements |
| INV-002 | `failures` + `errors` + `skipped` ≤ `tests` |
| INV-003 | Output is well-formed XML (parseable by any XML parser) |
| INV-004 | Test case names are unique within the suite |

## Edge Cases

| ID | Scenario | Expected Behavior | Source |
|----|----------|-------------------|--------|
| EC-001 | Server disconnects mid-test-run (partial results) | Completed checks written normally; uncompleted checks written as `<skipped>` with reason "server disconnected" | DEC-014 |
| EC-002 | Zero conformance checks completed | Valid XML with `<testsuite tests="0">` and no `<testcase>` children |  |
| EC-003 | Failure message contains XML-special characters (`<`, `>`, `&`) | Characters are properly XML-escaped in output |  |
| EC-004 | Test case name contains non-ASCII characters | UTF-8 encoded in XML output with proper XML declaration |  |
| EC-005 | Very long failure message (> 10KB) | Message is included in full; no truncation (CI systems handle display) |  |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-HP-001 | 3 checks: 2 pass, 1 fail (invalid response format) | `<testsuite tests="3" failures="1" errors="0" skipped="0">` with 3 `<testcase>` elements |
| TV-HP-002 | All 10 checks pass | `<testsuite tests="10" failures="0" errors="0" skipped="0">` with timing on each |
| TV-HP-003 | Single check with 0.5s execution time | `<testcase name="..." time="0.500">` (no inner elements) |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-EC-001 | 5 checks run, server disconnects after 3rd | `<testsuite tests="5" failures="0" errors="0" skipped="2">` with 2 `<skipped message="server disconnected">` |
| TV-EC-002 | Failure message: `Expected <200> but got <500> for method "tools/call"` | XML-escaped: `Expected &lt;200&gt; but got &lt;500&gt; for method &quot;tools/call&quot;` |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-ERR-001 | No test results (empty run) | `<testsuite tests="0" failures="0" errors="0" skipped="0" time="0.000"/>` |
| TV-ERR-002 | Internal test runner error during a check | `<testcase>` with `<error message="internal error: ...">` (distinct from `<failure>`) |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | Output validates against standard JUnit XML schema (XSD) | XML schema validation |
| VP-002 | Attribute counts match child element counts | Automated count verification |
| VP-003 | All XML-special characters in messages are properly escaped | Fuzz test with special characters |
| VP-004 | CI systems (GitHub Actions, Jenkins) can parse the output | Integration test with CI parsers |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-020 (Conformance Report Formats) |
| Domain Invariants | DI-014 (exit code semantics) |
| Edge Cases | DEC-014 |
| Priority | P1 |
| NFRs | — |
