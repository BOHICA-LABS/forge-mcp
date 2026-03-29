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
subsystem: "Server Discovery & Connection Management"
capability: "CAP-001"
lifecycle_status: active
introduced: v0.1.0
---

# BC-1.01.002 — Dual-Schema Config Parsing (mcpServers vs servers)

## Summary

Parses MCP configuration files supporting two distinct JSON schemas: the `"mcpServers"` schema (Claude Desktop, Cursor, Windsurf) and the `"servers"` schema with explicit `"type"` field (VS Code). Normalizes both into a unified internal `ServerEntry` representation.

## Preconditions

- PRE-001: A `DiscoveredConfig` with `exists: true` and no `access_error` has been provided by BC-1.01.001.
- PRE-002: The config file is valid UTF-8 text.

## Postconditions

- POST-001: For `"mcpServers"` schema: each key under `mcpServers` becomes a `ServerEntry` with `name` = key, `transport` inferred from presence of `command` (stdio) or `url` (HTTP).
- POST-002: For `"servers"` schema: each entry under `servers` becomes a `ServerEntry` with `transport` determined by the explicit `"type"` field (`"stdio"` or `"sse"`).
- POST-003: All `ServerEntry` records include: `name`, `transport` (Stdio | Http), `config` (transport-specific fields), `source_editor`, `source_path`, `enabled` (default true), `always_allow` (list, default empty).
- POST-004: The `disabled` field (Cursor-specific) is mapped to `enabled = !disabled`.
- POST-005: The `alwaysAllow` field (Cursor-specific) is mapped to `always_allow` list.
- POST-006: Environment variables in `env` are stored as-is (not expanded). Expansion happens at connection time (BC-1.02.001).

## Invariants

- **DI-015**: Config files are read-only. Parsing MUST NOT modify the source file.

## Edge Cases

| ID | Condition | Expected Behavior |
|----|-----------|-------------------|
| EC-001 | DEC-010: Same server name appears in both `mcpServers` and `servers` keys within a single file | Treat as a conflict. Return both entries with a warning flag. Conflict resolution is deferred to BC-1.01.003. |
| EC-002 | FM-007: Config file contains invalid JSON | Return `Err(E-CFG-003: JSON parse error at <path>: <serde error>)`. No partial results for that file. |
| EC-003 | FM-008: Config file is valid JSON but has neither `mcpServers` nor `servers` key | Return empty server list for that file with warning `E-CFG-004: No recognized schema`. |
| EC-004 | DEC-008: Config file is valid JSON but `mcpServers` value is not an object | Return `Err(E-CFG-005: mcpServers is not a JSON object)` for that file. |
| EC-005 | Server entry has both `command` and `url` fields | Infer transport from schema context: `mcpServers` → prefer `command` (stdio); `servers` → use explicit `type`. Log warning. |
| EC-006 | Server entry has `disabled: true` (Cursor) | Set `enabled = false`. Server is included in registry but not auto-connected. |
| EC-007 | Unknown fields in server entry | Ignore unknown fields. Do not error. Forward-compatible parsing. |
| EC-008 | `env` contains values that are not strings | Return `Err(E-CFG-007: env values must be strings)` for that server entry. Other entries in the file are still parsed. |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-001 | `{"mcpServers": {"my-server": {"command": "npx", "args": ["-y", "my-mcp"], "env": {"KEY": "val"}}}}` | `[ServerEntry {name: "my-server", transport: Stdio, config: StdioConfig {command: "npx", args: ["-y", "my-mcp"], env: {"KEY": "val"}}, enabled: true}]` |
| TV-002 | `{"servers": {"my-server": {"type": "stdio", "command": "node", "args": ["server.js"]}}}` (VS Code schema) | `[ServerEntry {name: "my-server", transport: Stdio, config: StdioConfig {command: "node", args: ["server.js"], env: {}}, enabled: true}]` |
| TV-003 | `{"mcpServers": {"remote": {"url": "https://api.example.com/mcp", "headers": {"Authorization": "Bearer tok"}}}}` | `[ServerEntry {name: "remote", transport: Http, config: HttpConfig {url: "https://api.example.com/mcp", headers: {"Authorization": "Bearer tok"}}, enabled: true}]` |
| TV-004 | Cursor config with `"disabled": true` and `"alwaysAllow": ["tool_a"]` | `ServerEntry {enabled: false, always_allow: ["tool_a"]}` |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-005 | `{"mcpServers": {}}` — valid JSON, empty servers | Empty server list, no error |
| TV-006 | `{"unrelated_key": 42}` — no recognized schema | Empty server list + warning `E-CFG-004` |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-007 | `{invalid json` | `Err(E-CFG-003)` |
| TV-008 | `{"mcpServers": [1,2,3]}` — wrong type | `Err(E-CFG-005)` |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | Both schema variants produce identical `ServerEntry` structures for equivalent server definitions | Unit test: define same server in both schemas, assert `ServerEntry` equality (modulo source fields) |
| VP-002 | Unknown JSON fields do not cause parse errors | Fuzz test with random additional fields |
| VP-003 | `disabled: true` always maps to `enabled: false` | Unit test with truth table |

## Traceability

- **L2 Capability**: CAP-001 (Multi-Editor Config Discovery)
- **Domain Invariant**: DI-015 (Config files are read-only)
- **Edge Cases**: DEC-008, DEC-010
- **Failure Modes**: FM-007, FM-008
- **Priority**: P0
