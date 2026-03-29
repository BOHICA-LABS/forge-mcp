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
capability: "CAP-014"
lifecycle_status: active
introduced: v0.1.0
---

# BC-6.14.002 — Alert State Machine (Normal → Breached → Recovered)

## Summary

Each monitored metric per server has an alert state that follows a three-state
machine: **normal**, **breached**, **recovered**. Transitions emit structured
events. Duplicate breach alerts are suppressed. A hysteresis mechanism prevents
rapid oscillation between breached and recovered states.

## State Machine

```
                threshold exceeded
    NORMAL ─────────────────────────► BREACHED
       ▲                                  │
       │                                  │ metric below threshold
       │  hysteresis timer expires        │ for N seconds
       │                                  ▼
       └──────────────────────────── RECOVERED
```

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | Alerting thresholds are configured (BC-6.14.001) |
| PRE-002 | Metrics are being collected (BC-6.13.001, BC-6.13.002) |
| PRE-003 | The alert evaluator runs on a periodic cycle (default: every 5 seconds) |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | When a metric exceeds its threshold, the state transitions from `normal` → `breached` |
| POST-002 | A `E-MON-001` (threshold breached) event is emitted on the first breach |
| POST-003 | While in `breached` state, no additional breach events are emitted for the same metric (DI-009) |
| POST-004 | When the metric drops below threshold, the state transitions from `breached` → `recovered` |
| POST-005 | A `E-MON-002` (threshold recovered) event is emitted on recovery |
| POST-006 | The metric must remain below threshold for the hysteresis duration (default: 30s) before transitioning from `recovered` → `normal` |
| POST-007 | If the metric re-exceeds threshold during hysteresis, transition back to `breached` without passing through `normal` |
| POST-008 | All state transitions are logged with: server, metric, old_state, new_state, value, threshold, timestamp |

## Invariants

| ID | Invariant |
|----|-----------|
| DI-009 | No duplicate breach alerts: a breach event is emitted at most once per breach episode |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Metric oscillates above/below threshold rapidly | Hysteresis prevents rapid normal↔breached flapping; state stays breached until hysteresis completes |
| EC-002 | Server disconnects while in breached state | State remains `breached`; connection status alert handled separately |
| EC-003 | Threshold changed while in breached state | Re-evaluate immediately; if new threshold is not exceeded, transition to recovered |
| EC-004 | Multiple metrics breach simultaneously | Each metric has independent state machine; multiple E-MON-001 events emitted |
| EC-005 | Metric exactly equals threshold | Not breached (strictly greater than for latency/error, strictly less than for throughput) |
| EC-006 | First evaluation after startup with metric already above threshold | Transition directly to `breached`; emit E-MON-001 |
| EC-007 | Hysteresis timer expires while metric is exactly at threshold | Transition to `normal` (threshold is the boundary, not a breach) |

## Event Schema

### E-MON-001 — Threshold Breached

```json
{
  "event": "E-MON-001",
  "server": "server-name",
  "metric": "latency_p99",
  "value": 6200,
  "threshold": 5000,
  "state": "breached",
  "timestamp": "2026-03-29T11:25:00Z"
}
```

### E-MON-002 — Threshold Recovered

```json
{
  "event": "E-MON-002",
  "server": "server-name",
  "metric": "latency_p99",
  "value": 3100,
  "threshold": 5000,
  "state": "recovered",
  "timestamp": "2026-03-29T11:26:30Z"
}
```

## Canonical Test Vectors

### Happy Path

| Time | Metric Value | Threshold | State Before | Event | State After |
|------|-------------|-----------|--------------|-------|-------------|
| t=0 | 3000ms | 5000ms | normal | — | normal |
| t=5 | 6000ms | 5000ms | normal | E-MON-001 | breached |
| t=10 | 7000ms | 5000ms | breached | — (no dup) | breached |
| t=15 | 4000ms | 5000ms | breached | E-MON-002 | recovered |
| t=45 | 4000ms | 5000ms | recovered | — (hysteresis complete) | normal |

### Edge Case — Hysteresis Interrupted

| Time | Metric Value | Threshold | State Before | Event | State After |
|------|-------------|-----------|--------------|-------|-------------|
| t=0 | 6000ms | 5000ms | normal | E-MON-001 | breached |
| t=5 | 4000ms | 5000ms | breached | E-MON-002 | recovered |
| t=15 | 6000ms | 5000ms | recovered | E-MON-001 | breached |

### Error

| Scenario | Expected Behavior |
|----------|-------------------|
| Metric collection fails (no data) | State unchanged; no transition; warning logged |
| NaN metric value | Treated as collection failure; no transition |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | E-MON-001 is emitted exactly once per breach episode | State machine test |
| VP-002 | No transition from `recovered` → `normal` before hysteresis duration | Timer test |
| VP-003 | Each metric × server combination has an independent state machine | Unit test |
| VP-004 | All state transitions produce a log entry with required fields | Integration test |
| VP-005 | Threshold change triggers immediate re-evaluation | Unit test |

## Traceability

| Source | Target |
|--------|--------|
| CAP-014 | This contract |
| DI-009 | Invariant (no duplicate breach alerts) |
| E-MON-001 | Event (threshold breached) |
| E-MON-002 | Event (threshold recovered) |
| BC-6.14.001 | Configurable thresholds (input to this state machine) |
| BC-6.13.001 | Metric collection (provides metric values) |
