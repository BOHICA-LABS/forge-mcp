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

# BC-2.05.001 — Tool List and Invocation with Pagination

## Summary

Lists all tools available from a connected MCP server using cursor-based pagination (exhausting all pages), and invokes tools with provided arguments. Correctly distinguishes tool-level errors (`isError: true` in response content) from protocol-level errors (JSON-RPC error objects).

## Preconditions

- PRE-001: Connection is initialized and `capabilities.tools` is present in the server's negotiated capabilities.
- PRE-002: For `tools/call`: the tool name exists in the cached tool list.

## Postconditions

- POST-001: `tools/list` sends paginated requests with cursor until the server returns a response with no `nextCursor` (or `nextCursor: null`).
- POST-002: The complete tool list is assembled from all pages and cached in the connection context.
- POST-003: Each tool in the list contains: `name`, `description` (optional), `inputSchema` (JSON Schema).
- POST-004: `tools/call` sends a request with `{name, arguments}` and returns the `CallToolResult`.
- POST-005: If `CallToolResult` contains `content` items with `isError: true`, the result is returned as a `ToolError` (not a protocol error). The caller can inspect the error content.
- POST-006: If the server responds with a JSON-RPC error object (error code, error message), it is returned as a `ProtocolError`.
- POST-007: If `notifications/tools/list_changed` is received from the server, the cached tool list is invalidated. Next `tools/list` call fetches fresh data.

## Invariants

- **DI-019**: All pagination pages MUST be exhausted. Forge never presents a partial tool list to the user.
- **DI-020**: Tool errors (`isError: true`) and protocol errors (JSON-RPC error) are distinct types. They are never conflated.

## Edge Cases

| ID | Condition | Expected Behavior |
|----|-----------|-------------------|
| EC-001 | DEC-020: Pagination cursor loops (server returns the same cursor repeatedly) | Detect loop after seeing the same cursor twice. Break pagination. Emit warning `E-PRT-001: Pagination cursor loop detected for tools/list`. Return tools collected so far. |
| EC-002 | DEC-021: Tool list changes during pagination (list_changed notification mid-page) | Complete current pagination. Then re-fetch from the beginning (invalidated cache). |
| EC-003 | Server returns 0 tools | Valid state. Empty tool list cached. User informed: "Server <name> has no tools." |
| EC-004 | Tool invocation with invalid arguments (schema mismatch) | Forge performs client-side JSON Schema validation against `inputSchema`. If invalid, return `Err(E-PRT-002: Argument validation failed: <details>)` without sending to server. |
| EC-005 | Tool invocation for a tool not in cached list | Return `Err(E-PRT-003: Unknown tool: <name>)`. Suggest refreshing tool list. |
| EC-006 | Server returns tool with duplicate names across pages | Last occurrence wins (page order). Warning emitted. |
| EC-007 | Tool result contains mixed content (text + image + isError) | Return all content items as-is. The `isError` applies to the tool result as a whole. |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-001 | Server has 3 tools, single page (no cursor) | `tools/list` returns 3 tools. No pagination needed. |
| TV-002 | Server has 150 tools, 3 pages of 50 | 3 `tools/list` requests sent. 150 tools in cache. |
| TV-003 | `tools/call {name: "read_file", arguments: {path: "/tmp/test.txt"}}` | `CallToolResult` with `content: [{type: "text", text: "file contents"}]` |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-004 | Pagination cursor loop (cursor "abc" returned twice) | Pagination stops. Warning `E-PRT-001`. Partial tool list returned. |
| TV-005 | Tool returns `isError: true` content | Result type is `ToolError`, not `ProtocolError`. Error content accessible. |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-006 | `tools/call` for tool "nonexistent" | `Err(E-PRT-003: Unknown tool: nonexistent)` — not sent to server |
| TV-007 | Server returns JSON-RPC error `{code: -32602, message: "Invalid params"}` | `ProtocolError` returned with code and message. Distinct from tool error. |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | All pagination pages are exhausted (DI-019) | Integration test: server with 5 pages, verify 5 requests sent |
| VP-002 | Tool errors and protocol errors have different types (DI-020) | Type system enforcement + unit test |
| VP-003 | Client-side argument validation catches schema mismatches | Unit test with invalid arguments against known schema |

## Traceability

- **L2 Capability**: CAP-005 (Core MCP Operations)
- **Domain Invariants**: DI-019, DI-020
- **Edge Cases**: DEC-020, DEC-021
- **Priority**: P0
