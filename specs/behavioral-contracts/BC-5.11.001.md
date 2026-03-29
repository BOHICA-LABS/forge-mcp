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

# BC-5.11.001 — Subcommand Dispatch

## Summary

The CLI binary parses the first positional argument as a subcommand and routes
execution to the corresponding handler. Invalid or missing subcommands produce
a diagnostic on stderr and exit with the appropriate code.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | The `forge-mcp` binary is invoked with zero or more arguments |
| PRE-002 | The process has a valid stdout and stderr file descriptor |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | A recognized subcommand (`list`, `call`, `info`, `grep`, `test`, `audit`, `tui`, `daemon`, `config`, `diff`) is routed to its handler |
| POST-002 | An unrecognized subcommand produces a diagnostic message on stderr listing valid subcommands |
| POST-003 | A missing subcommand (bare `forge-mcp`) prints usage help on stderr and exits with code 1 |
| POST-004 | Required arguments for the dispatched subcommand are validated before handler invocation |
| POST-005 | Missing required arguments produce a diagnostic on stderr naming the missing argument |

## Invariants

| ID | Invariant |
|----|-----------|
| DI-013 | JSON data output goes to stdout; human-readable diagnostics go to stderr |
| DI-014 | Exit codes are deterministic for a given outcome and do not vary by output format or verbosity |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Empty argument list (bare `forge-mcp`) | Print usage help on stderr, exit 1 |
| EC-002 | Unrecognized subcommand (`forge-mcp frobnicate`) | Print "unknown subcommand 'frobnicate'" on stderr with list of valid subcommands, exit 1 |
| EC-003 | Subcommand with missing required arg (`forge-mcp call` without tool name) | Print missing-argument diagnostic on stderr, exit 1 |
| EC-004 | Subcommand with `--help` flag (`forge-mcp list --help`) | Print subcommand-specific help on stderr, exit 0 |
| EC-005 | Subcommand with extra unknown flags (`forge-mcp list --banana`) | Print unknown-flag diagnostic on stderr, exit 1 |
| EC-006 | Case sensitivity (`forge-mcp LIST`) | Reject as unrecognized (subcommands are lowercase only) |
| EC-007 | Subcommand after global flags (`forge-mcp --verbose list`) | Global flags parsed before subcommand; dispatch still works |
| EC-008 | Double-dash separator (`forge-mcp -- list`) | `list` treated as positional, not subcommand; behavior defined per parser |

## Canonical Test Vectors

### Happy Path

| Input | Expected Output | Exit Code |
|-------|----------------|-----------|
| `forge-mcp list` | JSON array of servers on stdout | 0 |
| `forge-mcp call myserver tool_name '{"arg":1}'` | Tool result JSON on stdout | 0 |
| `forge-mcp info myserver` | Server info JSON on stdout | 0 |
| `forge-mcp grep "pattern"` | Matching tools JSON on stdout | 0 |
| `forge-mcp test myserver` | Test results on stdout | 0 (all pass) or 1 (failures) |
| `forge-mcp audit myserver` | Audit report on stdout | 0 (clean) or 4 (findings) |
| `forge-mcp config show` | Config JSON on stdout | 0 |
| `forge-mcp diff server1 server2` | Diff output on stdout | 0 |

### Edge Case

| Input | Expected Output | Exit Code |
|-------|----------------|-----------|
| `forge-mcp` | Usage help on stderr | 1 |
| `forge-mcp --help` | Full help on stderr | 0 |
| `forge-mcp frobnicate` | Unknown subcommand diagnostic on stderr | 1 |
| `forge-mcp call` | Missing argument diagnostic on stderr | 1 |

### Error

| Input | Expected Output | Exit Code |
|-------|----------------|-----------|
| `forge-mcp list --config /nonexistent.toml` | Config file not found on stderr | 3 |
| `forge-mcp call unreachable_server tool` | Connection error on stderr | 2 |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | All 10 subcommands are routable and reach their handler | Exhaustive test |
| VP-002 | Unrecognized subcommands never reach any handler | Negative test |
| VP-003 | Missing required arguments are caught before handler invocation | Unit test |
| VP-004 | Exit codes match the exit code semantics table (BC-5.11.003) | Cross-reference test |

## Traceability

| Source | Target |
|--------|--------|
| CAP-011 | This contract |
| DI-013 | Invariant (stdout/stderr separation) |
| DI-014 | Invariant (exit code determinism) |
| BC-5.11.003 | Exit code semantics |
