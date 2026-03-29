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
subsystem: "Server Discovery & Connection Management"
capability: "CAP-002"
lifecycle_status: active
introduced: v0.1.0
---

# BC-1.02.002 — Streamable HTTP Transport Connection Establishment

## Summary

Establishes a connection to an MCP server via Streamable HTTP transport. Connects to the server URL, manages the `Mcp-Session-Id` header for session continuity, validates TLS certificates, and completes the MCP initialize/initialized handshake.

## Preconditions

- PRE-001: A `ServerEntry` with `transport: Http` exists in the unified registry.
- PRE-002: The `ServerEntry` has a valid `url` field (well-formed HTTPS or HTTP URL).
- PRE-003: The server is marked as `enabled: true`.
- PRE-004: Network connectivity to the server host is available.

## Postconditions

- POST-001: An HTTP client connection is established to the configured URL using Streamable HTTP transport (single-endpoint POST model per MCP 2025-11-25, not the deprecated HTTP+SSE dual-endpoint model). Note: `/sse`-style legacy endpoint paths are non-compliant unless the server exposes the 2025-11-25 single-endpoint Streamable HTTP semantics at that path.
- POST-002: If the server returns a `Mcp-Session-Id` header in the initialize response, it is stored and sent with all subsequent requests.
- POST-003: Custom `headers` from the config are included in every HTTP request to the server.
- POST-004: The MCP `initialize` request is sent and a valid `InitializeResult` is received.
- POST-005: The `notifications/initialized` notification is sent after successful initialize.
- POST-006: The connection is registered in the session's connection pool with state `Connected`.
- POST-007: TLS certificate validation uses the system certificate store by default.

## Invariants

- **DI-001**: Capability negotiation (initialize/initialized) MUST complete before any other MCP method is called.
- **DI-004**: Transport layer is managed exclusively by rmcp.
- **DI-018**: Only Streamable HTTP transport is supported. The deprecated SSE transport MUST NOT be used.

## Edge Cases

| ID | Condition | Expected Behavior |
|----|-----------|-------------------|
| EC-001 | FM-004: TLS certificate validation fails (self-signed, expired, hostname mismatch) | Return `Err(E-CON-004: TLS error — <detail> for <hostname>)`. Do NOT bypass validation by default. |
| EC-002 | Server returns HTTP 401/403 on initialize | Return `Err(E-CON-009: Authentication failed: HTTP <status> for <url>)`. Include response body if present. |
| EC-003 | Server URL uses `http://` (not HTTPS) | Allow connection but emit warning `E-CON-010: Insecure HTTP connection to <url>`. |
| EC-004 | Server does not return `Mcp-Session-Id` header | Connection proceeds without session ID. Re-establishment (BC-1.02.003) will create a new session. |
| EC-005 | DNS resolution failure | Return `Err(E-CON-001: DNS resolution failed for <hostname>)`. |
| EC-006 | Server returns HTTP 502/503/504 (gateway errors) | Return `Err(E-CON-011: Server unavailable: HTTP <status> for <url>)`. Reconnection handled by BC-1.02.003. |
| EC-007 | Connection timeout (configurable, default 30s) | Return `Err(E-CON-003: timeout after <seconds>s waiting for server response)`. |
| EC-008 | Server URL contains path (e.g., `https://api.example.com/mcp/v1`) | Use full URL as-is. Do not strip or modify the path. |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-001 | `url: "https://mcp.example.com/mcp", headers: {"Authorization": "Bearer tok123"}` | Connection established, initialize handshake completes, `Mcp-Session-Id` stored if returned |
| TV-002 | Server returns `Mcp-Session-Id: sess-abc-123` | Session ID stored, subsequent requests include `Mcp-Session-Id: sess-abc-123` header |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-003 | `url: "https://self-signed.example.com"` with self-signed cert | `Err(E-CON-004: TLS error — self-signed certificate for self-signed.example.com)` |
| TV-004 | `url: "http://localhost:8080"` (plain HTTP) | Connection succeeds, warning `E-CON-010` emitted |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-005 | `url: "https://nonexistent.invalid"` | `Err(E-CON-001: DNS resolution failed for nonexistent.invalid)` |
| TV-006 | Server returns HTTP 403 | `Err(E-CON-009: Authentication failed: HTTP 403 for https://example.com/mcp)` |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | TLS validation is never bypassed without explicit user opt-in | Integration test with self-signed cert, verify rejection |
| VP-002 | Custom headers are included in every request, not just initialize | Integration test with header inspection middleware |
| VP-003 | Deprecated SSE transport is never used | Static analysis: no SSE transport construction in codebase |

## Traceability

- **L2 Capability**: CAP-002 (Transport Connection)
- **Domain Invariants**: DI-001, DI-004, DI-018
- **Failure Modes**: FM-004
- **Priority**: P0
