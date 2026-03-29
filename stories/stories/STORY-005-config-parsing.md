---
document_type: story
story_id: STORY-005
epic_id: EPIC-01
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-004]
blocks: [STORY-006]
behavioral_contracts: [BC-1.01.002]
verification_properties: [VP-004]
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-005: Dual-Schema Config Parsing (mcpServers vs servers)

## Narrative
- **As a** developer with MCP servers configured in Claude Desktop, Cursor, VS Code, or Windsurf
- **I want to** have Forge MCP parse both JSON config schemas into a unified server registry
- **So that** I can use a single tool regardless of which editor I use

> **SR-001 addressed here:** `StdioConfig` and `HttpConfig` struct fields are explicitly defined in ACs.
> **SR-004 addressed here:** EC-003 (neither `mcpServers` nor `servers`) explicitly states processing continues for other files.
> **SR-005 addressed here:** `DiscoveredConfig` properties are used as input, now fully defined in STORY-004.

## Acceptance Criteria

### AC-001 (traces to BC-1.01.002 postcondition POST-001)
For `"mcpServers"` schema: each key under `mcpServers` becomes a `ServerEntry` with `name` = key, transport inferred from `command` (→ Stdio) or `url` (→ Http). Transport config for Stdio: `StdioConfig { command: String, args: Vec<String>, env: HashMap<String,String> }`. Transport config for Http: `HttpConfig { url: String, headers: HashMap<String,String> }`.
- **Test:** `test_BC_1_01_002_mcp_servers_schema_stdio()`
- **Test:** `test_BC_1_01_002_mcp_servers_schema_http()`

### AC-002 (traces to BC-1.01.002 postcondition POST-002)
For `"servers"` schema (VS Code): transport determined by explicit `"type"` field (`"stdio"` → Stdio, `"sse"` → Http). Same `StdioConfig`/`HttpConfig` output structs.
- **Test:** `test_BC_1_01_002_servers_schema_vscode()`

### AC-003 (traces to BC-1.01.002 postcondition POST-003)
All `ServerEntry` records include: `name: String`, `transport: Transport` (Stdio | Http), `config: TransportConfig`, `source_editor: EditorKind`, `source_path: PathBuf`, `enabled: bool` (default true), `always_allow: Vec<String>` (default empty).
- **Test:** `test_BC_1_01_002_server_entry_all_fields()`

### AC-004 (traces to BC-1.01.002 postcondition POST-004)
The Cursor `disabled` field maps to `enabled = !disabled`. `disabled: true` → `enabled: false`. Server still appears in registry.
- **Test:** `test_BC_1_01_002_disabled_maps_to_enabled_false()`

### AC-005 (traces to BC-1.01.002 postcondition POST-005)
The Cursor `alwaysAllow` field maps to `always_allow: Vec<String>`.
- **Test:** `test_BC_1_01_002_always_allow_mapping()`

### AC-006 (traces to BC-1.01.002 postcondition POST-006)
`env` values are stored as-is (not expanded). Expansion deferred to connection time (BC-1.02.001).
- **Test:** `test_BC_1_01_002_env_not_expanded()`

### AC-007 (traces to BC-1.01.002 edge case EC-002)
Invalid JSON returns `Err(E-CFG-003: JSON parse error at <path>)`. No partial results for the file.
- **Test:** `test_BC_1_01_002_invalid_json_errors()`

### AC-008 (traces to BC-1.01.002 edge case EC-003 — SR-004)
Config file valid JSON but lacks both `mcpServers` and `servers` keys → returns empty server list for that file with warning `E-CFG-004`. **Processing continues for other discovered config files** (does not abort the overall discovery).
- **Test:** `test_BC_1_01_002_no_recognized_schema_continues()`

### AC-009 (traces to BC-1.01.002 edge case EC-007)
Unknown fields in server entry are ignored without error. Forward-compatible parsing.
- **Test:** `test_BC_1_01_002_unknown_fields_ignored()`

### AC-010 (traces to BC-1.01.002 edge case EC-008)
`env` values that are not strings return `Err(E-CFG-007)` for that entry. Other entries in the file are still parsed.
- **Test:** `test_BC_1_01_002_env_non_string_per_entry_error()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `parse_config(DiscoveredConfig)` | `forge-discovery/src/parser.rs` | Pure |
| `ServerEntry` struct | `forge-core/src/types.rs` | Pure |
| `StdioConfig`, `HttpConfig` | `forge-core/src/types.rs` | Pure |
| `Transport` enum | `forge-core/src/types.rs` | Pure |

## UX Screens
- N/A — backend story

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Both command and url fields present | mcpServers: prefer command; servers: use explicit type |
| EC-002 | Invalid JSON | Err(E-CFG-003), no partial results |
| EC-003 | No recognized schema | Empty list + E-CFG-004 warning, continue to other files (SR-004) |
| EC-004 | disabled=true | enabled=false, still in registry |
| EC-005 | Non-string env value | Err(E-CFG-007) for that entry, others parsed |
| EC-006 | Unknown fields | Ignored silently |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `parse_config()` | Pure | Input is DiscoveredConfig with content string; no I/O |
| `ServerEntry` / config types | Pure | Data structures only |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~1,400 |
| BC-1.01.002 (referenced) | ~1,000 |
| forge-core types (new) | ~400 |
| Test vectors from BC | ~500 |
| **Total** | **~3,300** |
| Agent context window | 200K |
| **Budget usage** | **~1.7%** |

## Tasks

1. [ ] Write failing tests for all 10 ACs
2. [ ] Define `ServerEntry`, `Transport`, `StdioConfig`, `HttpConfig` in forge-core (per SR-001)
3. [ ] Define `TransportConfig` enum wrapping StdioConfig / HttpConfig
4. [ ] Implement `parse_config(discovered: DiscoveredConfig) -> Result<Vec<ServerEntry>, ConfigError>`
5. [ ] Handle mcpServers schema (schema detection → stdio/http inference)
6. [ ] Handle servers schema (VS Code explicit type field)
7. [ ] Handle disabled → enabled, alwaysAllow → always_allow
8. [ ] Handle forward-compatible unknown field parsing (serde `deny_unknown_fields` off)
9. [ ] Add fuzz target for parse_config (VP-004)
10. [ ] Verify Red Gate
11. [ ] Implement to pass all tests
12. [ ] Refactor

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-004 | DiscoveredConfig has content: String, path: PathBuf, editor: EditorKind | Use these exact field names | serde needs to handle file content, not path |
| STORY-001 | forge-core is L0 domain kernel | Put ServerEntry in forge-core/src/types.rs | |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| ServerEntry defined in forge-core (domain kernel) | dependency-graph.md ADR | `use forge_core::types::ServerEntry` |
| parse_config is pure (no I/O) | purity-boundary-map.md | Function signature: no `&Path`, no `fs::read` |
| DI-015: no file writes | BC invariant | Verified by code review |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| serde | >= 1.0 | Struct deserialization | `#[derive(Deserialize)]` |
| serde_json | >= 1.0 | JSON parsing | `serde_json::from_str` |
| thiserror | >= 1.0 | Config error types | `#[derive(thiserror::Error)]` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-core/src/types.rs` | ServerEntry, Transport, StdioConfig, HttpConfig | YES (stub) — this story fills it |
| `crates/forge-discovery/src/parser.rs` | parse_config() pure function | NO — this story creates it |
| `crates/forge-discovery/src/error.rs` | ConfigError, E-CFG-NNN codes | NO — this story creates it |
| `crates/forge-discovery/tests/parser_tests.rs` | Unit tests with BC test vectors | NO — this story creates it |
| `fuzz/fuzz_targets/parse_config.rs` | Fuzz target for VP-004 | NO — this story creates it |
