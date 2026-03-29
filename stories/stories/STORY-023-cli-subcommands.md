---
document_type: story
story_id: STORY-023
epic_id: EPIC-05
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-013]
blocks: [STORY-024, STORY-025, STORY-026]
behavioral_contracts: [BC-5.11.001, BC-5.11.003]
verification_properties: []
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-023: CLI Subcommand Dispatch & Exit Code Semantics

## Narrative
- **As an** AI agent or script
- **I want to** invoke Forge MCP with well-defined subcommands and predictable exit codes
- **So that** I can build automation pipelines that react correctly to success and failure

## Acceptance Criteria

### AC-001 (traces to BC-5.11.001 postcondition — subcommand dispatch)
`forge-mcp` dispatches correctly to: `list [server]`, `call <server> <tool> [args]`, `info <server>`, `grep <pattern>`, `test <server>`, `audit <server>`, `tui`, `daemon start|stop|restart|sessions`. Unknown subcommands print help and exit 1.
- **Test:** `test_BC_5_11_001_subcommand_dispatch()`

### AC-002 (traces to BC-5.11.003 postcondition — exit code 0)
Successful operations exit with code 0.
- **Test:** `test_BC_5_11_003_success_exits_0()`

### AC-003 (traces to BC-5.11.003 postcondition — exit code 1)
Test failures (conformance test failures) exit with code 1.
- **Test:** `test_BC_5_11_003_test_failure_exits_1()`

### AC-004 (traces to BC-5.11.003 postcondition — exit code 2)
Connection errors exit with code 2.
- **Test:** `test_BC_5_11_003_connection_error_exits_2()`

### AC-005 (traces to BC-5.11.003 postcondition — exit code 3)
Config errors (no config files found, parse errors) exit with code 3.
- **Test:** `test_BC_5_11_003_config_error_exits_3()`

### AC-006 (traces to BC-5.11.003 postcondition — exit code 4)
Security findings detected during `audit` exit with code 4.
- **Test:** `test_BC_5_11_003_security_finding_exits_4()`

### AC-007 (traces to BC-5.11.001 — NFR-001 cold start)
First invocation of `forge-mcp --help` completes in < 50ms wall time. (NFR-001 cold start.)
- **Test:** Benchmark with hyperfine

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `main.rs` clap dispatch | `forge-mcp/src/main.rs` | Effectful |
| Exit code mapping | `forge-mcp/src/exit_codes.rs` | Pure (DI-014) |
| Subcommand structs | `forge-mcp/src/commands/` | Mixed |

## UX Screens
- N/A — CLI story

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Unknown flag | Clap error, exit 1, help shown |
| EC-002 | Multiple errors (config + connection) | Most severe exit code wins |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Exit code mapping | Pure (DI-014) | Error → code lookup table |
| clap dispatch | Effectful | Process I/O |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~900 |
| BC-5.11.001, BC-5.11.003 | ~600 |
| clap patterns | ~400 |
| **Total** | **~1,900** |
| Agent context window | 200K |
| **Budget usage** | **~1%** |

## Tasks

1. [ ] Write failing tests for all 7 ACs
2. [ ] Define all subcommand structs with clap derives
3. [ ] Implement exit code mapping (pure, DI-014)
4. [ ] Implement main dispatch loop
5. [ ] Benchmark cold start (NFR-001)
6. [ ] Verify Red Gate
7. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-010 | Daemon lazy start from CLI | CLI checks for daemon before dispatch | clap must be fast: avoid loading all subsystems at parse time |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Exit codes per DI-014 | purity-boundary-map.md | Pure exit code mapping function |
| Single binary (AD-001) | ARCH-INDEX.md | All subcommands in forge-mcp binary |
| Lazy-load for NFR-001 | nfr-catalog.md | Don't initialize TUI/daemon on CLI commands |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| clap | >= 4.5 | CLI arg parsing with derive | `#[derive(Parser, Subcommand)]` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-mcp/src/main.rs` | clap dispatch | YES (stub from STORY-001) |
| `crates/forge-mcp/src/exit_codes.rs` | Exit code mapping | NO — this story creates it |
| `crates/forge-mcp/src/commands/mod.rs` | Subcommand module root | NO |
| `crates/forge-mcp/src/commands/list.rs` | list subcommand | NO |
| `crates/forge-mcp/src/commands/call.rs` | call subcommand | NO |
| `crates/forge-mcp/src/commands/info.rs` | info subcommand | NO |
