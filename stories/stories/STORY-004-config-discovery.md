---
document_type: story
story_id: STORY-004
epic_id: EPIC-01
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-001]
blocks: [STORY-005, STORY-007, STORY-008]
behavioral_contracts: [BC-1.01.001]
verification_properties: [VP-004]
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-004: Config File Discovery & Path Resolution

## Narrative
- **As an** AI Platform Engineer or DevEx Engineer
- **I want to** have Forge MCP discover all MCP configuration files from supported editors automatically
- **So that** I don't need to manually specify config paths for Claude Desktop, Cursor, VS Code, or Windsurf

> **SR-005 addressed here:** The `DiscoveredConfig` struct properties are fully defined in this story's AC and file structure requirements per SR-005 recommendation.

## Acceptance Criteria

### AC-001 (traces to BC-1.01.001 postcondition POST-001)
`discover_configs(os, project_root)` returns a `Vec<DiscoveredConfig>` where each entry contains: `editor: EditorKind` (ClaudeDesktop | Cursor | VSCode | Windsurf), `path: PathBuf` (absolute), `scope: ConfigScope` (Global | Project), `exists: bool`, `access_error: Option<String>`.
- **Test:** `test_BC_1_01_001_discovered_config_struct_shape()`

### AC-002 (traces to BC-1.01.001 postcondition POST-002)
All known editor config paths for the detected OS are probed, even if they do not exist on disk. On macOS: probes all 6 locations (Claude Desktop global, Cursor global, Cursor project, VS Code global, VS Code workspace, Windsurf global).
- **Test:** `test_BC_1_01_001_all_paths_probed_macos()`

### AC-003 (traces to BC-1.01.001 postcondition POST-003)
Project-scoped config paths (`.cursor/mcp.json`, `.vscode/mcp.json`) are resolved relative to the provided `project_root`. When `project_root` is `None`, project-scoped entries are omitted from the result.
- **Test:** `test_BC_1_01_001_project_scoped_resolution()`

### AC-004 (traces to BC-1.01.001 postcondition POST-004)
The returned list preserves deterministic discovery order: Claude Desktop → Cursor (global) → Cursor (project) → VS Code (global) → VS Code (workspace) → Windsurf. This order is identical across repeated invocations.
- **Test:** `test_BC_1_01_001_deterministic_order()`

### AC-005 (traces to BC-1.01.001 invariant DI-015)
`discover_configs` does not create, modify, or delete any file. The function is read-only. Verified by running in a read-only temporary directory.
- **Test:** `test_BC_1_01_001_no_filesystem_writes()`

### AC-006 (traces to BC-1.01.001 edge case EC-001)
When no config files exist on disk, the function returns a list with all entries having `exists: false`. No error is raised. Zero `exists: true` entries.
- **Test:** `test_BC_1_01_001_no_configs_exist()`

### AC-007 (traces to BC-1.01.001 edge case EC-004)
When a config file exists but has permission denied, the entry has `exists: true` and `access_error: Some("Permission denied (os error 13)")`. Discovery continues for remaining files.
- **Test:** `test_BC_1_01_001_permission_denied_continues()`

### AC-008 (traces to BC-1.01.001 edge case EC-005)
When `$HOME` / `%APPDATA%` is unset, returns `Err(E-CFG-001)` and discovery aborts.
- **Test:** `test_BC_1_01_001_missing_home_dir_errors()`

### AC-009 (traces to BC-1.01.001 edge case EC-006)
Symlinked config files are followed. The result reports the original (symlink) path, not the resolved target.
- **Test:** `test_BC_1_01_001_symlink_followed()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `discover_configs()` public API | `forge-discovery/src/discovery.rs` | Effectful (FS reads) |
| `os_paths()` path table | `forge-discovery/src/paths.rs` | Pure |
| `DiscoveredConfig` struct | `forge-discovery/src/types.rs` | Pure |
| `EditorKind` enum | `forge-discovery/src/types.rs` | Pure |

## UX Screens
- N/A — backend story; feeds SCR-002 (Server Sidebar) indirectly

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | No configs exist | Empty list (all exists=false), no error |
| EC-002 | 0-byte config file | exists=true, deferred to BC-1.01.002 |
| EC-003 | Non-UTF-8 path | access_error with E-CFG-002, continue |
| EC-004 | Permission denied | exists=true, access_error set, continue |
| EC-005 | $HOME unset | Err(E-CFG-001), abort |
| EC-006 | Symlinked file | Follow symlink, report original path |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `os_paths()` | Pure | Returns hardcoded OS path table, no I/O |
| `discover_configs()` | Effectful | Reads filesystem (exists checks) |
| `DiscoveredConfig` | Pure | Data type only |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~1,200 |
| BC-1.01.001 (referenced) | ~800 |
| Referenced code files | ~400 |
| Test fixtures | ~300 |
| **Total** | **~2,700** |
| Agent context window | 200K |
| **Budget usage** | **~1.4%** |

## Tasks

1. [ ] Write failing tests for all 9 ACs (test-writer)
2. [ ] Define `EditorKind` enum with 4 variants
3. [ ] Define `ConfigScope` enum (Global | Project)
4. [ ] Define `DiscoveredConfig` struct with all fields (per SR-005)
5. [ ] Implement `os_paths()` pure function with OS-specific path table
6. [ ] Implement `discover_configs(os, project_root)` effectful function
7. [ ] Handle permission denied, symlinks, non-UTF-8 paths
8. [ ] Verify DI-015 invariant (no writes) via read-only FS test
9. [ ] Add proptest for deterministic ordering
10. [ ] Verify Red Gate (all tests fail before implementation)
11. [ ] Implement to pass tests
12. [ ] Refactor for clarity

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-001 | forge-discovery is L1 (depends on forge-core only) | Follow layer boundaries | `dirs` crate needed for cross-platform home dir |
| STORY-002 | Test infra uses fixture files, not real editor installs | Use tempdir for FS tests | |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| forge-discovery depends only on forge-core | dependency-graph.md L1 rule | No imports from L2+ crates |
| Config read-only (DI-015) | purity-boundary-map.md | No `fs::write` in discovery module |
| Pure/effectful separation | purity-boundary-map.md | `os_paths()` pure, `discover_configs()` effectful |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| dirs | >= 5.0 | Cross-platform home/app dirs | `dirs::home_dir()` |
| serde | >= 1.0 | Serialize DiscoveredConfig | `#[derive(Serialize, Deserialize)]` |
| tempfile | >= 3.0 (dev) | Test fixture FS | `tempfile::tempdir()` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-discovery/src/lib.rs` | Crate root, re-exports | YES (stub) |
| `crates/forge-discovery/src/types.rs` | DiscoveredConfig, EditorKind, ConfigScope | NO — this story creates it |
| `crates/forge-discovery/src/paths.rs` | OS path tables (pure) | NO — this story creates it |
| `crates/forge-discovery/src/discovery.rs` | discover_configs() (effectful) | NO — this story creates it |
| `crates/forge-discovery/tests/discovery_tests.rs` | Integration tests | NO — this story creates it |
