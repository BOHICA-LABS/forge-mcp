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

# BC-7.17.001 — Permission Escalation Detection

## Summary

The security auditor detects when an MCP server requests capabilities beyond
what was declared during the `initialize` handshake, or when tools attempt to
access resources outside their declared scope. This maps to OWASP AST03
(Over-Privileged Skills).

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | The `initialize` response from the server has been captured, including declared `capabilities` |
| PRE-002 | Subsequent server behavior (requests, notifications) is being monitored |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | The server's declared capabilities from `initialize` are recorded as the authorized set |
| POST-002 | Any server request that requires a capability not in the authorized set is flagged |
| POST-003 | Capability escalation findings have severity: critical |
| POST-004 | Findings include evidence: the undeclared capability, the request that exercised it |
| POST-005 | Tool calls that access file paths outside declared `roots` are flagged |
| POST-006 | Root boundary findings include: the attempted path, the declared root, and the tool name |
| POST-007 | Findings map to AST03 (Over-Privileged Skills) |

## Capability Tracking

| MCP Capability | Meaning | Escalation if Undeclared |
|---------------|---------|--------------------------|
| `tools` | Server can provide tools | Server sends tools/* without declaring |
| `resources` | Server can provide resources | Server sends resources/* without declaring |
| `prompts` | Server can provide prompts | Server sends prompts/* without declaring |
| `logging` | Server can send log messages | Server sends logging/* without declaring |
| `experimental` | Server uses experimental features | Server uses undeclared experimental methods |

## Invariants

| ID | Invariant |
|----|-----------|
| DI-010 | Every security finding must include specific evidence |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Server declares `tools` capability, then sends `resources/read` request | Critical finding: `resources` not declared |
| EC-002 | Server declares all capabilities | No escalation findings for capability usage |
| EC-003 | Server sends a custom/experimental method not in spec | Info finding: unknown method (may be extension) |
| EC-004 | Tool accesses `/tmp/data.txt` when root is `/home/user/project` | High finding: path outside root |
| EC-005 | Tool accesses `/home/user/project/../../../etc/passwd` | High finding: path traversal detected (canonicalized path is outside root) |
| EC-006 | Tool accesses `/home/user/project/subdir/file.txt` when root is `/home/user/project` | No finding: within root |
| EC-007 | Server with no declared roots | Root enforcement is not applicable; no root-related findings (tools may access any path) |
| EC-008 | Initialize response is missing `capabilities` field entirely | Treat as empty capabilities; any capability usage is escalation |
| EC-009 | Server disconnects and reconnects with different capabilities | New `initialize` establishes new baseline; no cross-session comparison (see BC-7.16.003 for drift) |

## Canonical Test Vectors

### Happy Path

| Declared Capabilities | Server Action | Findings |
|----------------------|---------------|----------|
| `{tools: {}}` | `tools/list` response | None (authorized) |
| `{tools: {}, resources: {}}` | `resources/read` | None (authorized) |
| `{tools: {}}` | `resources/read` | Critical: undeclared capability `resources` |

### Root Boundary

| Declared Roots | Tool Path | Findings |
|---------------|-----------|----------|
| `["/home/user/project"]` | `/home/user/project/src/main.rs` | None (within root) |
| `["/home/user/project"]` | `/etc/passwd` | High: outside root |
| `["/home/user/project"]` | `/home/user/project/../../etc/passwd` | High: path traversal |
| `[]` (empty roots) | Any path | No findings (no roots = no enforcement) |

### Error

| Scenario | Expected Behavior |
|----------|-------------------|
| Cannot parse `initialize` response | Warning: unable to establish capability baseline; all subsequent checks flagged as info |
| Root path is relative (not absolute) | Warning: root should be absolute; finding for ambiguous root configuration |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | Undeclared capability usage always produces a critical finding | Unit test |
| VP-002 | Declared capability usage never produces a finding | Negative test |
| VP-003 | Path traversal (../) is detected after canonicalization | Unit test |
| VP-004 | Paths within declared roots produce no findings | Negative test |
| VP-005 | Empty capabilities set means all usage is escalation | Unit test |
| VP-006 | No-roots servers do not produce root boundary findings | Unit test |

## Traceability

| Source | Target |
|--------|--------|
| CAP-017 | This contract |
| DI-010 | Invariant (evidence required) |
| AST03 | OWASP AST10 mapping (Over-Privileged Skills) |
| BC-7.17.002 | Root enforcement (complementary, this detects, that verifies) |
| BC-7.16.004 | Severity classification (escalation findings classified here) |
| BC-7.18.001 | Audit report (escalation findings in report) |
