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

# BC-2.05.004 — Sampling Proxy to External LLM

## Summary

Handles `sampling/createMessage` requests from MCP servers by proxying them to a configured external LLM API. Translates between MCP's sampling message format and the LLM provider's API format. Supports tool calling within sampling requests.

## Preconditions

- PRE-001: The client advertised `sampling` capability during negotiation (BC-2.04.002).
- PRE-002: The sampling handler is registered with rmcp.
- PRE-003: A `sampling/createMessage` request is received from the server.

## Postconditions

- POST-001: The `CreateMessageParams` from the server are parsed: `messages`, `modelPreferences` (optional), `systemPrompt` (optional), `includeContext` (optional), `maxTokens`.
- POST-002: Messages are translated to the external LLM's API format (e.g., OpenAI chat completions format).
- POST-003: If `modelPreferences` contains hints, the best available model matching those hints is selected. If no hint or no match, the default configured model is used.
- POST-004: The request is sent to the external LLM API with appropriate authentication.
- POST-005: The LLM response is translated back to MCP's `CreateMessageResult` format: `role`, `content`, `model` (the actual model used), `stopReason`.
- POST-006: If the LLM response includes tool calls, they are included in the result's content as tool-call content items.
- POST-007: The full sampling request/response is logged in the traffic inspector for debugging.

## Invariants

- None specific beyond DI-001 (connection must be initialized).

## Edge Cases

| ID | Condition | Expected Behavior |
|----|-----------|-------------------|
| EC-001 | DEC-016 / FM-016: No LLM provider configured | Return MCP error response: `{code: -32603, message: "No LLM provider configured for sampling"}`. |
| EC-002 | LLM API returns HTTP 429 (rate limited) | Return MCP error response: `{code: -32603, message: "LLM rate limited. Retry after <N>s."}`. Include `Retry-After` value if provided by LLM API. |
| EC-003 | LLM API returns HTTP 500/502/503 | Return MCP error response: `{code: -32603, message: "LLM API error: <status>"}`. |
| EC-004 | LLM API timeout (configurable, default 120s) | Return MCP error response: `{code: -32603, message: "LLM API timeout after <N>s"}`. |
| EC-005 | `maxTokens` exceeds LLM model's context limit | Clamp to model's max and include warning in response metadata. |
| EC-006 | `modelPreferences.hints` requests a model not available | Fall back to default model. Log warning `E-SMP-001: Requested model hint <hint> not available. Using default: <model>`. |
| EC-007 | Sampling request contains image content | Forward to LLM if model supports vision. If not, return error: `{code: -32602, message: "Configured model does not support image input"}`. |
| EC-008 | Empty messages array | Return MCP error: `{code: -32602, message: "Empty messages array in sampling request"}`. |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-001 | Server sends `sampling/createMessage` with `messages: [{role: "user", content: {type: "text", text: "What is 2+2?"}}], maxTokens: 100` | LLM called, response: `{role: "assistant", content: {type: "text", text: "4"}, model: "gpt-4", stopReason: "endTurn"}` |
| TV-002 | Server includes `modelPreferences: {hints: [{name: "claude-3"}]}`, configured LLM is Anthropic | Claude 3 model selected. Response includes `model: "claude-3-sonnet-..."`. |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-003 | No LLM provider configured | MCP error: `{code: -32603, message: "No LLM provider configured for sampling"}` |
| TV-004 | LLM returns rate limit (429) with `Retry-After: 30` | MCP error includes "Retry after 30s" |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-005 | LLM API key is invalid (401) | MCP error: `{code: -32603, message: "LLM authentication failed"}` |
| TV-006 | Empty messages array | MCP error: `{code: -32602, message: "Empty messages array"}` |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | No LLM API credentials are leaked in MCP error responses | Unit test: verify error messages do not contain API keys |
| VP-002 | Sampling responses always include the actual model used | Unit test: verify `model` field is non-empty in all success responses |
| VP-003 | All sampling traffic is logged in traffic inspector | Integration test: verify log entries for request and response |

## Traceability

- **L2 Capability**: CAP-005 (Core MCP Operations)
- **Edge Cases**: DEC-016
- **Failure Modes**: FM-016
- **Priority**: P1
