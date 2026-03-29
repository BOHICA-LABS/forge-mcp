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
subsystem: "CLI Mode"
capability: "CAP-011"
lifecycle_status: active
introduced: v0.1.0
---

# BC-5.11.002 — Structured JSON Output on stdout

## Summary

When the `--output json` or `--json` flag is present, all data output is emitted
as valid, parseable JSON on stdout. Human-readable diagnostics (progress, warnings,
errors) are emitted exclusively on stderr. This separation enables reliable piping
and machine consumption.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | The user invokes a subcommand with `--output json` or `--json` flag |
| PRE-002 | stdout is a valid file descriptor (may be a pipe or file) |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | stdout contains exactly one JSON value (object or array) per invocation |
| POST-002 | The JSON on stdout is valid per RFC 8259 |
| POST-003 | The JSON on stdout is parseable by `jq .` without error |
| POST-004 | stderr contains zero JSON — only human-readable diagnostics |
| POST-005 | No ANSI escape codes appear on stdout |
| POST-006 | No partial JSON is written to stdout on error — either complete JSON or nothing |

## Invariants

| ID | Invariant |
|----|-----------|
| DI-013 | JSON data output goes to stdout; human-readable diagnostics go to stderr |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | `--json` with a subcommand that has no data output (e.g., `config set`) | Emit `{}` or `{"status":"ok"}` on stdout, not empty |
| EC-002 | Error during execution with `--json` | Emit error JSON on stdout (`{"error":...}`), diagnostic on stderr |
| EC-003 | `--json` combined with `--verbose` | Verbose diagnostics on stderr only; stdout JSON unaffected |
| EC-004 | stdout is a closed pipe (broken pipe) | Exit gracefully, do not panic; write diagnostic to stderr |
| EC-005 | Very large result set with `--json` | Complete JSON array emitted (no truncation without explicit `--limit`) |
| EC-006 | `--json` without subcommand | Usage help on stderr (no JSON on stdout), exit 1 |
| EC-007 | Binary/non-UTF-8 data in tool response | Escape per JSON spec (\\uXXXX) or base64-encode; never emit invalid JSON |

## Canonical Test Vectors

### Happy Path

| Input | stdout | stderr | Exit Code |
|-------|--------|--------|-----------|
| `forge-mcp list --json` | `[{"name":"server1","transport":"stdio",...}]` | (empty or progress) | 0 |
| `forge-mcp info myserver --json` | `{"name":"myserver","tools":[...],...}` | (empty or progress) | 0 |
| `forge-mcp call srv tool '{}' --json` | `{"result":{"content":[...]}}` | (empty or progress) | 0 |

### Edge Case

| Input | stdout | stderr | Exit Code |
|-------|--------|--------|-----------|
| `forge-mcp list --json \| jq .` | Valid parsed JSON | — | 0 |
| `forge-mcp config set key val --json` | `{"status":"ok"}` | (empty) | 0 |
| `forge-mcp list --json --verbose` | JSON only | Verbose diagnostics | 0 |

### Error

| Input | stdout | stderr | Exit Code |
|-------|--------|--------|-----------|
| `forge-mcp call unreachable tool --json` | `{"error":"connection_error","message":"..."}` | Connection diagnostic | 2 |
| `forge-mcp list --json --config /bad` | `{"error":"config_error","message":"..."}` | Config diagnostic | 3 |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | Every subcommand with `--json` produces valid JSON on stdout | Exhaustive test |
| VP-002 | `jq .` succeeds on stdout for every subcommand | Integration test |
| VP-003 | stderr never contains JSON when `--json` is active | Negative test |
| VP-004 | stdout never contains ANSI escape codes | Property test |
| VP-005 | Broken pipe does not cause panic | Fault injection test |

## Traceability

| Source | Target |
|--------|--------|
| CAP-011 | This contract |
| DI-013 | Invariant (stdout/stderr separation) |
| BC-5.11.001 | Subcommand dispatch (prerequisite) |
| BC-5.12.002 | Pipeable output (complementary) |
