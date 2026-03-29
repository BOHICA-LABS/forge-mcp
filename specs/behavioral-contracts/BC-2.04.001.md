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
capability: "CAP-004"
lifecycle_status: active
introduced: v0.1.0
---

# BC-2.04.001 — Bidirectional Capability Negotiation

## Summary

Performs the MCP capability negotiation handshake: sends `initialize` with client info and capabilities, parses the server's `InitializeResult` (serverInfo, capabilities, protocolVersion), sends `notifications/initialized`, and gates all subsequent method calls on the negotiated capability set.

## Preconditions

- PRE-001: A transport connection is established (BC-1.02.001 or BC-1.02.002).
- PRE-002: No previous initialize handshake has been completed on this connection (first-time only; re-negotiation requires reconnection).

## Postconditions

- POST-001: An `initialize` JSON-RPC request is sent containing:
  - `clientInfo`: `{name: "forge-mcp", version: "<semver>"}`
  - `protocolVersion`: the latest protocol version supported by Forge
  - `capabilities`: the client's advertised capabilities (per BC-2.04.002)
- POST-002: The server's `InitializeResult` is parsed and stored:
  - `serverInfo.name` and `serverInfo.version`
  - `capabilities` object (tools, resources, prompts, logging, experimental)
  - `protocolVersion` (the server's supported version)
- POST-003: `notifications/initialized` is sent to the server after successful parse.
- POST-004: A `NegotiatedCapabilities` record is created and stored in the connection context, containing the intersection/union of client and server capabilities.
- POST-005: All subsequent MCP method calls are gated on `NegotiatedCapabilities`:
  - `tools/list`, `tools/call` → requires server `capabilities.tools`
  - `resources/list`, `resources/read` → requires server `capabilities.resources`
  - `prompts/list`, `prompts/get` → requires server `capabilities.prompts`
  - `logging/setLevel` → requires server `capabilities.logging`
  - `completion/complete` → requires server `capabilities.completions` (if advertised)
- POST-006: Attempting a method the server does not support returns `Err(E-CAP-001: Server does not support <method>. Missing capability: <cap>)` without sending the request.

## Invariants

- **DI-001**: The initialize/initialized handshake MUST complete before any other MCP method is called on the connection. Any method call before negotiation returns `Err(E-CAP-002: Connection not initialized)`.
- **DI-002**: Methods are gated on negotiated capabilities. Forge never sends a request the server has not advertised support for.

## Edge Cases

| ID | Condition | Expected Behavior |
|----|-----------|-------------------|
| EC-001 | Server returns empty capabilities object `{}` | No methods are available. All tool/resource/prompt calls return `Err(E-CAP-001)`. Connection is valid but limited to ping. |
| EC-002 | Server returns unknown capability fields | Unknown fields are stored but ignored. Forward-compatible. |
| EC-003 | Server's `protocolVersion` differs from client's | Accept the connection. Version compatibility handled by BC-2.04.003. |
| EC-004 | Server returns malformed InitializeResult (missing required fields) | Return `Err(E-CAP-003: Malformed InitializeResult: missing <field>)`. Connection transitions to `Failed`. |
| EC-005 | Initialize request times out (30s) | Return `Err(E-CON-004)`. Connection closed. |
| EC-006 | Client sends a method call before initialized notification is sent | Blocked by DI-001 gate. Returns `Err(E-CAP-002)`. |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-001 | Server returns `{serverInfo: {name: "test-server", version: "1.0"}, capabilities: {tools: {listChanged: true}}, protocolVersion: "2025-03-26"}` | Negotiation completes. `tools/list` is available. `resources/list` is gated (not advertised). |
| TV-002 | Server returns full capabilities (tools, resources, prompts, logging) | All method families available. |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-003 | Server returns `{capabilities: {}}` | Negotiation completes. All method calls return `Err(E-CAP-001)`. Ping still works. |
| TV-004 | Client attempts `tools/list` before initialize | `Err(E-CAP-002: Connection not initialized)` |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-005 | Server returns `{"error": {"code": -32600, "message": "Invalid Request"}}` for initialize | `Err(E-CAP-004: Server rejected initialize: Invalid Request)`. Connection closed. |
| TV-006 | Server returns JSON missing `protocolVersion` field | `Err(E-CAP-003: Malformed InitializeResult: missing protocolVersion)` |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | No MCP method is sent before initialized notification | Integration test with method call timing assertions |
| VP-002 | Every unsupported method call is rejected locally (never sent to server) | Unit test: mock server with limited caps, attempt all methods, verify no wire traffic for unsupported ones |
| VP-003 | Unknown server capability fields do not cause parse errors | Fuzz test with random capability objects |

## Traceability

- **L2 Capability**: CAP-004 (Capability Negotiation)
- **Domain Invariants**: DI-001, DI-002
- **Priority**: P0
