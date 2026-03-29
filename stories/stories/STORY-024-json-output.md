---
document_type: story
story_id: STORY-024
epic_id: EPIC-05
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-023]
blocks: [STORY-025, STORY-064]
behavioral_contracts: [BC-5.11.002, BC-5.12.001]
verification_properties: []
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-024: Structured JSON Output & Agent-Optimized Tokens

## Narrative
- **As an** AI agent consuming Forge MCP output
- **I want to** receive structured JSON on stdout with minimal token overhead
- **So that** I can parse results programmatically and stay within context budget

## Acceptance Criteria

### AC-001 (traces to BC-5.11.002 postcondition — JSON on stdout)
All command results are emitted as JSON on stdout. Diagnostics (warnings, errors) go to stderr. No mixing of JSON and plain text on stdout.
- **Test:** `test_BC_5_11_002_json_stdout_diagnostics_stderr()`

### AC-002 (traces to BC-5.11.002 postcondition — output schemas)
`list` output has `{ "servers": [...] }`. `call` output has `{ "result": { "content": [...], "isError": bool } }`. `info` output has `{ "server": {...}, "capabilities": {...} }`. `grep` output has `{ "matches": [...] }`.
- **Test:** `test_BC_5_11_002_output_schemas()`

### AC-003 (traces to BC-5.12.001 postcondition — token count)
`forge-mcp list <server>` + `forge-mcp call <server> <tool>` combined output is ≤ 500 tokens (GPT-4 tokenizer). No verbose wrappers or redundant nesting. (NFR-003.)
- **Test:** `test_BC_5_12_001_token_count_within_budget()`

### AC-004 (traces to BC-5.12.001 — no verbose mode default)
By default, output is minimal. `--verbose` flag enables extended output with metadata. Without `--verbose`, no extra fields beyond the core result.
- **Test:** `test_BC_5_12_001_minimal_default_output()`

### AC-005 (traces to BC-5.11.002 — pretty-print flag)
`--pretty` flag enables indented JSON output. Default is compact (single-line JSON).
- **Test:** `test_BC_5_11_002_pretty_print_flag()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| Output formatters | `forge-mcp/src/output.rs` | Pure |
| Token counter | `forge-mcp/src/output.rs` | Pure |
| Stdout/stderr separation | `forge-mcp/src/main.rs` | Effectful |

## UX Screens
- N/A — CLI story

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Tool result contains binary data | Base64-encoded in JSON |
| EC-002 | Server list empty | `{ "servers": [] }` — valid empty JSON |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Output formatters | Pure | Transform data → JSON string |
| Token counter | Pure | Count bytes/tokens on string |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~800 |
| BC-5.11.002, BC-5.12.001 | ~600 |
| NFR-003 | ~300 |
| **Total** | **~1,700** |
| Agent context window | 200K |
| **Budget usage** | **~0.9%** |

## Tasks

1. [ ] Write failing tests for all 5 ACs
2. [ ] Define output schema types as Rust structs with serde
3. [ ] Implement stdout/stderr separation
4. [ ] Implement compact vs pretty-print JSON
5. [ ] Implement token count validation (NFR-003)
6. [ ] Verify Red Gate
7. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-023 | Subcommand dispatch established | Output goes through subcommand handlers | Token counting uses tiktoken or approximation |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| JSON stdout, diagnostics stderr | BC-5.11.002 | Never println!() JSON on stderr |
| Token budget (NFR-003) | nfr-catalog.md | Automated token count check |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| serde_json | >= 1.0 | JSON output | `serde_json::to_string()` / `to_string_pretty()` |
| tiktoken-rs | dev | Token count validation | `tokenize` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-mcp/src/output.rs` | Output formatters + token counter | NO — this story creates it |
| `crates/forge-mcp/tests/output_tests.rs` | Output schema tests | NO |
