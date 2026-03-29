---
document_type: story
story_id: STORY-061
epic_id: EPIC-10
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-016, STORY-006]
blocks: []
behavioral_contracts: [BC-10.23.001]
verification_properties: []
priority: P2
assumption_validations: []
risk_mitigations: []
---

# STORY-061: Server Tool Schema Diff

## Narrative
- **As an** MCP Server Author
- **I want to** compare tool schemas between two server instances
- **So that** I can identify breaking API changes before deploying

## Acceptance Criteria

### AC-001 (traces to BC-10.23.001 postcondition — tool schema diff)
`diff_tool_schemas(tools_a, tools_b) -> ToolSchemaDiff` produces: `added: Vec<Tool>`, `removed: Vec<Tool>`, `changed: Vec<(Tool, Tool)>` (old, new pairs), `unchanged: Vec<Tool>`.
- **Test:** `test_BC_10_23_001_tool_schema_diff()`

### AC-002 (traces to BC-10.23.001 — field-level diff)
For changed tools, the diff includes field-level changes: `name`, `description`, `inputSchema` (JSON diff showing added/removed/changed properties).
- **Test:** `test_BC_10_23_001_field_level_diff()`

### AC-003 (traces to BC-10.23.001 — CLI command)
`forge-mcp diff <server-a> <server-b>` runs tool schema diff and outputs JSON or text. Output includes summary counts and field-level diff.
- **Test:** `test_BC_10_23_001_cli_diff_command()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `diff_tool_schemas()` | `forge-config/src/schema_diff.rs` | Pure |
| CLI diff command | `forge-mcp/src/commands/diff.rs` | Effectful |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Same tools on both servers | All unchanged |
| EC-002 | One server has no tools | All as removed/added |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| diff_tool_schemas() | Pure | Set operations on Vec<Tool> |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~600 |
| BC-10.23.001 | ~400 |
| **Total** | **~1,000** |
| Agent context window | 200K |
| **Budget usage** | **~0.5%** |

## Tasks

1. [ ] Write failing tests for all 3 ACs
2. [ ] Implement `diff_tool_schemas()` pure function
3. [ ] Implement JSON schema diff (field-level)
4. [ ] Implement CLI diff command
5. [ ] Verify Red Gate
6. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-059 | Diff pattern established in forge-config | Same crate, extend | JSON Schema diff needs deep comparison |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| forge-config fully pure | purity-boundary-map.md | |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| json-patch | >= 0.3 | JSON diff | `json_patch::diff` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-config/src/schema_diff.rs` | diff_tool_schemas() | NO — this story creates it |
| `crates/forge-mcp/src/commands/diff.rs` | CLI diff command | NO |
