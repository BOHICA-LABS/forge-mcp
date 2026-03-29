---
document_type: story
story_id: STORY-025
epic_id: EPIC-05
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 3
depends_on: [STORY-024]
blocks: []
behavioral_contracts: [BC-5.12.002]
verification_properties: []
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-025: Pipeable Output & Shell Composition

## Narrative
- **As a** DevOps engineer
- **I want to** pipe Forge MCP output to `jq`, `grep`, and other Unix tools
- **So that** I can compose shell scripts that extract exactly the data I need

## Acceptance Criteria

### AC-001 (traces to BC-5.12.002 postcondition — pipe-safe stdout)
When stdout is piped (not a TTY), all color codes and terminal control sequences are suppressed automatically. Raw JSON flows to the pipe consumer.
- **Test:** `test_BC_5_12_002_no_ansi_in_pipe()`

### AC-002 (traces to BC-5.12.002 postcondition — jq compatibility)
`forge-mcp list | jq '.[].name'` works correctly. Output JSON is valid and parseable by standard JSON tools.
- **Test:** `test_BC_5_12_002_jq_compatible_output()`

### AC-003 (traces to BC-5.12.002 — null-separated output for xargs)
`forge-mcp list --null-separated` outputs each JSON record followed by `\0` instead of newline, enabling `xargs -0` composition.
- **Test:** `test_BC_5_12_002_null_separated_output()`

### AC-004 (traces to BC-5.12.002 — no stdin buffering)
Stdout is flushed after each JSON record. No buffering that would delay piped consumers.
- **Test:** `test_BC_5_12_002_stdout_flushed()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| TTY detection | `forge-mcp/src/output.rs` | Effectful |
| Pipe mode formatting | `forge-mcp/src/output.rs` | Pure |

## UX Screens
- N/A

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | stdout redirected to file | Same as pipe mode (no color) |
| EC-002 | Force-color in pipe | `--color=always` flag overrides TTY detection |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Pipe mode formatting | Pure | String transformation |
| TTY detection | Effectful | OS syscall |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~600 |
| BC-5.12.002 | ~300 |
| **Total** | **~900** |
| Agent context window | 200K |
| **Budget usage** | **~0.5%** |

## Tasks

1. [ ] Write failing tests for all 4 ACs
2. [ ] Implement TTY detection (isatty)
3. [ ] Suppress ANSI codes when piped
4. [ ] Implement --null-separated flag
5. [ ] Ensure stdout flush after each record
6. [ ] Verify Red Gate
7. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-024 | Output as pure formatters | Add TTY detection on top | Windows has no isatty; use is_terminal crate |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Pipeable output (BC-5.12.002) | CLI spec | isatty check on stdout |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| is-terminal | >= 0.4 | Cross-platform isatty | `stdout.is_terminal()` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-mcp/src/output.rs` | TTY detection + pipe mode | YES (from STORY-024) |
