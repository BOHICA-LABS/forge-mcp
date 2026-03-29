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

# BC-5.11.003 — Exit Code Semantics Compliance

## Summary

The CLI binary uses a fixed set of exit codes with deterministic semantics. The
exit code for a given outcome is the same regardless of output format (`--json`,
`--table`, default), verbosity level, or any other presentation flag.

## Exit Code Table

| Code | Meaning | Example |
|------|---------|---------|
| 0 | Success | `forge-mcp list` completes |
| 1 | Test or audit failure (logical failure, not crash) | `forge-mcp test` with failing tests |
| 2 | Connection error | Server unreachable, transport failure |
| 3 | Configuration error | Missing config, invalid TOML, unknown key |
| 4 | Security finding | `forge-mcp audit` found issues |
| 100 | Internal / unexpected error | Panic, unhandled error |

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | The CLI binary is invoked and completes execution (does not receive SIGKILL) |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | The process exits with exactly one code from the exit code table |
| POST-002 | The exit code reflects the most severe outcome (e.g., connection error + test failure = exit 2) |
| POST-003 | Exit code 0 is returned if and only if the operation succeeded without findings or failures |
| POST-004 | Exit code does not change when `--json` is added or removed |
| POST-005 | Exit code does not change when `--verbose` is added or removed |
| POST-006 | Exit code does not change when `--quiet` is added or removed |

## Invariants

| ID | Invariant |
|----|-----------|
| DI-014 | Exit codes are deterministic for a given outcome and do not vary by output format or verbosity |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Multiple error categories in one run (e.g., some servers connect, some don't) | Exit with highest-priority code (2 > 1 for connection+test) |
| EC-002 | `forge-mcp audit` with both security findings and connection error | Exit 2 (connection error takes priority over findings) |
| EC-003 | `forge-mcp test --json` vs `forge-mcp test` with same failing test | Same exit code (1) regardless of output format |
| EC-004 | `forge-mcp list` with `--verbose` vs without | Same exit code (0) for same result |
| EC-005 | Panic / unhandled error in any subcommand | Exit 100, never 0 |
| EC-006 | SIGTERM received during execution | Exit with non-zero (implementation-defined, not 0) |
| EC-007 | Config error prevents connection attempt | Exit 3 (config), not 2 (connection) |
| EC-008 | `forge-mcp audit` with zero findings | Exit 0 (clean audit) |

## Canonical Test Vectors

### Happy Path

| Input | Scenario | Exit Code |
|-------|----------|-----------|
| `forge-mcp list` | All servers discovered | 0 |
| `forge-mcp test srv` | All tests pass | 0 |
| `forge-mcp audit srv` | No findings | 0 |
| `forge-mcp info srv` | Server reachable | 0 |
| `forge-mcp config show` | Valid config | 0 |

### Edge Case

| Input | Scenario | Exit Code |
|-------|----------|-----------|
| `forge-mcp test srv` | 2 of 5 tests fail | 1 |
| `forge-mcp test srv --json` | 2 of 5 tests fail (JSON output) | 1 |
| `forge-mcp audit srv` | 3 security findings | 4 |
| `forge-mcp audit srv --json` | 3 security findings (JSON output) | 4 |

### Error

| Input | Scenario | Exit Code |
|-------|----------|-----------|
| `forge-mcp call unreachable tool` | Connection refused | 2 |
| `forge-mcp list --config /nonexistent` | Config file missing | 3 |
| `forge-mcp call srv tool` | Internal panic | 100 |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | For every subcommand, adding `--json` does not change the exit code | Property test |
| VP-002 | For every subcommand, adding `--verbose` does not change the exit code | Property test |
| VP-003 | Exit code 0 is never returned when an error occurred | Invariant test |
| VP-004 | Exit code 100 is returned for all unhandled/panic scenarios | Fault injection test |
| VP-005 | Priority ordering: 100 > 3 > 2 > 4 > 1 > 0 | Unit test |

## Traceability

| Source | Target |
|--------|--------|
| CAP-011 | This contract |
| DI-014 | Invariant (exit code determinism) |
| BC-5.11.001 | Subcommand dispatch (each handler returns appropriate code) |
| BC-5.11.002 | JSON output (exit code must not vary with format) |
