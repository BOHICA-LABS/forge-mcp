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
subsystem: "Config Drift Detection"
capability: "CAP-021"
lifecycle_status: active
introduced: v0.1.0
---

# BC-9.21.001 — Cross-Editor Config Comparison

## Summary

Compares MCP server definitions across all discovered editor configuration
sources (VS Code, Cursor, Claude Desktop, Windsurf, etc.). Identifies
servers present in some sources but not others, and servers with the same
name but different configurations. Handles both `mcpServers` and `servers`
schema variants. Operates in strict read-only mode per DI-015.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | At least two config sources have been discovered and parsed |
| PRE-002 | Each config source has been normalized to a common internal representation |
| PRE-003 | Server names have been extracted from each source for comparison |
| PRE-004 | Config files are readable (file permissions allow read access) |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | All server definitions across all sources have been compared |
| POST-002 | Identical servers (same name, same config) are identified as "in sync" |
| POST-003 | Same-name servers with different configs are flagged as "drifted" |
| POST-004 | Servers missing from one or more sources are flagged as "missing" |
| POST-005 | Servers present in only one source are flagged as "extra" |
| POST-006 | No config files have been modified (DI-015: read-only) |
| POST-007 | Both `mcpServers` and `servers` schema variants are handled transparently |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | Config files are never written to, modified, or created (DI-015) |
| INV-002 | Comparison is symmetric: if A differs from B, then B differs from A |
| INV-003 | Server identity is determined by name, not by position in config file |
| INV-004 | Schema variant (`mcpServers` vs `servers`) does not affect comparison results |

## Edge Cases

| ID | Scenario | Expected Behavior | Source |
|----|----------|-------------------|--------|
| EC-001 | Only one config source found | Report that comparison requires ≥ 2 sources; list the single source found |  |
| EC-002 | Same server name, one uses `mcpServers` schema, other uses `servers` | Compared by normalized representation; schema difference is NOT flagged as drift |  |
| EC-003 | Server present in VS Code settings.json AND VS Code mcp.json | Both sources compared; may show same server defined twice in same editor |  |
| EC-004 | Config file is malformed JSON/YAML | Skip that source with warning; compare remaining sources |  |
| EC-005 | Server name differs by case only (`MyServer` vs `myserver`) | Treated as different servers (names are case-sensitive per MCP spec) |  |
| EC-006 | Config file has no server definitions | Source included in comparison with zero servers |  |
| EC-007 | Environment variable references in config (`${HOME}`, `$PATH`) | Compared as literal strings; no env var expansion (config-level concern) |  |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-HP-001 | VS Code: `{sqlite: {command: "uvx", args: ["mcp-server-sqlite"]}}`, Claude Desktop: `{sqlite: {command: "uvx", args: ["mcp-server-sqlite"]}}` | `sqlite`: in sync across VS Code, Claude Desktop |
| TV-HP-002 | VS Code: `{sqlite: ..., github: ...}`, Cursor: `{sqlite: ...}` | `sqlite`: in sync; `github`: missing from Cursor |
| TV-HP-003 | Three sources all identical | All servers: in sync |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-EC-001 | VS Code: `{sqlite: {command: "uvx", args: ["mcp-server-sqlite", "--db", "test.db"]}}`, Claude Desktop: `{sqlite: {command: "uvx", args: ["mcp-server-sqlite", "--db", "prod.db"]}}` | `sqlite`: drifted — args differ (VS Code: `test.db`, Claude Desktop: `prod.db`) |
| TV-EC-002 | VS Code uses `mcpServers` key, Cursor uses `servers` key, same content | All servers: in sync (schema variant normalized) |
| TV-EC-003 | Only VS Code config found | Warning: "comparison requires ≥ 2 sources"; list VS Code servers |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-ERR-001 | Claude Desktop config is malformed JSON | Warning: "skipping Claude Desktop: parse error at line 5"; compare remaining sources |
| TV-ERR-002 | No config sources found at all | Error: "no MCP config sources found"; exit with appropriate code |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | No config files are modified during comparison | File hash comparison before/after |
| VP-002 | All discovered sources are included in comparison | Source count in output matches discovery count |
| VP-003 | Drift detection is symmetric | A↔B comparison yields same diffs as B↔A |
| VP-004 | Schema normalization does not lose information | Round-trip check: normalized → compared → reported matches original |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-021 (Config Drift Detection) |
| Domain Invariants | DI-015 (read-only — never modify configs) |
| Edge Cases | — |
| Priority | P2 |
| NFRs | — |
