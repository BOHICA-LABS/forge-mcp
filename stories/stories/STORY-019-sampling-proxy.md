---
document_type: story
story_id: STORY-019
epic_id: EPIC-02
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-014]
blocks: []
behavioral_contracts: [BC-2.05.004]
verification_properties: [VP-001]
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-019: Sampling Proxy to External LLM

## Narrative
- **As an** MCP server author testing sampling-enabled servers
- **I want to** have Forge MCP proxy sampling requests to a configured LLM
- **So that** I can test servers that rely on `sampling/createMessage`

## Acceptance Criteria

### AC-001 (traces to BC-2.05.004 postcondition — proxy request)
When a server sends `sampling/createMessage`, the `ClientHandler` forwards the request to the configured LLM API endpoint (OpenAI-compatible). Returns the LLM response back to the server as a `CreateMessageResult`.
- **Test:** `test_BC_2_05_004_sampling_proxy_success()`

### AC-002 (traces to BC-2.05.004 postcondition — no LLM configured)
When no LLM endpoint is configured (`FORGE_LLM_URL` unset), `create_message` returns `Err(E-PRO-007)`. The sampling capability is NOT advertised in this case (from STORY-014).
- **Test:** `test_BC_2_05_004_no_llm_configured_errors()`

### AC-003 (traces to BC-2.05.004 — model preference passthrough)
The `modelPreferences` field in the sampling request is forwarded to the LLM API as-is. Forge MCP does not override model selection.
- **Test:** `test_BC_2_05_004_model_preferences_passthrough()`

### AC-004 (traces to BC-2.05.004 — human-in-the-loop passthrough)
The `includeContext` and `stopSequences` fields are forwarded. The TUI shows a pending sampling request indicator when in interactive mode.
- **Test:** `test_BC_2_05_004_human_in_loop_fields()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `create_message` handler | `forge-core/src/handler.rs` | Effectful (HTTP to LLM) |
| LLM proxy config | `forge-core/src/config.rs` | Pure |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | LLM API returns error | Propagate as sampling error to server |
| EC-002 | LLM API rate-limited | Propagate 429 error |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| LLM proxy config | Pure | Struct with URL and API key |
| `create_message` | Effectful | HTTP outbound call |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~700 |
| BC-2.05.004 | ~400 |
| **Total** | **~1,100** |
| Agent context window | 200K |
| **Budget usage** | **~0.6%** |

## Tasks

1. [ ] Write failing tests (with wiremock LLM stub)
2. [ ] Implement `create_message` handler in ClientHandler
3. [ ] Implement LLM proxy HTTP client
4. [ ] Add `FORGE_LLM_URL` config reading
5. [ ] Verify Red Gate
6. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-014 | Sampling not advertised without config | No sampling in create_message if no URL | wiremock for LLM stub (not DTU) |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| No embedded LLM (out of scope) | PRD 1.5 | Only proxy, no local model |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| reqwest | >= 0.12 | HTTP to LLM API | `reqwest::Client` |
| wiremock | dev | LLM API stub | `WireMockServer` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-core/src/handler.rs` | create_message impl | YES (from STORY-014) |
| `crates/forge-core/src/llm_proxy.rs` | LLM proxy client | NO — this story creates it |
