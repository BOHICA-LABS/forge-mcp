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
capability: "CAP-017"
lifecycle_status: active
introduced: v0.1.0
---

# BC-7.17.002 — Root Enforcement Compliance Verification

## Summary

The security auditor verifies that MCP server tools respect the declared `roots`
boundaries. File paths in tool arguments and responses are inspected to detect
access outside root directories. This maps to OWASP AST06 (Weak Isolation).

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | The server declared one or more `roots` during initialization |
| PRE-002 | Tool call arguments and responses are available for inspection |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | File paths in tool arguments (`inputSchema` fields of type string that represent paths) are checked against declared roots |
| POST-002 | File paths in tool responses are checked against declared roots |
| POST-003 | A path is "within roots" if its canonicalized absolute form starts with one of the declared root prefixes |
| POST-004 | Paths outside all declared roots produce a high-severity finding |
| POST-005 | Path traversal sequences (`../`, `..\\`) are detected before canonicalization as an additional signal |
| POST-006 | Symlinks that resolve outside roots are flagged (if detectable from metadata) |
| POST-007 | Findings include: the violating path, the declared roots, and the tool name |
| POST-008 | Findings map to AST06 (Weak Isolation) |

## Path Resolution Rules

1. Relative paths are resolved against the first declared root
2. `../` sequences are resolved via canonicalization
3. `~` (home directory) is expanded before comparison
4. Symlink targets are checked if the tool response includes resolved paths
5. Windows-style paths (`C:\`, `\\server\share`) are normalized to forward slashes

## Invariants

| ID | Invariant |
|----|-----------|
| DI-010 | Every security finding must include specific evidence |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Tool arg: `../../../etc/passwd` with root `/home/user/proj` | High finding: canonicalizes to `/etc/passwd`, outside root |
| EC-002 | Tool arg: `./src/main.rs` with root `/home/user/proj` | No finding: resolves to `/home/user/proj/src/main.rs` |
| EC-003 | Tool response contains `/var/log/syslog` with root `/home/user/proj` | High finding: response path outside root |
| EC-004 | Multiple roots declared: `["/home/user/proj", "/tmp/scratch"]` | Path valid if within ANY declared root |
| EC-005 | Tool arg is a URL, not a file path (`https://example.com`) | Not a root enforcement issue; skip (network patterns handled by BC-7.16.002) |
| EC-006 | Tool arg contains null bytes (`/home/user/proj\x00/../../etc/passwd`) | High finding: null byte injection attempt |
| EC-007 | Server declares no roots | Root enforcement is disabled; no findings generated (documented in BC-7.17.001 EC-007) |
| EC-008 | Tool schema has no path-like fields | No path checking needed; no findings |
| EC-009 | Path contains Unicode normalization attack (`/home/user/proj/ﬁle.txt` vs `file.txt`) | Paths are compared after NFC normalization |
| EC-010 | Windows path `C:\Users\proj\..\..\..\Windows\System32` | Canonicalized to `C:\Windows\System32`; outside root if root is `C:\Users\proj` |

## Canonical Test Vectors

### Happy Path

| Root | Tool Arg/Response Path | Finding |
|------|----------------------|---------|
| `/home/user/proj` | `/home/user/proj/src/lib.rs` | None |
| `/home/user/proj` | `/home/user/proj/deep/nested/file.txt` | None |
| `["/home/user/proj", "/tmp"]` | `/tmp/cache.json` | None |

### Edge Case — Violations

| Root | Tool Arg/Response Path | Finding | Severity |
|------|----------------------|---------|----------|
| `/home/user/proj` | `/etc/passwd` | Root boundary violation | High |
| `/home/user/proj` | `../../../etc/shadow` | Path traversal + root violation | High |
| `/home/user/proj` | `/home/user/other-proj/secret.key` | Root boundary violation | High |
| `/home/user/proj` | `/home/user/proj` (the root itself) | None (root directory is within bounds) |

### Error

| Scenario | Expected Behavior |
|----------|-------------------|
| Path is empty string | Skip check; info finding "empty path in tool argument" |
| Path contains invalid characters | Warning; attempted canonicalization; finding if resolution fails |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | Paths within declared roots never produce findings | Negative test |
| VP-002 | Paths outside all declared roots always produce findings | Unit test |
| VP-003 | Path traversal (`../`) is detected and resolved correctly | Unit test |
| VP-004 | Multiple roots are checked (path valid if in ANY root) | Unit test |
| VP-005 | Null byte injection is detected | Security test |
| VP-006 | No-roots servers produce zero root enforcement findings | Unit test |

## Traceability

| Source | Target |
|--------|--------|
| CAP-017 | This contract |
| DI-010 | Invariant (evidence required) |
| AST06 | OWASP AST10 mapping (Weak Isolation) |
| BC-7.17.001 | Permission escalation (complementary: escalation detects undeclared capabilities, this verifies root boundaries) |
| BC-7.16.004 | Severity classification (root violations classified here) |
| BC-7.18.001 | Audit report (root violations in report) |
