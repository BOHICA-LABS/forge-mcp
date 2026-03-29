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
subsystem: "Conformance Testing"
capability: "CAP-019"
lifecycle_status: active
introduced: v0.1.0
---

# BC-8.19.002 — Method Coverage and Error Handling Conformance

## Summary

Exercises the full MCP method surface (~25 methods) against a target server,
validating correct response formats, error handling semantics, and method
coverage. Distinguishes between JSON-RPC transport errors and MCP tool-level
errors (isError flag). Targets ≥ 90% method coverage per NFR-010.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | Target MCP server has completed `initialize` / `initialized` handshake |
| PRE-002 | Server's advertised capabilities are known from `InitializeResult` |
| PRE-003 | Test runner has a catalog of all MCP spec methods with expected request/response shapes |
| PRE-004 | Server is in a stable state (not mid-shutdown or mid-restart) |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Every advertised method has been exercised at least once |
| POST-002 | Response format for each method matches MCP specification schema |
| POST-003 | Error responses use correct JSON-RPC error codes (-32600, -32601, -32602, -32603) |
| POST-004 | Tool call errors use `isError: true` in `CallToolResult`, NOT JSON-RPC errors |
| POST-005 | Method coverage percentage is computed and reported |
| POST-006 | Coverage ≥ 90% of all spec methods (NFR-010) |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | JSON-RPC errors and MCP tool errors are never conflated |
| INV-002 | Every response has a valid JSON-RPC `id` matching the request |
| INV-003 | Notification methods (no `id`) never receive responses |
| INV-004 | Method coverage percentage is monotonically non-decreasing during a test run |

## Edge Cases

| ID | Scenario | Expected Behavior | Source |
|----|----------|-------------------|--------|
| EC-001 | Server disconnects mid-test-run | Test runner captures partial results; uncompleted methods marked as "error" not "fail" | DEC-014 |
| EC-002 | Server returns success for a method it should not support | Flagged as conformance violation (capability lie) | DEC-012 |
| EC-003 | Server returns non-standard error code | Logged as warning; conformance check for error code range fails |  |
| EC-004 | Tool call returns `isError: true` with empty content | Valid per spec; test records as tool-level error |  |
| EC-005 | Server sends progress notifications for long-running tool calls | Test validates progress token format and ordering |  |
| EC-006 | Batch JSON-RPC request with multiple methods | Server processes each independently; partial failures don't cascade |  |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-HP-001 | `tools/list` on server with tools capability | `ListToolsResult` with `tools` array; each tool has `name`, `inputSchema` |
| TV-HP-002 | `tools/call` with valid tool name and matching input schema | `CallToolResult` with `content` array, `isError` absent or false |
| TV-HP-003 | `resources/list` on server with resources capability | `ListResourcesResult` with `resources` array |
| TV-HP-004 | `prompts/list` on server with prompts capability | `ListPromptsResult` with `prompts` array |
| TV-HP-005 | `ping` request | Empty result `{}` (JSON-RPC success) |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-EC-001 | `tools/call` with valid tool name but invalid input schema | `CallToolResult` with `isError: true` (tool-level error, NOT JSON-RPC error) |
| TV-EC-002 | Request with unknown method name `foo/bar` | JSON-RPC error: method not found (-32601) |
| TV-EC-003 | `tools/call` with non-existent tool name | `CallToolResult` with `isError: true` OR JSON-RPC error (spec allows both) |
| TV-EC-004 | Request with malformed JSON-RPC envelope (missing `jsonrpc` field) | JSON-RPC error: invalid request (-32600) |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-ERR-001 | `tools/call` with missing `name` field | JSON-RPC error: invalid params (-32602) |
| TV-ERR-002 | `resources/read` with non-existent URI | Error (resource not found); exact code server-dependent |
| TV-ERR-003 | Any method call after server has shut down | Connection error or JSON-RPC internal error (-32603) |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | Method coverage ≥ 90% of all MCP spec methods | Automated coverage counter |
| VP-002 | Zero conflation of JSON-RPC errors with tool-level isError semantics | Automated semantic check |
| VP-003 | All response envelopes are valid JSON-RPC 2.0 | Schema validation |
| VP-004 | Partial results are captured on server disconnect | Disconnect simulation test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-019 (Conformance Testing) |
| Domain Invariants | DI-014 (exit code semantics) |
| Edge Cases | DEC-012, DEC-014 |
| Priority | P1 |
| NFRs | NFR-010 (≥ 90% method coverage) |
