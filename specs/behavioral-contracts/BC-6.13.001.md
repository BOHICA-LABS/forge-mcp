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

# BC-6.13.001 — Passive Latency and Throughput Metric Collection

## Summary

Health monitoring derives latency, throughput, and connection status metrics by
passively observing MCP protocol traffic between the client and connected servers.
No synthetic probes are injected. The monitoring overhead must not impact server
latency by more than 1%.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | At least one MCP server connection is active |
| PRE-002 | The monitoring subsystem is initialized and observing the transport layer |
| PRE-003 | System clock is available for timestamp generation |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Latency is recorded as the duration from JSON-RPC request send to response receive, per method |
| POST-002 | Throughput is calculated as messages per second over a configurable sliding window (default: 60s) |
| POST-003 | Connection status is one of: `connected`, `disconnected`, `error` — with a timestamp for the last state change |
| POST-004 | Latency percentiles (p50, p95, p99) are available for the current window |
| POST-005 | Metrics are keyed per server (multi-server metrics are independent) |
| POST-006 | Metric collection adds ≤ 1% latency overhead to observed requests |

## Invariants

| ID | Invariant |
|----|-----------|
| DI-008 | Monitoring overhead must not impact server latency by > 1% |
| NFR-004 | Monitoring latency overhead ≤ 1% of baseline |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Idle server with no traffic for > 5 minutes (ASM-009) | Metrics show stale data with `last_updated` timestamp; throughput drops to 0; latency retains last-known values with staleness indicator |
| EC-002 | Server disconnects mid-request | Record the in-flight request as a timeout; transition connection status to `disconnected` |
| EC-003 | Burst of 1000 requests in 1 second | Sliding window handles burst without overflow; throughput spike correctly reflected |
| EC-004 | Server reconnects after disconnect | Connection status transitions to `connected`; metrics resume from fresh baseline |
| EC-005 | Multiple concurrent requests to same server | Each request tracked independently; latency is per-request, not averaged across concurrent set |
| EC-006 | Notification messages (no response expected) | Counted for throughput but not for latency (no response to measure) |
| EC-007 | Clock skew or NTP adjustment during measurement | Latency uses monotonic clock, not wall clock; immune to NTP jumps |
| EC-008 | Zero-traffic server just connected | Metrics initialized with zero throughput, no latency data, status `connected` |

## Canonical Test Vectors

### Happy Path

| Scenario | Input Traffic | Expected Metrics |
|----------|--------------|------------------|
| 10 tool calls, each 50ms latency | 10 requests + 10 responses over 5s | latency_p50=50ms, latency_p99=50ms, throughput=2.0 msg/s, status=connected |
| Mixed latencies: 5×10ms, 5×100ms | 10 req/resp pairs | latency_p50≈50ms, latency_p99≈100ms |
| Idle after burst | 100 msg in 1s, then 60s silence | throughput decays from 100 to 0 over window |

### Edge Case

| Scenario | Expected Metrics |
|----------|------------------|
| No traffic for 10 minutes | throughput=0, latency=(last known), status=connected, last_updated=(10 min ago) |
| Server disconnects | status=disconnected, timestamp=(now) |
| Reconnection after 30s | status=connected, metrics reset |

### Error

| Scenario | Expected Behavior |
|----------|-------------------|
| Transport layer returns error on send | Connection status → `error`, error logged, in-flight request marked as failed |
| Response JSON is malformed | Counted as protocol error (see BC-6.13.002), not latency measurement |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | Monitoring adds ≤ 1% latency overhead (measured via benchmark) | Performance test |
| VP-002 | Throughput calculation matches actual message count / window size | Unit test |
| VP-003 | Latency percentiles are mathematically correct for known input distributions | Property test |
| VP-004 | Connection status transitions are: connected↔disconnected, connected↔error, error→connected | State machine test |
| VP-005 | Monotonic clock is used for latency (not wall clock) | Unit test |

## Traceability

| Source | Target |
|--------|--------|
| CAP-013 | This contract |
| DI-008 | Invariant (monitoring overhead) |
| NFR-004 | Non-functional requirement (latency overhead) |
| ASM-009 | Edge case (idle server stale metrics) |
| BC-6.13.002 | Error rate tracking (uses same observation layer) |
| BC-6.14.001 | Alerting thresholds (consumes these metrics) |
| BC-6.15.001 | TUI visualization (visualizes these metrics) |
