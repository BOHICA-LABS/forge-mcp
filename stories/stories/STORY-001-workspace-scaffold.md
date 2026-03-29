---
document_type: story
story_id: STORY-001
epic_id: EPIC-00
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 3
depends_on: []
blocks: [STORY-002, STORY-003, STORY-004, STORY-007, STORY-008, STORY-013, STORY-023]
behavioral_contracts: []
verification_properties: []
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-001: Cargo Workspace Scaffold & CI Pipeline

## Narrative
- **As a** developer working on Forge MCP
- **I want to** have a correctly-structured Cargo workspace with all 10 crates, cross-compilation CI, and standard tooling configured
- **So that** all downstream stories can start from a compilable baseline with green CI

## Acceptance Criteria

### AC-001: Workspace compiles with all 10 crate stubs
All 10 crates (`forge-core`, `forge-discovery`, `forge-daemon`, `forge-tui`, `forge-traffic`, `forge-health`, `forge-security`, `forge-conformance`, `forge-config`, `forge-mcp`) compile with `cargo build` producing zero errors. Each crate contains at minimum a `lib.rs` or `main.rs` stub.
- **Test:** `test_workspace_compiles()`

### AC-002: Binary crate `forge-mcp` produces runnable binary
`cargo build --bin forge-mcp` succeeds. The resulting binary exits with code 0 when invoked with `--help`. (Traces to AD-001 single output binary.)

### AC-003: CI pipeline runs on 5 targets
The CI configuration (GitHub Actions workflow) builds and tests on: `linux-x64`, `linux-arm64`, `macos-x64`, `macos-arm64`, `windows-x64`. (Traces to NFR-008.)

### AC-004: Cross-compilation produces static binaries < 25MB
Release builds with LTO+strip produce a binary ≤ 25MB on linux-x64. (Traces to NFR-009.)

### AC-005: Dependency direction enforced structurally
The Cargo workspace `[dependencies]` graph matches the layering model (L0→L1→L2→L3→L4). forge-core has no internal workspace dependencies. (Traces to AD-001, dependency-graph.md layering model.)

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| Cargo.toml workspace | root | N/A (build config) |
| 10 crate stubs | crates/* | Pure (empty libs) |
| CI workflow | .github/workflows/ | N/A |

## UX Screens
- N/A — infrastructure story

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Missing crate in workspace | `cargo build` fails with clear "package not found" error |
| EC-002 | Windows cross-compile fails | CI reports failure with artifact for specific target |
| EC-003 | Binary exceeds 25MB | CI fails the size check step |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Build system | N/A | Not application code |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~800 |
| Cargo.toml / workspace config | ~400 |
| CI workflow YAML | ~600 |
| Tool outputs overhead | ~200 |
| **Total** | **~2,000** |
| Agent context window | 200K |
| **Budget usage** | **~1%** |

## Tasks

1. [ ] Create root `Cargo.toml` workspace manifest listing all 10 crates
2. [ ] Create `crates/forge-core/` with `Cargo.toml` + `src/lib.rs` stub
3. [ ] Repeat for all 10 crates (forge-discovery, forge-daemon, forge-tui, forge-traffic, forge-health, forge-security, forge-conformance, forge-config, forge-mcp)
4. [ ] Configure correct `[dependencies]` per layering model in each crate's `Cargo.toml`
5. [ ] Create `.github/workflows/ci.yml` with 5-target cross-compilation matrix
6. [ ] Configure release profile with LTO + strip
7. [ ] Add binary size check step to CI
8. [ ] Verify `cargo build` succeeds
9. [ ] Verify `forge-mcp --help` exits 0

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| N/A — first story | N/A | N/A | N/A |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Single binary output | AD-001 | Verify `forge-mcp` is the only `[[bin]]` target |
| Layered dependency graph | dependency-graph.md | Review `[dependencies]` in each crate's `Cargo.toml` |
| rmcp as exclusive transport | AD-002 | Include `rmcp` dep only in `forge-core` Cargo.toml |
| Tokio async runtime | AD-005 | Include `tokio` only where needed (forge-core, forge-daemon, forge-mcp) |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| rmcp | workspace (latest 0.x) | Official MCP SDK (AD-002, DI-004) | `forge-core` only |
| tokio | >= 1.38 | Async runtime required by rmcp (AD-005) | `forge-core`, `forge-daemon`, `forge-mcp` |
| serde | >= 1.0 | JSON serialization everywhere | `forge-core`, `forge-mcp` |
| serde_json | >= 1.0 | JSON parsing | `forge-core`, `forge-discovery`, `forge-mcp` |
| clap | >= 4.5 | CLI arg parsing | `forge-mcp` only |
| ratatui | >= 0.28 | TUI framework | `forge-tui` only |
| crossterm | >= 0.27 | Terminal backend (AD-007) | `forge-tui` only |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `Cargo.toml` | Workspace manifest | NO — this story creates it |
| `crates/forge-core/Cargo.toml` | L0 domain kernel crate | NO |
| `crates/forge-core/src/lib.rs` | Empty stub | NO |
| `crates/forge-discovery/Cargo.toml` | L1 config discovery | NO |
| `crates/forge-daemon/Cargo.toml` | L3 connection daemon | NO |
| `crates/forge-tui/Cargo.toml` | L3 TUI dashboard | NO |
| `crates/forge-traffic/Cargo.toml` | L1 traffic capture | NO |
| `crates/forge-health/Cargo.toml` | L1 health monitoring | NO |
| `crates/forge-security/Cargo.toml` | L2 security auditing | NO |
| `crates/forge-conformance/Cargo.toml` | L2 conformance testing | NO |
| `crates/forge-config/Cargo.toml` | L2 config diff | NO |
| `crates/forge-mcp/Cargo.toml` | L4 binary entry | NO |
| `crates/forge-mcp/src/main.rs` | Binary entry point stub | NO |
| `.github/workflows/ci.yml` | 5-target CI pipeline | NO |
