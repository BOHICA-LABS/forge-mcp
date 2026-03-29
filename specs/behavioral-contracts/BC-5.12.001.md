---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-03-29T11:25:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: ""
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: "CLI Mode"
capability: "CAP-012"
lifecycle_status: active
introduced: v0.1.0
---

# BC-5.12.001 — Agent-Optimized Minimal Token Output

## Summary

The CLI produces compact JSON output optimized for LLM agent consumption. A
complete discover → inspect → call workflow must consume fewer than 500 tokens
(GPT-4 tokenizer). Output omits null fields, avoids pretty-printing by default,
and uses minimal JSON wrappers.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | A subcommand is invoked that produces JSON output |
| PRE-002 | The `--json` or `--output json` flag is set (or output defaults to JSON for agent mode) |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | JSON output contains no null-valued fields |
| POST-002 | JSON output is not pretty-printed by default (single line, no indentation) |
| POST-003 | JSON wrapper overhead is minimal: no envelope fields beyond `result` or top-level array |
| POST-004 | A discover → inspect → call workflow for a single server with 5 tools produces ≤ 500 GPT-4 tokens total across all 3 commands |
| POST-005 | Field names use short but unambiguous keys (e.g., `desc` not `description` when context is clear) |
| POST-006 | `--pretty` flag is available to opt into indented output when human readability is needed |

## Invariants

| ID | Invariant |
|----|-----------|
| NFR-003 | Agent token budget: discover → inspect → call < 500 tokens (GPT-4 tokenizer) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Server with 50+ tools in `list` output | Token count may exceed 500 for list alone, but the 3-step workflow (list → info on 1 tool → call) stays under 500 |
| EC-002 | Tool with very long description (> 200 chars) | Description is included but not truncated (user can pipe to jq for filtering); token budget measured on typical server |
| EC-003 | `--pretty` combined with `--json` | Pretty-printed JSON emitted; token budget is not a concern in human mode |
| EC-004 | Empty result (no tools, no servers) | Emit `[]` not `{"tools":[]}` — minimal wrapper |
| EC-005 | Tool with all optional fields null | All null fields omitted; only name and required fields present |
| EC-006 | Nested null fields in tool inputSchema | Null fields pruned at all nesting levels |

## Canonical Test Vectors

### Happy Path — Token Budget Verification

**Scenario:** Server `test-srv` has 5 tools: `read_file`, `write_file`, `search`, `execute`, `list_dir`.

| Step | Command | Expected stdout (compact) | Token Estimate |
|------|---------|--------------------------|----------------|
| 1. Discover | `forge-mcp list --json` | `[{"name":"test-srv","transport":"stdio","tools":5}]` | ~25 tokens |
| 2. Inspect | `forge-mcp info test-srv --json` | `{"name":"test-srv","tools":[{"name":"read_file","desc":"Read a file","input":{"type":"object","properties":{"path":{"type":"string"}}}},...]` | ~250 tokens |
| 3. Call | `forge-mcp call test-srv read_file '{"path":"/tmp/x"}' --json` | `{"content":[{"type":"text","text":"hello"}]}` | ~30 tokens |
| **Total** | | | **~305 tokens** |

### Edge Case

| Input | Expected Output | Notes |
|-------|----------------|-------|
| `forge-mcp list --json --pretty` | Indented JSON array | Token budget not enforced in pretty mode |
| `forge-mcp info srv --json` (srv has 0 tools) | `{"name":"srv","tools":[]}` | Minimal output |
| Tool response with null fields | `{"content":[{"type":"text","text":"ok"}]}` (no nulls) | Nulls pruned |

### Error

| Input | Expected Output | Exit Code |
|-------|----------------|-----------|
| `forge-mcp call srv tool '{}' --json` (server down) | `{"error":"connection_error","message":"..."}` | 2 |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | 3-step workflow on a 5-tool server produces ≤ 500 GPT-4 tokens total | Integration test + token counter |
| VP-002 | No null fields appear in any JSON output | Property test |
| VP-003 | Default output is single-line (no newlines within JSON value) | Unit test |
| VP-004 | `--pretty` flag produces indented multi-line JSON | Unit test |

## Traceability

| Source | Target |
|--------|--------|
| CAP-012 | This contract |
| NFR-003 | Token budget target |
| BC-5.11.002 | JSON output contract (this refines compactness) |
| BC-5.12.002 | Pipeable output (complementary) |
