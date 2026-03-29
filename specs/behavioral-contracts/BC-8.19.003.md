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

# BC-8.19.003 — Transport Compliance Validation

## Summary

Validates that MCP servers correctly implement the transport layer for both
stdio and Streamable HTTP transports. Checks encoding, framing, session
management, SSE fallback, and keepalive behavior per the MCP transport
specification.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | Target server is reachable via the transport under test (stdio or HTTP) |
| PRE-002 | For stdio: server process can be spawned with stdin/stdout/stderr accessible |
| PRE-003 | For HTTP: server endpoint URL is known and accepts HTTP POST |
| PRE-004 | Test runner can send raw bytes (not just parsed JSON) to validate encoding |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | stdio transport: all messages are UTF-8 encoded with newline delimiters |
| POST-002 | stdio transport: server diagnostic output goes to stderr only, never stdout |
| POST-003 | HTTP transport: server issues and manages `Mcp-Session-Id` header correctly |
| POST-004 | HTTP transport: SSE streaming works for long-running operations |
| POST-005 | HTTP transport: JSON response mode works for simple request/response |
| POST-006 | Ping/keepalive: server responds to `ping` within timeout |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | stdout is reserved exclusively for JSON-RPC messages (stdio transport) |
| INV-002 | Each JSON-RPC message is exactly one line (newline-delimited) on stdio |
| INV-003 | HTTP session ID, once issued, must be accepted for the session lifetime |
| INV-004 | SSE events follow `event: message\ndata: {json}\n\n` format |

## Edge Cases

| ID | Scenario | Expected Behavior | Source |
|----|----------|-------------------|--------|
| EC-001 | Server writes non-UTF-8 bytes to stdout (stdio) | Test detects encoding violation; marks as conformance failure |  |
| EC-002 | Server writes log/debug output to stdout instead of stderr | Test detects non-JSON content on stdout; flags as transport violation |  |
| EC-003 | HTTP request without `Mcp-Session-Id` after session established | Server may reject (400) or start new session; test validates consistent behavior |  |
| EC-004 | HTTP request with expired or invalid session ID | Server returns appropriate error (404 or 400) |  |
| EC-005 | Very large JSON-RPC message (> 1MB) over stdio | Server handles without truncation or crash |  |
| EC-006 | Concurrent HTTP requests on same session | Server handles correctly (ordering may vary) |  |
| EC-007 | SSE connection drops mid-stream | Client can reconnect; server resumes or restarts operation |  |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-HP-001 | stdio: send `{"jsonrpc":"2.0","id":1,"method":"ping"}\n` on stdin | Receive `{"jsonrpc":"2.0","id":1,"result":{}}\n` on stdout |
| TV-HP-002 | HTTP POST to `/mcp` with `initialize` request | Response includes `Mcp-Session-Id` header; body is valid `InitializeResult` |
| TV-HP-003 | HTTP GET to `/mcp` with `Accept: text/event-stream` | SSE stream opened; server sends events as they occur |
| TV-HP-004 | stdio: check stderr during normal operation | stderr may contain log lines; stdout contains only JSON-RPC |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-EC-001 | stdio: send message without trailing newline | Server either waits for newline or processes (implementation-defined); no crash |
| TV-EC-002 | HTTP: send request with `Mcp-Session-Id: invalid-uuid` | Error response (404 or 400) |
| TV-EC-003 | HTTP: send DELETE to `/mcp` with valid session ID | Session terminated; subsequent requests with that ID fail |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-ERR-001 | stdio: send binary garbage on stdin | Server rejects with parse error (-32700) or ignores; no crash |
| TV-ERR-002 | HTTP: POST with `Content-Type: text/plain` instead of `application/json` | 415 Unsupported Media Type or JSON-RPC parse error |
| TV-ERR-003 | HTTP: POST to non-existent endpoint `/mcp-wrong` | 404 Not Found |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | stdio stdout contains exclusively valid JSON-RPC messages | Byte-level stream analysis |
| VP-002 | HTTP session lifecycle (create → use → delete) works end-to-end | Integration test |
| VP-003 | SSE events are well-formed and parseable | SSE parser validation |
| VP-004 | Ping response time < 5 seconds under normal load | Timed ping test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-019 (Conformance Testing) |
| Domain Invariants | — |
| Edge Cases | — |
| Priority | P1 |
| NFRs | — |
