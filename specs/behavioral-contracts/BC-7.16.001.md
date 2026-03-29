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
capability: "CAP-016"
lifecycle_status: active
introduced: v0.1.0
---

# BC-7.16.001 — Dangerous Tool Pattern Detection

## Summary

The security auditor scans MCP tool metadata (name, description, input/output
schemas) for patterns indicating dangerous capabilities: filesystem access,
code execution, and network operations. Each finding includes specific evidence,
severity classification, and OWASP AST10 category mapping.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | Tool metadata has been retrieved from the server via `tools/list` |
| PRE-002 | Each tool has at minimum: name, description, and inputSchema |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Tools with filesystem patterns (read, write, delete, create, mkdir, rmdir, path, file) in description or schema are flagged |
| POST-002 | Tools with execution patterns (exec, eval, shell, run, command, subprocess, spawn) in description or schema are flagged |
| POST-003 | Tools with network patterns (http, https, fetch, request, curl, dns, socket, connect, url) in description or schema are flagged |
| POST-004 | Each finding has a severity: critical (exec), high (fs write/delete), medium (fs read, network), low (indirect indicators), info (informational) |
| POST-005 | Each finding maps to OWASP AST10: AST01 (Malicious Skills) for exec/network, AST03 (Over-Privileged) for broad fs access |
| POST-006 | Each finding includes evidence: tool name, matched text from description, matched schema field path |
| POST-007 | Pattern matching is case-insensitive |
| POST-008 | A tool matching multiple patterns produces one finding per pattern category (not deduplicated across categories) |

## Invariants

| ID | Invariant |
|----|-----------|
| DI-010 | Every security finding must include specific evidence (tool name, description text, or schema field that triggered the finding) |

## Pattern Categories

| Category | Patterns | Severity | AST10 |
|----------|----------|----------|-------|
| Code Execution | exec, eval, shell, run, command, subprocess, spawn, compile, interpret | Critical | AST01 |
| Filesystem Write | write, delete, create, mkdir, rmdir, move, rename, chmod, truncate | High | AST03 |
| Filesystem Read | read, open, list, glob, stat, exists, find | Medium | AST03 |
| Network Outbound | http, https, fetch, request, curl, post, get (HTTP), put (HTTP), dns, socket | Medium | AST01 |
| Network Listen | listen, serve, bind, accept | High | AST01 |
| Database | sql, query, insert, update, delete (DB context), drop, migrate | High | AST03 |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Tool named "read_file" with description "Reads a file from disk" | Flagged: fs read (medium), evidence: name="read_file", desc contains "file from disk" |
| EC-002 | Tool named "search" with description "Searches the codebase" | Not flagged for fs (no direct fs pattern); "search" alone is not a dangerous pattern |
| EC-003 | Tool with "execute" in description but context is "execute a search query" | Flagged: exec pattern matched. Confidence < 1.0 because context suggests non-dangerous use. Human review recommended. |
| EC-004 | Tool with empty description | Only schema-based patterns checked; no description-based findings |
| EC-005 | Tool with deeply nested schema field `input.config.options.shell` | Detected: schema path `input.config.options.shell` matches exec pattern |
| EC-006 | Tool description in non-English language | Pattern matching on English keywords only; non-English descriptions produce info-level "unanalyzable description" finding |
| EC-007 | Tool with 100+ schema fields | All fields scanned; no truncation |
| EC-008 | Tool name contains "execute" but is "execute_query" (DB) | Flagged under both exec and database categories |

## Canonical Test Vectors

### Happy Path

| Tool Name | Description | Schema | Findings |
|-----------|------------|--------|----------|
| `run_command` | "Runs a shell command" | `{cmd: string}` | Critical: exec pattern (name: "run", desc: "shell command") |
| `read_file` | "Reads file contents" | `{path: string}` | Medium: fs read (name: "read_file", desc: "file contents") |
| `http_request` | "Makes an HTTP request" | `{url: string, method: string}` | Medium: network (name: "http_request", desc: "HTTP request") |
| `calculator` | "Performs math" | `{expr: string}` | No findings |

### Edge Case

| Tool Name | Description | Schema | Findings |
|-----------|------------|--------|----------|
| `search` | "Searches codebase" | `{query: string}` | No findings (search is not a dangerous pattern) |
| `analyze` | "Analyzes and executes plan" | `{plan: object}` | Critical: exec pattern in desc ("executes"), confidence < 1.0 |

### Error

| Scenario | Expected Behavior |
|----------|-------------------|
| Tool metadata missing description | Schema-only scan; info finding "missing description" |
| Tool metadata missing inputSchema | Description-only scan; info finding "missing schema" |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | Known-dangerous tools (run_command, exec_code, write_file) always produce findings | Unit test |
| VP-002 | Known-safe tools (calculator, get_time, list_models) produce no findings | Negative test |
| VP-003 | Every finding includes non-empty evidence (tool name + matched text) | Property test |
| VP-004 | Pattern matching is case-insensitive | Unit test |
| VP-005 | Nested schema fields are scanned to arbitrary depth | Unit test |

## Traceability

| Source | Target |
|--------|--------|
| CAP-016 | This contract |
| DI-010 | Invariant (evidence required) |
| AST01 | OWASP AST10 mapping (Malicious Skills) |
| AST03 | OWASP AST10 mapping (Over-Privileged Skills) |
| BC-7.16.004 | Severity classification (findings feed into classification) |
| BC-7.18.001 | Audit report (findings appear in report) |
| BC-7.18.003 | AST10 coverage (this contract covers AST01, AST03) |
