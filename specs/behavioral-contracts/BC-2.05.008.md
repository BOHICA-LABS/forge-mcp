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

# BC-2.05.008 — Completion/Autocomplete Requests

## Summary

Sends `completion/complete` requests to MCP servers for argument autocompletion when users are composing prompt arguments or resource URIs. Parses and presents the completion results.

## Preconditions

- PRE-001: Connection is initialized.
- PRE-002: The server supports completions (indicated by server capabilities or inferred from available prompts/resources).
- PRE-003: A completion context is provided: either a `ref` (prompt or resource reference) and the `argument` being completed.

## Postconditions

- POST-001: `completion/complete` sends a request with:
  - `ref`: `{type: "ref/prompt", name: "<prompt_name>"}` or `{type: "ref/resource", uri: "<resource_uri>"}`
  - `argument`: `{name: "<arg_name>", value: "<partial_value>"}`
- POST-002: The server's `CompleteResult` is parsed: `completion.values` (list of strings), `completion.total` (optional total count), `completion.hasMore` (boolean).
- POST-003: Results are returned to the caller (TUI autocomplete dropdown, CLI tab-completion) as an ordered list.
- POST-004: If `hasMore` is true, the UI may indicate additional results are available.

## Invariants

- None specific. Completion is best-effort.

## Edge Cases

| ID | Condition | Expected Behavior |
|----|-----------|-------------------|
| EC-001 | Server returns empty completion list | Return empty results. No error. UI shows "no completions". |
| EC-002 | Server does not support completions | Return empty results. No error sent. Feature is gracefully unavailable. |
| EC-003 | Completion request times out (5s) | Return empty results. Log warning `E-PRT-009: Completion timeout for <ref>`. |
| EC-004 | User types faster than completion responses arrive | Cancel previous pending completion request. Send new one. Only latest results displayed. |
| EC-005 | `partial_value` is empty string | Valid. Server should return all possible completions for the argument. |
| EC-006 | Completion values contain special characters (spaces, quotes) | Return as-is. Escaping is the caller's responsibility. |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-001 | `completion/complete {ref: {type: "ref/prompt", name: "translate"}, argument: {name: "language", value: "en"}}` | `{completion: {values: ["english", "en-us", "en-gb"], hasMore: false}}` |
| TV-002 | `completion/complete` with empty partial value | Full list of possible values for the argument |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-003 | Server returns empty completion | `{completion: {values: [], hasMore: false}}` — no error |
| TV-004 | Completion times out after 5s | Empty results returned. Warning logged. |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-005 | Server returns JSON-RPC error for completion | `ProtocolError` returned. UI shows no completions. |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | Completion timeout does not block the UI thread | Integration test: slow server, verify UI remains responsive |
| VP-002 | Rapid typing cancels stale completion requests | Integration test: send 5 completions in 100ms, verify only last response displayed |
| VP-003 | Empty results do not cause errors | Unit test: empty response → verify no error |

## Traceability

- **L2 Capability**: CAP-005 (Core MCP Operations)
- **Priority**: P2
