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

# BC-1.01.003 — Config Source Aggregation with Conflict Attribution

## Summary

Merges server entries from all discovered config sources into a unified server registry. Preserves source attribution for each entry and detects conflicts when the same server name appears across multiple config sources with different parameters.

## Preconditions

- PRE-001: BC-1.01.001 has completed config discovery.
- PRE-002: BC-1.01.002 has parsed all discovered configs into `ServerEntry` lists.
- PRE-003: Each `ServerEntry` has `source_editor` and `source_path` populated.

## Postconditions

- POST-001: The unified registry contains one `ServerEntry` per unique server name.
- POST-002: When a server name appears in exactly one source, it is included without conflict.
- POST-003: When a server name appears in multiple sources with identical parameters (transport, command/url, args, env), the entry is deduplicated. The first source in discovery order is recorded as primary; others are recorded as `also_found_in`.
- POST-004: When a server name appears in multiple sources with different parameters, a `Conflict` record is created containing: `server_name`, `sources` (list of `{editor, path, entry}`), and `resolution` (default: first-discovered-wins).
- POST-005: All conflicts are reported via event `E-CFG-006` with full attribution.
- POST-006: The conflict resolution strategy is deterministic: discovery order from BC-1.01.001 determines winner.

## Invariants

- **DI-015**: Config files are read-only. Aggregation MUST NOT write back to any config file.

## Edge Cases

| ID | Condition | Expected Behavior |
|----|-----------|-------------------|
| EC-001 | DEC-010: Server "my-server" in Claude Desktop uses stdio transport, same name in VS Code uses HTTP transport | Conflict created. First-discovered (Claude Desktop) wins in default registry. Conflict reported via E-CFG-006. |
| EC-002 | All config sources return zero servers | Unified registry is empty. No error. Warning emitted: `E-CFG-008: No MCP servers found in any config source`. |
| EC-003 | 50+ servers across all sources | Registry handles arbitrary count. No hardcoded limit. |
| EC-004 | Same server name in Cursor global and Cursor project scope | Project-scoped entry wins (more specific). This is NOT a conflict — it is intentional scoping. |
| EC-005 | Server name contains special characters (spaces, unicode) | Preserved as-is. Server names are opaque strings. |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-001 | Claude Desktop has `server-a` (stdio), Cursor has `server-b` (stdio) — no overlap | Registry: `[server-a, server-b]`, zero conflicts |
| TV-002 | Claude Desktop has `server-a` (stdio, command=npx), VS Code has `server-a` (stdio, command=npx) — identical | Registry: `[server-a]` with `also_found_in: [VSCode]`, zero conflicts |
| TV-003 | Three sources each with unique servers | Registry contains union of all servers, zero conflicts |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-004 | Claude Desktop: `db` → stdio `npx db-mcp`; Cursor: `db` → http `https://db.example.com` | Registry: `[db]` with Claude Desktop version. Conflict: `{name: "db", sources: [ClaudeDesktop(stdio), Cursor(http)]}`. Event E-CFG-006 emitted. |
| TV-005 | Cursor global: `lint` → npx lint-mcp; Cursor project: `lint` → npx lint-mcp-v2 | Registry: `[lint]` with project-scoped version (lint-mcp-v2). No conflict (project > global). |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-006 | Zero config sources provided (BC-1.01.001 returned empty) | Empty registry. Warning E-CFG-008. |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | Conflict detection is symmetric: servers A-in-X vs A-in-Y produces same conflict record regardless of discovery order | Unit test with reversed discovery order |
| VP-002 | Every conflict produces exactly one E-CFG-006 event | Integration test counting events |
| VP-003 | Project-scoped configs always override global-scoped configs from the same editor | Unit test with Cursor global + project overlap |

## Traceability

- **L2 Capability**: CAP-001 (Multi-Editor Config Discovery), CAP-021 (Structured Output)
- **Domain Invariant**: DI-015 (Config files are read-only)
- **Edge Cases**: DEC-010
- **Priority**: P0
