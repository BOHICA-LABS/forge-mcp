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
subsystem: "Traffic Inspection"
capability: "CAP-009"
lifecycle_status: active
introduced: v0.1.0
---

# BC-4.09.001 — Transparent JSON-RPC Message Capture

## Summary

The traffic inspection subsystem captures all JSON-RPC messages flowing between MCP clients and servers without modifying their content. Capture preserves byte-identical payloads (DI-005) and wire order (DI-006). Both directions are captured (client→server and server→client), including server-initiated messages such as sampling requests, elicitation requests, and notifications. Every captured message is tagged with a high-resolution timestamp.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | At least one MCP server connection is active through Forge's proxy layer |
| PRE-002 | Message capture is enabled (default: enabled; can be disabled per-server via config) |
| PRE-003 | Capture buffer has available capacity (BC-4.09.003) |
| PRE-004 | rmcp handler extensibility allows message interception (ASM-015) |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Every JSON-RPC message transiting the proxy is captured |
| POST-002 | Captured message bytes are identical to wire bytes (DI-005) — no transformation, prettification, or reencoding |
| POST-003 | Messages are stored in wire order (DI-006) — the sequence matches transmission order |
| POST-004 | Each captured message includes: direction (client→server or server→client), timestamp (microsecond resolution), server identifier, raw payload |
| POST-005 | Server-initiated messages (sampling/createMessage, elicitation, notifications, logging) are captured identically to client-initiated messages |
| POST-006 | Capture does not block or delay message forwarding (non-blocking capture path) |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | DI-005: Captured bytes are byte-identical to wire bytes (no transformation) |
| INV-002 | DI-006: Wire order is preserved in the capture sequence |
| INV-003 | Capture adds < 1% latency overhead to message forwarding (DI-008) |
| INV-004 | No message is silently dropped — if capture fails, a warning is emitted (see BC-4.09.003 for buffer overflow) |
| INV-005 | Timestamp monotonically increases for messages captured in order |

## Edge Cases

| ID | Scenario | Expected Behavior | Source |
|----|----------|-------------------|--------|
| EC-001 | High message rate (>1000 messages/sec) | Capture continues; if buffer fills, FIFO eviction begins per BC-4.09.003 | DEC-011 |
| EC-002 | Very large message payload (>1MB) | Capture entire message; count against buffer memory limit | — |
| EC-003 | Malformed JSON in message (server sends invalid JSON-RPC) | Capture raw bytes as-is; do not validate or reject | — |
| EC-004 | Binary content in message (e.g., base64 encoded in JSON) | Capture as-is; JSON structure preserved | — |
| EC-005 | Concurrent messages on multiple server connections | Each message tagged with its server identifier; interleaved captures maintain per-connection wire order | — |
| EC-006 | Connection drops mid-message (partial frame) | Capture whatever bytes were received; mark as incomplete | — |
| EC-007 | Capture disabled for a specific server | Messages for that server flow through proxy but are not captured; other servers unaffected | — |
| EC-008 | Server sends batch JSON-RPC request (array of requests) | Capture as single message (the entire array); do not decompose | — |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-HP-001 | Client sends `{"jsonrpc":"2.0","method":"tools/list","id":1}` | Captured: direction=client→server, timestamp=T1, payload=exact bytes, server=srv-1 |
| TV-HP-002 | Server responds `{"jsonrpc":"2.0","result":{"tools":[...]},"id":1}` | Captured: direction=server→client, timestamp=T2 (T2 > T1), payload=exact bytes, server=srv-1 |
| TV-HP-003 | Server sends notification `{"jsonrpc":"2.0","method":"notifications/progress","params":{...}}` | Captured: direction=server→client, server-initiated |
| TV-HP-004 | Server sends sampling request `{"jsonrpc":"2.0","method":"sampling/createMessage","id":5}` | Captured: direction=server→client, server-initiated request |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-EC-001 | 1500 messages/sec sustained for 10 seconds | All 15000 messages captured (or evicted per BC-4.09.003 if buffer fills); none silently dropped |
| TV-EC-002 | 2MB JSON payload | Entire payload captured as single message; 2MB counted against buffer |
| TV-EC-003 | Invalid JSON: `{"method": "test"` (truncated) | Raw bytes captured; marked as-is without validation error |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-ERR-001 | Capture buffer is at 100% capacity with no eviction possible | Should not occur (FIFO always evicts); if implementation bug prevents eviction, log E-CAP-002 and skip capture for this message |
| TV-ERR-002 | Timestamp clock returns error | Use fallback monotonic counter; log warning "System clock unavailable, using monotonic counter" |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | For all captured messages: captured_bytes == wire_bytes (DI-005 byte-identity) | Property-based test |
| VP-002 | For all message sequences: capture order matches wire order (DI-006) | Sequence test |
| VP-003 | Capture latency overhead < 1% of baseline message forwarding time | Benchmark test |
| VP-004 | For all directions: both client→server and server→client messages are captured | Invariant test |
| VP-005 | For all message types (request, response, notification, batch): capture succeeds | Exhaustive type test |

## Traceability

| Item | Reference |
|------|-----------|
| L2 Capability | CAP-009 (Message Capture & Analysis) |
| Domain Invariant | DI-005 (byte-identical capture), DI-006 (wire order), DI-008 (< 1% overhead) |
| Decision | DEC-011 (high message rate handling) |
| Assumption | ASM-015 (rmcp handler extensibility for capture) |
| Related BCs | BC-4.09.002 (timing analysis), BC-4.09.003 (buffer management), BC-3.08.001 (rendering) |
