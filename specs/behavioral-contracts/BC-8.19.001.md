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

# BC-8.19.001 — Capability Negotiation Conformance Validation

## Summary

Validates that an MCP server correctly implements capability negotiation during
the `initialize` handshake. Tests protocol version agreement, capability
advertisement accuracy, and graceful handling of unknown or mismatched
capabilities.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | Target MCP server is reachable via configured transport (stdio or HTTP) |
| PRE-002 | Server accepts JSON-RPC 2.0 messages |
| PRE-003 | Conformance test runner has a valid client implementation for sending `initialize` |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Server responds to `initialize` with a valid `InitializeResult` containing `protocolVersion`, `capabilities`, and `serverInfo` |
| POST-002 | Negotiated protocol version is the highest mutually supported version |
| POST-003 | Server does not invoke methods for capabilities it did not advertise |
| POST-004 | Server returns appropriate error for methods corresponding to unadvertised capabilities |
| POST-005 | Test results include pass/fail for each capability negotiation check |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | The `initialize` → `initialized` handshake must complete before any other method calls |
| INV-002 | Server MUST NOT send requests to client before `initialized` notification |
| INV-003 | Capability advertisement is immutable for the lifetime of a session |

## Edge Cases

| ID | Scenario | Expected Behavior | Source |
|----|----------|-------------------|--------|
| EC-001 | Server advertises capabilities it does not actually support (lies about capabilities) | Test detects mismatch by exercising each advertised capability and flagging failures | DEC-012 |
| EC-002 | Client requests protocol version the server does not support | Server responds with its highest supported version; test validates downgrade | DEC-015 |
| EC-003 | Client sends unknown capability in `initialize` request | Server ignores unknown capabilities and responds normally | Spec |
| EC-004 | Server omits `capabilities` field entirely | Test flags as conformance failure — field is required | Spec |
| EC-005 | Multiple rapid `initialize` calls in sequence | Server rejects or handles gracefully (session already initialized) | Spec |
| EC-006 | Client sends `initialize` with empty capabilities object | Server responds with its capabilities regardless of client capabilities | Spec |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-HP-001 | `initialize` with `protocolVersion: "2025-03-26"`, `capabilities: { roots: { listChanged: true } }`, valid `clientInfo` | `InitializeResult` with `protocolVersion: "2025-03-26"`, non-empty `capabilities`, valid `serverInfo` |
| TV-HP-002 | After successful init, call `tools/list` when server advertised `tools` capability | Valid `ListToolsResult` with tool definitions |
| TV-HP-003 | Send `initialized` notification after `InitializeResult` | No error; server is ready to accept further requests |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-EC-001 | `initialize` with `protocolVersion: "9999-01-01"` | Server responds with its actual highest supported version, not the requested version |
| TV-EC-002 | Call `tools/list` when server did NOT advertise `tools` capability | Error response: method not found or capability not supported |
| TV-EC-003 | Call `resources/list` before `initialize` completes | Error response or connection rejection |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-ERR-001 | `initialize` with missing `protocolVersion` field | JSON-RPC error: invalid params (-32602) |
| TV-ERR-002 | `initialize` with malformed `clientInfo` (missing `name`) | JSON-RPC error: invalid params (-32602) |
| TV-ERR-003 | Non-JSON-RPC message sent as first message | Server rejects or ignores; does not crash |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | All advertised capabilities are exercisable without error | Automated conformance test |
| VP-002 | Unadvertised capabilities return appropriate errors | Automated conformance test |
| VP-003 | Protocol version negotiation follows spec downgrade rules | Automated conformance test |
| VP-004 | Pre-initialization requests are rejected | Automated conformance test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-019 (Conformance Testing) |
| Domain Invariants | DI-014 (exit code semantics) |
| Edge Cases | DEC-012, DEC-015 |
| Priority | P1 |
| NFRs | NFR-010 (≥ 90% method coverage) |
