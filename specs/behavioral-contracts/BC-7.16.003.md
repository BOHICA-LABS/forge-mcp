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

# BC-7.16.003 — Schema Drift / Rug Pull Detection

## Summary

The security auditor records a cryptographic hash of each tool's **canonical
metadata** (name, description, inputSchema) at first connection. Separately, it
stores a hash of **auxiliary metadata** (annotations). On subsequent connections
or `tools/list_changed` notifications, the auditor compares current metadata
against the stored baseline. Changes to canonical metadata produce high-severity
findings; changes to auxiliary metadata (annotations only) produce info-level
findings. This detects "rug pull" attacks where a tool's capabilities change
silently, while still tracking annotation drift at lower severity.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | Tool metadata has been retrieved from the server |
| PRE-002 | A baseline hash store is available (in-memory or persisted) |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | On first connection to a server, a SHA-256 hash of each tool's canonical metadata is computed and stored |
| POST-002 | Canonical metadata for hashing includes: name, description, inputSchema (JSON-canonicalized). Annotations are **excluded** from the canonical hash. |
| POST-002a | Auxiliary metadata (annotations) is stored separately and compared on re-connection. Annotation-only changes produce info-level findings (see EC-009). |
| POST-003 | On subsequent connections, current canonical hashes are compared against stored baselines |
| POST-004 | Any canonical hash mismatch produces a finding with severity ≥ high |
| POST-005 | The finding includes: tool name, changed fields (diff), old hash, new hash |
| POST-006 | When `tools/list_changed` notification is received, a re-scan is triggered |
| POST-007 | New tools (not in baseline) produce an info-level "new tool added" finding |
| POST-008 | Removed tools (in baseline but not in current) produce a medium-level "tool removed" finding |
| POST-009 | Findings map to AST04 (Insecure Metadata) and AST07 (Update Drift) |

## Hash Computation

### Canonical Hash (name, description, inputSchema)

```
canonical_metadata = JSON.stringify({
  name: tool.name,
  description: tool.description,
  inputSchema: canonicalize(tool.inputSchema)
}, keys_sorted)

canonical_hash = SHA-256(canonical_metadata)
```

### Auxiliary Hash (annotations only)

```
auxiliary_metadata = JSON.stringify({
  annotations: canonicalize(tool.annotations)
}, keys_sorted)

auxiliary_hash = SHA-256(auxiliary_metadata)
```

Annotations are compared separately because they are auxiliary hints (e.g.,
`readOnlyHint`, `destructiveHint`) that do not change a tool's functional
contract. A canonical hash mismatch is a high-severity finding (potential rug
pull); an auxiliary hash mismatch is an info-level finding (annotation drift).

JSON canonicalization: keys sorted alphabetically at all nesting levels, no
whitespace, null values included (they are part of the schema contract).

## Invariants

| ID | Invariant |
|----|-----------|
| DI-010 | Every security finding must include specific evidence |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Tool description changes from "Read file" to "Read and write file" | High finding: description changed, evidence includes old/new description |
| EC-002 | Tool inputSchema adds a new required field | High finding: schema changed, evidence includes diff |
| EC-003 | Tool inputSchema reorders keys but content is identical | No finding (canonical JSON is order-independent) |
| EC-004 | `tools/list_changed` sent but no actual changes | Re-scan finds no diff; no findings |
| EC-005 | `tools/list_changed` NOT sent but tools actually changed (FM-020) | Detected on next manual scan or periodic check; finding includes note about missing notification |
| EC-006 | Server sends `tools/list_changed` and changes 20 tools simultaneously | 20 individual findings, one per changed tool |
| EC-007 | Baseline store is empty (fresh install, no history) | First connection establishes baseline; no findings |
| EC-008 | Server restarts and tools are identical | No findings (hashes match) |
| EC-009 | Tool annotations change but name/description/schema don't | Info-level finding (annotations are auxiliary metadata) |
| EC-010 | Risk R-014: schema drift enables privilege escalation | High finding with R-014 risk reference in evidence |

## Canonical Test Vectors

### Happy Path — No Drift

| Baseline Tools | Current Tools | Findings |
|---------------|---------------|----------|
| `[{name:"read_file", desc:"Read", schema:{path:string}}]` | `[{name:"read_file", desc:"Read", schema:{path:string}}]` | None |

### Happy Path — Drift Detected

| Baseline Tools | Current Tools | Findings |
|---------------|---------------|----------|
| `[{name:"read_file", desc:"Read", schema:{path:string}}]` | `[{name:"read_file", desc:"Read and execute", schema:{path:string, cmd:string}}]` | High: description changed + schema changed |

### Edge Case

| Scenario | Findings |
|----------|----------|
| New tool `write_file` added (not in baseline) | Info: "new tool added: write_file" |
| Tool `read_file` removed from server | Medium: "tool removed: read_file" |
| Schema keys reordered, content identical | None (canonical JSON) |
| `tools/list_changed` but no actual changes | None |

### Error

| Scenario | Expected Behavior |
|----------|-------------------|
| Cannot compute hash (malformed schema) | Warning logged; tool flagged with info finding "unhashable metadata" |
| Baseline store corrupted | Re-establish baseline from current; warning on stderr |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | Identical metadata across connections produces zero findings | Unit test |
| VP-002 | Any single-character change in description produces a finding | Unit test |
| VP-003 | Schema key reordering does not produce a finding | Unit test (canonical JSON) |
| VP-004 | New tools produce info-level findings | Unit test |
| VP-005 | Removed tools produce medium-level findings | Unit test |
| VP-006 | SHA-256 hash is used (not MD5 or other weak hash) | Code review |
| VP-007 | `tools/list_changed` triggers re-scan | Integration test |

## Traceability

| Source | Target |
|--------|--------|
| CAP-016 | This contract |
| DI-010 | Invariant (evidence required) |
| AST04 | OWASP AST10 mapping (Insecure Metadata) |
| AST07 | OWASP AST10 mapping (Update Drift) |
| FM-020 | Failure mode (drift without notification) |
| R-014 | Risk (schema drift enables escalation) |
| BC-7.16.001 | Dangerous pattern detection (re-scan after drift for new dangerous patterns) |
| BC-7.18.001 | Audit report (drift findings in report) |
