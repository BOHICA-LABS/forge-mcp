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

# BC-2.05.010 — Tool Error vs Protocol Error Distinction

## Summary

Enforces a clear distinction between tool-level errors (`isError: true` in `CallToolResult` content) and protocol-level errors (JSON-RPC error objects). These two error types are represented as different types in the Forge type system, displayed differently in the traffic inspector, and counted separately in health metrics.

## Preconditions

- PRE-001: A `tools/call` response has been received from the server.

## Postconditions

- POST-001: If the response is a successful JSON-RPC result with `content` containing items where `isError: true`:
  - The result is classified as a `ToolError`.
  - The error content (text or other content types) is extracted and available to the caller.
  - In the traffic inspector, displayed with a yellow "Tool Error" badge.
  - Health metrics: increments `tool_errors` counter for the server.
- POST-002: If the response is a JSON-RPC error object (`{error: {code, message, data}}`):
  - The result is classified as a `ProtocolError`.
  - The error code, message, and optional data are extracted.
  - In the traffic inspector, displayed with a red "Protocol Error" badge.
  - Health metrics: increments `protocol_errors` counter for the server.
- POST-003: If the response is a successful result with no `isError` flags:
  - The result is classified as `Success`.
  - Health metrics: increments `successful_calls` counter.
- POST-004: The type system enforces this distinction at compile time (Rust enums). Callers MUST pattern-match on the result type.

## Invariants

- **DI-020**: Tool errors and protocol errors are NEVER conflated. They are distinct enum variants. A tool error is a valid protocol response indicating the tool failed. A protocol error indicates the protocol itself failed (invalid request, method not found, etc.).

## Edge Cases

| ID | Condition | Expected Behavior |
|----|-----------|-------------------|
| EC-001 | DEC-023: Response has `isError: true` in some content items but not others | The entire result is classified as `ToolError`. Mixed content (some error, some success) is preserved — caller sees all items. |
| EC-002 | `isError: true` with empty content text | `ToolError` with empty error message. Valid but unusual. Log warning `E-PRT-011: Tool error with empty content`. |
| EC-003 | JSON-RPC error with non-standard error code | `ProtocolError` with the provided code. Non-standard codes are accepted (forward-compatible). |
| EC-004 | JSON-RPC error with `data` field containing structured info | `data` is preserved in the `ProtocolError` and displayed in traffic inspector. |
| EC-005 | Tool returns `isError: false` explicitly | Treated as `Success` (same as absent `isError`). |
| EC-006 | Server returns both a result and an error in the same JSON-RPC response | Invalid JSON-RPC. Treat as `ProtocolError` (error takes precedence per JSON-RPC spec). |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-001 | `{result: {content: [{type: "text", text: "result data"}]}}` | `Success` — content returned. |
| TV-002 | `{result: {content: [{type: "text", text: "file not found", isError: true}]}}` | `ToolError` — error content: "file not found". Traffic inspector: yellow badge. |
| TV-003 | `{error: {code: -32602, message: "Invalid params"}}` | `ProtocolError` — code: -32602. Traffic inspector: red badge. |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-004 | `{result: {content: [{type: "text", text: "ok"}, {type: "text", text: "warning", isError: true}]}}` | `ToolError` (mixed content). Both items preserved. |
| TV-005 | `{error: {code: -32001, message: "Custom error", data: {detail: "extra info"}}}` | `ProtocolError` with `data.detail` accessible. |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-006 | Response has both `result` and `error` fields | `ProtocolError` (error takes precedence). Warning logged about invalid JSON-RPC. |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | `ToolError` and `ProtocolError` are distinct types (DI-020) | Compile-time enforcement via Rust enum. Static analysis: no `as` casts between them. |
| VP-002 | Traffic inspector displays different badges for tool vs protocol errors | UI test: inject both error types, verify distinct visual treatment |
| VP-003 | Health metrics count tool errors and protocol errors separately | Integration test: inject errors, verify separate counter increments |
| VP-004 | `isError: true` never produces a `ProtocolError` | Unit test: all `isError: true` variants → verify `ToolError` type |

## Traceability

- **L2 Capability**: CAP-005 (Core MCP Operations)
- **Domain Invariant**: DI-020
- **Edge Cases**: DEC-023
- **Priority**: P0
