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
capability: "CAP-012"
lifecycle_status: active
introduced: v0.1.0
---

# BC-5.12.002 — Pipeable Output for Shell Composition

## Summary

CLI output on stdout is clean for piping to `jq`, `grep`, `awk`, and other
Unix tools. No ANSI escape codes, progress indicators, or interactive elements
appear on stdout. The CLI also accepts input from stdin for tool arguments,
enabling full shell pipeline composition.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | The CLI binary is invoked with stdout potentially redirected to a pipe or file |
| PRE-002 | stdin may be a pipe providing input data |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | stdout contains zero ANSI escape codes (no color, no cursor movement) |
| POST-002 | stdout contains zero carriage returns used for progress indication |
| POST-003 | Progress indicators (spinners, progress bars) appear only on stderr |
| POST-004 | `forge-mcp list --json \| jq '.[] .name'` produces one server name per line |
| POST-005 | `forge-mcp call srv tool --json \| jq .result` extracts the result field |
| POST-006 | `echo '{"path":"/tmp/x"}' \| forge-mcp call srv read_file --json --stdin` reads args from stdin |
| POST-007 | When stdout is a TTY and `--json` is not set, human-friendly formatting is allowed on stdout |

## Invariants

| ID | Invariant |
|----|-----------|
| DI-013 | JSON data output goes to stdout; human-readable diagnostics go to stderr |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | stdout is a pipe (not TTY) without `--json` | Default to compact output (auto-detect non-TTY) or require `--json` |
| EC-002 | stderr is redirected to /dev/null | Progress silenced; stdout data unaffected |
| EC-003 | stdin is a pipe with invalid JSON | Emit parse error on stderr, exit 3 |
| EC-004 | stdin is a pipe with valid JSON, but wrong schema for the tool | Forward to server; server returns tool error |
| EC-005 | Very large stdin input (> 1MB) | Stream to server without buffering entire input in memory (if transport supports it) |
| EC-006 | SIGPIPE received (downstream consumer closed pipe) | Exit gracefully without panic, no output on stdout after SIGPIPE |
| EC-007 | Combined pipeline: `forge-mcp list --json \| jq -r '.[].name' \| xargs -I{} forge-mcp info {} --json` | Each invocation produces clean JSON; no interleaving |
| EC-008 | `--color always` flag forced with pipe | ANSI codes on stderr only; stdout remains clean |

## Canonical Test Vectors

### Happy Path

| Pipeline | Expected Result |
|----------|----------------|
| `forge-mcp list --json \| jq length` | Integer count of servers |
| `forge-mcp info srv --json \| jq '.tools[].name'` | One tool name per line |
| `echo '{"query":"test"}' \| forge-mcp call srv search --json --stdin` | Tool result JSON |
| `forge-mcp audit srv --json \| jq '.findings[] \| select(.severity=="critical")'` | Filtered findings |

### Edge Case

| Pipeline | Expected Result |
|----------|----------------|
| `forge-mcp list --json \| head -c 1` | First byte of JSON (`[`); SIGPIPE handled gracefully |
| `forge-mcp list --json 2>/dev/null` | JSON on stdout, no stderr output |
| `echo 'not json' \| forge-mcp call srv tool --json --stdin` | Error on stderr, exit 3 |

### Error

| Pipeline | Expected Result | Exit Code |
|----------|----------------|-----------|
| `echo '{}' \| forge-mcp call unreachable tool --json --stdin` | Connection error JSON on stdout, diagnostic on stderr | 2 |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | stdout output contains zero bytes matching ANSI escape regex `\x1b\[[\d;]*m` | Property test |
| VP-002 | `jq .` succeeds on stdout of every subcommand with `--json` | Integration test |
| VP-003 | stdin JSON is correctly forwarded as tool arguments | Integration test |
| VP-004 | SIGPIPE causes graceful exit (no panic, no core dump) | Signal test |
| VP-005 | No progress indicator bytes appear on stdout | Property test |

## Traceability

| Source | Target |
|--------|--------|
| CAP-012 | This contract |
| DI-013 | Invariant (stdout/stderr separation) |
| BC-5.11.002 | JSON output (this contract extends pipeability) |
| BC-5.12.001 | Compact output (complementary) |
