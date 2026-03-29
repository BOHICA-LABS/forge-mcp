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

# BC-1.01.001 — Config File Discovery and Path Resolution

## Summary

Discovers MCP server configuration files from all supported editors (Claude Desktop, Cursor, VS Code, Windsurf) using OS-specific path resolution. Returns an ordered list of discovered config file paths with their source editor attribution.

## Preconditions

- PRE-001: The host operating system is one of: macOS, Windows, or Linux.
- PRE-002: The filesystem is accessible (no OS-level sandbox blocking home directory reads).
- PRE-003: No prior config discovery is in progress (single-threaded discovery).

## Postconditions

- POST-001: Returns a list of `DiscoveredConfig` structs, each containing: `editor` (enum), `path` (absolute), `scope` (global | project), `exists` (bool).
- POST-002: All known editor config paths for the detected OS are probed, even if some do not exist.
- POST-003: Project-scoped config paths (Cursor `.cursor/mcp.json`, VS Code `.vscode/mcp.json`) are resolved relative to the provided project root, or omitted if no project root is supplied.
- POST-004: The returned list preserves a deterministic discovery order: Claude Desktop → Cursor (global) → Cursor (project) → VS Code (global) → VS Code (workspace) → Windsurf.

## Invariants

- **DI-015**: Config files are read-only. Discovery MUST NOT create, modify, or delete any config file.

## Edge Cases

| ID | Condition | Expected Behavior |
|----|-----------|-------------------|
| EC-001 | DEC-007: No config files exist on disk for any editor | Return empty list with zero `exists=true` entries. No error raised. |
| EC-002 | DEC-008: Config file exists but is 0 bytes | Include in results with `exists=true`. Parsing failure is deferred to BC-1.01.002. |
| EC-003 | DEC-009: Config file is replaced/modified while discovery is in progress | Discovery reads file metadata (existence check) at a single point in time. No consistency guarantee across files. Subsequent parse (BC-1.01.002) reads the file content and may see the new version. |
| EC-004 | FM-006: Config file exists but is not readable (permission denied) | Include in results with `exists=true` and an `access_error` field describing the permission failure. Do not abort discovery of remaining files. |
| EC-005 | FM-009: Home directory environment variable is unset (`$HOME` / `%APPDATA%`) | Return error `E-CFG-001: Cannot resolve home directory`. Discovery aborts. |
| EC-006 | Symlinked config file | Follow symlinks. Report the original (symlink) path, not the resolved target. |
| EC-007 | Config path contains non-UTF-8 characters | Return error `E-CFG-002: Non-UTF-8 path encountered` for that specific path. Continue discovery for remaining paths. |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-001 | macOS with Claude Desktop config at `~/Library/Application Support/Claude/claude_desktop_config.json` | List contains entry: `{editor: ClaudeDesktop, path: "/Users/testuser/Library/Application Support/Claude/claude_desktop_config.json", scope: Global, exists: true}` |
| TV-002 | macOS with all 6 config locations populated | List contains exactly 6 entries, all with `exists: true`, in deterministic order |
| TV-003 | Linux with Cursor global config at `~/.cursor/mcp.json` | List contains entry: `{editor: Cursor, path: "/home/testuser/.cursor/mcp.json", scope: Global, exists: true}` |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-004 | macOS with no config files present | List contains 6 entries (4 global + 0 project since no project root), all with `exists: false` |
| TV-005 | macOS with Claude Desktop config present but permission denied | Entry has `exists: true`, `access_error: Some("Permission denied")` |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-006 | `$HOME` is unset on Linux | `Err(E-CFG-001)` — discovery aborts |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | Discovery never writes to the filesystem | Static analysis + integration test with read-only filesystem mock |
| VP-002 | All OS-specific paths are probed for the detected OS | Unit test per OS with full path table assertion |
| VP-003 | Discovery order is deterministic across invocations | Property test: run discovery N times, assert identical ordering |

## Traceability

- **L2 Capability**: CAP-001 (Multi-Editor Config Discovery)
- **Domain Invariant**: DI-015 (Config files are read-only)
- **Edge Cases**: DEC-007, DEC-008, DEC-009
- **Failure Modes**: FM-006, FM-007, FM-008, FM-009
- **Priority**: P0
