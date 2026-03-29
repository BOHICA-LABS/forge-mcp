---
document_type: story
story_id: STORY-031
epic_id: EPIC-04
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 3
depends_on: [STORY-029]
blocks: []
behavioral_contracts: [BC-4.10.002]
verification_properties: []
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-031: Full-Text Payload Search

## Narrative
- **As an** AI Platform Engineer
- **I want to** search across all captured message payloads for a term
- **So that** I can quickly find messages containing specific data

## Acceptance Criteria

### AC-001 (traces to BC-4.10.002 postcondition — substring search)
`search_messages(messages, "SSRF")` returns all messages whose serialized JSON payload contains the literal string "SSRF" (case-insensitive by default).
- **Test:** `test_BC_4_10_002_substring_search()`

### AC-002 (traces to BC-4.10.002 — case-sensitive option)
`SearchSpec { pattern: "ssrf", case_sensitive: true }` performs case-sensitive match.
- **Test:** `test_BC_4_10_002_case_sensitive_search()`

### AC-003 (traces to BC-4.10.002 — CLI grep subcommand)
`forge-mcp grep <server> <pattern>` invokes `search_messages` and outputs matching messages as JSON, each with `index`, `method`, `direction`, `timestamp_ms`, `match_preview` (first 200 chars of matched payload).
- **Test:** `test_BC_4_10_002_cli_grep_output()`

### AC-004 (traces to BC-4.10.002 — no matches)
When no messages match, `search_messages` returns empty Vec. CLI outputs `{ "matches": [] }` and exits 0.
- **Test:** `test_BC_4_10_002_no_matches_empty_result()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `search_messages()` | `forge-traffic/src/search.rs` | Pure |
| CLI grep command | `forge-mcp/src/commands/grep.rs` | Effectful |

## UX Screens
- SCR-004 (Traffic Inspector) — `/` also uses search for inline filtering

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Pattern is empty string | Returns all messages |
| EC-002 | Pattern with regex metacharacters | Treated as literal string, not regex |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| search_messages() | Pure | String matching on serialized payloads |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~600 |
| BC-4.10.002 | ~300 |
| **Total** | **~900** |
| Agent context window | 200K |
| **Budget usage** | **~0.5%** |

## Tasks

1. [ ] Write failing tests for all 4 ACs
2. [ ] Implement `search_messages()` pure function
3. [ ] Implement CLI `grep` subcommand
4. [ ] Verify Red Gate
5. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-030 | FilterSpec pattern established | SearchSpec is similar | Serialization to search on should be consistent |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| search_messages() is pure | purity-boundary-map.md | No I/O |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| (no new deps — use std string methods) | | | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-traffic/src/search.rs` | search_messages() | NO — this story creates it |
| `crates/forge-mcp/src/commands/grep.rs` | CLI grep command | NO |
