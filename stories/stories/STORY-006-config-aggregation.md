---
document_type: story
story_id: STORY-006
epic_id: EPIC-01
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 3
depends_on: [STORY-005]
blocks: [STORY-037, STORY-044, STORY-059]
behavioral_contracts: [BC-1.01.003]
verification_properties: []
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-006: Config Source Aggregation & Conflict Attribution

## Narrative
- **As an** AI Platform Engineer
- **I want to** have all discovered server configs aggregated into a single unified registry with conflict detection
- **So that** I can see all my MCP servers in one place and know when the same server is defined differently across editors

## Acceptance Criteria

### AC-001 (traces to BC-1.01.003 postcondition — aggregation result)
`aggregate_configs(Vec<Vec<ServerEntry>>) -> ServerRegistry` merges entries from all sources into a `ServerRegistry` where each server name maps to its winning `ServerEntry` plus a list of `ConflictRecord`s (if any).
- **Test:** `test_BC_1_01_003_aggregate_basic()`

### AC-002 (traces to BC-1.01.003 — conflict detection)
When the same server name appears in multiple config files (across editors), a `ConflictRecord` is created containing: `server_name: String`, `sources: Vec<(EditorKind, PathBuf, ServerEntry)>`. The winning entry is the one from the highest-priority source (Claude Desktop → Cursor → VS Code → Windsurf per discovery order).
- **Test:** `test_BC_1_01_003_conflict_detection()`

### AC-003 (traces to BC-1.01.003 — warning emission)
For each conflict detected, warning `E-CFG-006` is emitted: "Config conflict: server "<name>" defined differently in <source_a> and <source_b>". Warning goes to stderr and is included in the `ServerRegistry.conflicts` list.
- **Test:** `test_BC_1_01_003_conflict_warning_emitted()`

### AC-004 (traces to BC-1.01.003 — empty input)
`aggregate_configs([])` returns an empty `ServerRegistry` with zero entries and zero conflicts. No error.
- **Test:** `test_BC_1_01_003_empty_input()`

### AC-005 (traces to BC-1.01.003 — disabled servers included)
Servers with `enabled: false` are included in the registry (not filtered out). Consumers decide whether to auto-connect.
- **Test:** `test_BC_1_01_003_disabled_servers_included()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `aggregate_configs()` | `forge-discovery/src/aggregator.rs` | Pure |
| `ServerRegistry` | `forge-core/src/types.rs` | Pure |
| `ConflictRecord` | `forge-core/src/types.rs` | Pure |

## UX Screens
- SCR-002 (Server Sidebar) — displays the ServerRegistry
- SCR-009 (Config Drift View) — uses ConflictRecord data

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Same server, identical definition in two files | No conflict recorded (definitions are equal) |
| EC-002 | Server with disabled=true conflicts with enabled=true | Conflict recorded; winner uses priority order |
| EC-003 | Three-way conflict | Single ConflictRecord with 3 sources; winner from highest priority |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `aggregate_configs()` | Pure | Operates on Vec<ServerEntry> in memory |
| `ServerRegistry` | Pure | Data structure |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~800 |
| BC-1.01.003 (referenced) | ~600 |
| ServerRegistry type design | ~300 |
| **Total** | **~1,700** |
| Agent context window | 200K |
| **Budget usage** | **~0.9%** |

## Tasks

1. [ ] Write failing tests for all 5 ACs
2. [ ] Define `ServerRegistry` struct in forge-core
3. [ ] Define `ConflictRecord` struct in forge-core
4. [ ] Implement `aggregate_configs()` pure function
5. [ ] Implement conflict detection (same name, different configs)
6. [ ] Implement priority-based winner selection
7. [ ] Verify Red Gate
8. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-005 | ServerEntry fully defined | Use existing types | Conflict = different URL or command, not same |
| STORY-004 | Discovery order is deterministic | Use discovery order for conflict priority | |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| forge-discovery L1 (depends only on forge-core) | dependency-graph.md | No imports from L2+ |
| Aggregation is pure | purity-boundary-map.md | No I/O in aggregate_configs() |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| indexmap | >= 2.0 | Order-preserving HashMap for ServerRegistry | `IndexMap<String, ServerEntry>` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-core/src/types.rs` | ServerRegistry, ConflictRecord added | YES (from STORY-005) |
| `crates/forge-discovery/src/aggregator.rs` | aggregate_configs() | NO — this story creates it |
| `crates/forge-discovery/tests/aggregator_tests.rs` | Unit tests | NO |
