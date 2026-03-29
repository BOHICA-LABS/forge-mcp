---
document_type: story
story_id: STORY-065
epic_id: SR
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 3
depends_on: [STORY-005, STORY-047]
blocks: []
behavioral_contracts: [BC-1.01.002, BC-7.16.001]
verification_properties: []
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-065: SR Refinements — Config Struct Clarity & Security Heuristics Reference

## Narrative
- **As a** developer implementing Forge MCP
- **I want to** have clear documentation of config struct fields and canonical security heuristic examples
- **So that** I can implement both components correctly without ambiguity

> This story consolidates spec-review follow-up items that weren't addressable inline in earlier stories.

## Acceptance Criteria

### AC-001 (addresses SR-001 — transport config field formalization)
The `StdioConfig` and `HttpConfig` structs (defined in STORY-005) have inline doc comments confirming field types:
- `StdioConfig.command: String` — REQUIRED. The executable name or path.
- `StdioConfig.args: Vec<String>` — OPTIONAL (default empty). Arguments.
- `StdioConfig.env: HashMap<String,String>` — OPTIONAL (default empty). Env overrides.
- `HttpConfig.url: String` — REQUIRED. The endpoint URL.
- `HttpConfig.headers: HashMap<String,String>` — OPTIONAL (default empty). Auth headers.
These doc comments are present in `crates/forge-core/src/types.rs`.
- **Test:** Rust doc test: `cargo test --doc`

### AC-002 (addresses SR-004 — missing schema behavior)
The `parse_config()` function in STORY-005 handles EC-003 explicitly with this behavior documented in a comment: "Return empty list + E-CFG-004 warning. Processing continues for other discovered config files (does not abort)." Integration test verifies multi-file behavior.
- **Test:** `test_sr_004_missing_schema_continues_multi_file()`

### AC-003 (addresses SR-005 — DiscoveredConfig properties)
`DiscoveredConfig` struct (STORY-004) has doc comments for all fields:
- `editor: EditorKind` — Which editor this config belongs to.
- `path: PathBuf` — Absolute path to the config file.
- `scope: ConfigScope` — Global or project-scoped.
- `exists: bool` — Whether the file exists on disk.
- `access_error: Option<String>` — Set if exists=true but unreadable.
These are present and verified by doc tests.
- **Test:** `cargo test --doc`

### AC-004 (addresses SR-002 — security heuristics reference)
`tests/corpus/security_heuristics_reference.md` documents canonical examples of each security pattern:
- **SSRF examples:** `169.254.169.254`, `10.0.0.1`, `metadata.google.internal`
- **Exec examples:** `sh -c "cmd"`, `bash -c`, `subprocess.Popen`, `os.system`
- **Filesystem examples:** `read_file /etc/passwd`, `write_file /home/user/.ssh/authorized_keys`
- **Escalation examples:** `sudo apt-get`, `chmod 777 /`, `setuid 0`
This document serves as the initial rule corpus for achieving NFR-005 > 80% precision.
- **Test:** Document exists and is linked from forge-security README

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| Doc comments | forge-core/src/types.rs, forge-discovery/src/types.rs | N/A |
| Heuristics reference | tests/corpus/ | N/A (documentation) |

## UX Screens
- N/A — documentation story

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| N/A | N/A | Documentation story |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~600 |
| Existing type files | ~400 |
| **Total** | **~1,000** |
| Agent context window | 200K |
| **Budget usage** | **~0.5%** |

## Tasks

1. [ ] Add comprehensive doc comments to StdioConfig, HttpConfig (SR-001)
2. [ ] Add doc comments to DiscoveredConfig (SR-005)
3. [ ] Add code comment in parse_config() for EC-003 multi-file behavior (SR-004)
4. [ ] Write multi-file integration test for SR-004
5. [ ] Create `tests/corpus/security_heuristics_reference.md` (SR-002)
6. [ ] Run `cargo test --doc` to verify doc tests pass
7. [ ] Verify Red Gate
8. [ ] Implement to pass all tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-005 | StdioConfig/HttpConfig defined | Add doc comments here | Rust doc tests need `///` not `//` |
| STORY-047 | Security corpus started | Extend with SR-002 examples | |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| SR-001, SR-002, SR-004, SR-005 addressed | SR-INDEX.md | All 4 SR recommendations closed |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| (no new deps — doc comments only) | | | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-core/src/types.rs` | Doc comments on StdioConfig, HttpConfig | YES — add comments |
| `crates/forge-discovery/src/types.rs` | Doc comments on DiscoveredConfig | YES — add comments |
| `crates/forge-discovery/src/parser.rs` | EC-003 behavior comment | YES (from STORY-005) |
| `tests/corpus/security_heuristics_reference.md` | Canonical heuristics (SR-002) | NO — this story creates it |
