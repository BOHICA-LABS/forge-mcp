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
subsystem: "Health Monitoring"
capability: "CAP-013"
lifecycle_status: active
introduced: v0.1.0
---

# BC-6.13.002 — Error Rate Trend Tracking (Protocol vs Tool Errors)

## Summary

The monitoring subsystem tracks two distinct error categories: **protocol errors**
(JSON-RPC level: parse errors, invalid request, method not found, internal error)
and **tool errors** (tool execution returned `isError=true` in the MCP response).
These categories are never conflated. Error rates are computed as sliding window
ratios with time-series trend data.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | At least one MCP server connection is active and being monitored |
| PRE-002 | The monitoring subsystem is receiving request/response observations |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Protocol errors are counted separately from tool errors |
| POST-002 | Error rate is computed as `errors / total_requests` over a configurable sliding window |
| POST-003 | Time-series trend data is maintained for the last N minutes (configurable, default: 30) |
| POST-004 | Each error event includes: timestamp, category (protocol/tool), method name, error code/message |
| POST-005 | Error rates are available per-server and per-method |
| POST-006 | Trend direction (increasing/stable/decreasing) is derivable from the time-series data |

## Invariants

| ID | Invariant |
|----|-----------|
| DI-020 | Protocol errors and tool errors are tracked in separate counters and never conflated |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Tool returns `isError=true` with valid JSON-RPC envelope (DEC-023) | Counted as tool error only, NOT protocol error. JSON-RPC was successful. |
| EC-002 | JSON-RPC error -32600 (invalid request) | Counted as protocol error only |
| EC-003 | Server returns both: JSON-RPC error wrapping a tool error | Counted as protocol error (the outer envelope is the error) |
| EC-004 | Zero requests in window | Error rate is 0.0 (not NaN or undefined) |
| EC-005 | All requests are errors | Error rate is 1.0 |
| EC-006 | Error rate transitions from 0% to 50% in one window | Trend: increasing; alert threshold may trigger (see BC-6.14.001) |
| EC-007 | Time-series window rolls over with old data expiring | Graceful decay; no discontinuities in trend line |
| EC-008 | Notification messages (no response) that fail at transport | Counted as protocol error (transport failure) |

## Canonical Test Vectors

### Happy Path

| Scenario | Protocol Errors | Tool Errors | Total Requests | Protocol Rate | Tool Rate |
|----------|----------------|-------------|----------------|---------------|-----------|
| 100 requests, all succeed | 0 | 0 | 100 | 0.00 | 0.00 |
| 100 requests, 5 tool errors | 0 | 5 | 100 | 0.00 | 0.05 |
| 100 requests, 3 protocol errors | 3 | 0 | 100 | 0.03 | 0.00 |
| 100 requests, 3 protocol + 5 tool errors | 3 | 5 | 100 | 0.03 | 0.05 |

### Edge Case (DEC-023 — Tool Error is Not Protocol Error)

| Input | Classification | Rationale |
|-------|---------------|-----------|
| JSON-RPC response: `{"jsonrpc":"2.0","id":1,"result":{"content":[{"type":"text","text":"fail"}],"isError":true}}` | Tool error | Valid JSON-RPC response; tool flagged error |
| JSON-RPC response: `{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"method not found"}}` | Protocol error | JSON-RPC error envelope |
| Transport timeout (no response) | Protocol error | Transport-level failure |

### Error

| Scenario | Expected Behavior |
|----------|-------------------|
| Monitoring subsystem cannot parse a response | Log internally; do not count as either category (monitoring failure, not server error) |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | A tool error response (`isError=true`) increments only the tool error counter | Unit test |
| VP-002 | A JSON-RPC error response increments only the protocol error counter | Unit test |
| VP-003 | `protocol_error_count + tool_error_count ≤ total_request_count` for any window | Invariant test |
| VP-004 | Error rate is always in [0.0, 1.0] | Property test |
| VP-005 | Time-series data retains exactly N minutes of history (no unbounded growth) | Unit test |
| VP-006 | Zero-request windows produce error rate 0.0, not NaN | Edge case test |

## Traceability

| Source | Target |
|--------|--------|
| CAP-013 | This contract |
| DI-020 | Invariant (separate error categories) |
| DEC-023 | Edge case (tool error ≠ protocol error) |
| BC-6.13.001 | Passive metric collection (shared observation layer) |
| BC-6.14.001 | Alerting thresholds (error rate is an alertable metric) |
| BC-6.15.001 | TUI visualization (error rate sparkline) |
