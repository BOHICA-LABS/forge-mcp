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
subsystem: "MCP Protocol Operations"
capability: "CAP-005"
lifecycle_status: active
introduced: v0.1.0
---

# BC-2.05.003 — Prompt List and Retrieval with Pagination

## Summary

Lists all prompts from a connected MCP server with cursor-based pagination, retrieves individual prompts with arguments, and handles `notifications/prompts/list_changed` for cache invalidation.

## Preconditions

- PRE-001: Connection is initialized and `capabilities.prompts` is present in negotiated capabilities.
- PRE-002: For `prompts/get`: the prompt name exists in the cached prompt list.

## Postconditions

- POST-001: `prompts/list` exhausts all pagination pages (DI-019) and caches the complete prompt list.
- POST-002: Each prompt in the list contains: `name`, `description` (optional), `arguments` (list of `{name, description, required}`).
- POST-003: `prompts/get` sends `{name, arguments}` and returns `GetPromptResult` containing `description` (optional) and `messages` (list of prompt messages with `role` and `content`).
- POST-004: On `notifications/prompts/list_changed`, the prompt list cache is invalidated. Next `prompts/list` fetches fresh data.
- POST-005: Prompt arguments are validated against the prompt's argument schema before sending to server. Required arguments must be present.

## Invariants

- **DI-019**: All pagination pages MUST be exhausted for prompt listing.

## Edge Cases

| ID | Condition | Expected Behavior |
|----|-----------|-------------------|
| EC-001 | Prompt has required arguments but none provided | Return `Err(E-PRT-006: Missing required argument: <name> for prompt <prompt_name>)`. Not sent to server. |
| EC-002 | Prompt has no arguments (static prompt) | `prompts/get` sends `{name, arguments: {}}`. Valid. |
| EC-003 | Server returns 0 prompts | Valid state. Empty prompt list cached. |
| EC-004 | Prompt result contains embedded resource references (`resource` content type) | Return as-is. Resource resolution is the caller's responsibility. |
| EC-005 | Prompt name not in cached list | Return `Err(E-PRT-007: Unknown prompt: <name>)`. Suggest refreshing prompt list. |
| EC-006 | `list_changed` during `prompts/get` | Current `get` completes with the result. Cache is invalidated for next `list`. |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-001 | Server has 2 prompts: "summarize" (arg: text), "translate" (args: text, language) | `prompts/list` returns 2 entries with argument schemas. |
| TV-002 | `prompts/get {name: "summarize", arguments: {text: "Hello world"}}` | `GetPromptResult` with messages: `[{role: "user", content: {type: "text", text: "Summarize: Hello world"}}]` |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-003 | `prompts/get {name: "translate", arguments: {text: "hi"}}` — missing required `language` | `Err(E-PRT-006: Missing required argument: language for prompt translate)` |
| TV-004 | Server returns 0 prompts | Empty list cached. No error. |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-005 | `prompts/get {name: "nonexistent"}` | `Err(E-PRT-007: Unknown prompt: nonexistent)` |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | Pagination exhausts all pages (DI-019) | Integration test with multi-page prompt list |
| VP-002 | Required argument validation prevents sending incomplete requests | Unit test: prompt with required args, verify client-side rejection |
| VP-003 | list_changed invalidates cache | Integration test: list → list_changed → verify re-fetch |

## Traceability

- **L2 Capability**: CAP-005 (Core MCP Operations)
- **Domain Invariant**: DI-019
- **Priority**: P1
