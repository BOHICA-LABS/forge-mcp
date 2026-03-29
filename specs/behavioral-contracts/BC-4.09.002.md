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

# BC-4.09.002 — Per-Message Timing and Throughput Analysis

## Summary

The traffic inspection subsystem computes timing metrics from captured message timestamps: request-response latency (time between a request and its corresponding response by matching JSON-RPC `id`), inter-message gap (time between consecutive messages), and throughput (messages per second). All metrics are derived passively from captured message timestamps with no active probing. The analysis must not add more than 1% latency overhead (DI-008).

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | Message capture is active and producing timestamped messages (BC-4.09.001) |
| PRE-002 | At least two captured messages exist (minimum for any timing computation) |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Request-response latency is computed for every request/response pair matched by JSON-RPC `id` |
| POST-002 | Inter-message gap is computed for every pair of consecutive captured messages |
| POST-003 | Throughput (messages/second) is computed over a sliding window (default: 1 second) |
| POST-004 | Metrics are available to the TUI health metrics pane (BC-3.08.002) within 1 second of computation |
| POST-005 | Unmatched requests (no response received) are tracked with a "pending" status and a running timer |
| POST-006 | Metrics are computed per-server (each server connection has independent metrics) |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | DI-008: Timing analysis adds < 1% latency overhead to message forwarding |
| INV-002 | Latency values are always non-negative (response timestamp ≥ request timestamp) |
| INV-003 | Throughput values are always non-negative |
| INV-004 | Timing analysis is passive observation only — no synthetic messages are injected |
| INV-005 | Request-response matching uses JSON-RPC `id` field exclusively |

## Edge Cases

| ID | Scenario | Expected Behavior | Source |
|----|----------|-------------------|--------|
| EC-001 | Request sent but no response received within 30 seconds | Mark as "timed out" in metrics; latency recorded as ∞ (or sentinel value) | — |
| EC-002 | Response received for unknown request (id not in pending map) | Log warning "Orphan response for id={id}"; do not compute latency | — |
| EC-003 | Notification messages (no id field) | Skip request-response matching; count in throughput and inter-message gap only | — |
| EC-004 | Multiple requests with same id (protocol violation) | Match response to most recent request with that id; log warning | — |
| EC-005 | Zero messages in last second | Throughput = 0; no gap computed | — |
| EC-006 | Clock skew between capture timestamps | Detect negative latency (response before request); report as clock skew warning | — |
| EC-007 | Batch request/response (array) | Compute individual latencies for each matched id within the batch | — |
| EC-008 | Very high throughput (>5000 msg/sec) | Metrics computation must not fall behind; aggregate if necessary | — |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-HP-001 | Request at T=0ms, Response at T=50ms, same id=1 | Latency: 50ms |
| TV-HP-002 | Messages at T=0, T=100, T=250, T=300 ms | Inter-message gaps: 100ms, 150ms, 50ms |
| TV-HP-003 | 100 messages captured in 1-second window | Throughput: 100 msg/s |
| TV-HP-004 | Two servers: srv-A (latency ~10ms), srv-B (latency ~200ms) | Independent metrics: srv-A avg=10ms, srv-B avg=200ms |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-EC-001 | Request id=5, no response after 30s | Pending request marked "timed out"; latency = timeout sentinel |
| TV-EC-002 | Response id=99 with no matching request | Warning logged; response counted in throughput but no latency computed |
| TV-EC-003 | 3 notifications in sequence | No latency; 3 counted in throughput; 2 inter-message gaps computed |
| TV-EC-004 | Batch: `[{id:1},{id:2}]` request, `[{id:1},{id:2}]` response | Two individual latencies computed |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-ERR-001 | Captured message with unparseable id field | Skip request-response matching for this message; count in throughput |
| TV-ERR-002 | Negative computed latency (clock issue) | Report 0ms latency with clock skew warning; do not report negative value |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | For all matched request-response pairs: latency ≥ 0 | Invariant test |
| VP-002 | DI-008 compliance: analysis overhead < 1% of forwarding latency | Benchmark test |
| VP-003 | Throughput computed over sliding window equals message_count / window_duration | Property-based test |
| VP-004 | Request-response matching by id is correct for all id types (integer, string, null) | Property-based test |

## Traceability

| Item | Reference |
|------|-----------|
| L2 Capability | CAP-009 (Message Capture & Analysis) |
| Domain Invariant | DI-008 (< 1% latency impact) |
| Related BCs | BC-4.09.001 (message capture — data source), BC-3.08.002 (sparkline visualization — data consumer) |
