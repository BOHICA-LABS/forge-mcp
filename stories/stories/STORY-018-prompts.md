---
document_type: story
story_id: STORY-018
epic_id: EPIC-02
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-013]
blocks: [STORY-045]
behavioral_contracts: [BC-2.05.003]
verification_properties: [VP-001, VP-003]
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-018: Prompt List & Retrieval with Pagination

## Narrative
- **As an** AI Platform Engineer
- **I want to** list and retrieve prompt templates from MCP servers
- **So that** I can see what prompts a server offers and fetch them with arguments

## Acceptance Criteria

### AC-001 (traces to BC-2.05.003 postcondition — prompt list)
`connection.list_prompts()` returns `Vec<Prompt>` with pagination support. Each prompt has `name: String`, `description: Option<String>`, `arguments: Vec<PromptArgument>`.
- **Test:** `test_BC_2_05_003_list_prompts_paginated()`

### AC-002 (traces to BC-2.05.003 postcondition — prompt get)
`connection.get_prompt(name, arguments)` returns `PromptResult` with `description: Option<String>`, `messages: Vec<PromptMessage>`. Arguments are passed to the server as-is.
- **Test:** `test_BC_2_05_003_get_prompt_with_args()`

### AC-003 (traces to BC-2.05.003 — capability guard)
If server didn't advertise `prompts` capability, `list_prompts()` returns `Err(E-PRO-003)`.
- **Test:** `test_BC_2_05_003_capability_guard()`

### AC-004 (traces to BC-2.05.003 — list_changed notification)
When `notifications/prompts/list_changed` arrives, the cached prompt list is invalidated.
- **Test:** `test_BC_2_05_003_list_changed_invalidates_cache()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `list_prompts()`, `get_prompt()` | `forge-core/src/protocol.rs` | Effectful |

## UX Screens
- SCR-003 (Capability Browser, Prompts tab)

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Prompt with required argument missing | Server returns error, propagated |
| EC-002 | Empty prompt list | Empty Vec, no error |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `list_prompts()`, `get_prompt()` | Effectful | RPC calls |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~600 |
| BC-2.05.003 | ~400 |
| **Total** | **~1,000** |
| Agent context window | 200K |
| **Budget usage** | **~0.5%** |

## Tasks

1. [ ] Write failing tests for all 4 ACs
2. [ ] Implement `list_prompts()` (reuse pagination from STORY-016)
3. [ ] Implement `get_prompt()` with argument passing
4. [ ] Handle list_changed notification
5. [ ] Verify Red Gate
6. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-016 | Pagination reusable | Import from pagination module | |
| STORY-017 | list_changed pattern established | Same pattern for prompts | |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Capability guard | STORY-013 pattern | check server_capabilities().prompts |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| (no new deps) | | | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-core/src/protocol.rs` | list_prompts(), get_prompt() added | YES (from STORY-016) |
