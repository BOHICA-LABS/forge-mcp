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
capability: "CAP-015"
lifecycle_status: active
introduced: v0.1.0
---

# BC-6.15.002 — Metric Snapshot JSON Export via CLI

## Summary

The CLI command `forge-mcp info <server> --metrics --json` exports a point-in-time
snapshot of all health metrics for a server as structured JSON. This is not a
streaming interface — it captures the current state and exits.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | The specified server is configured in forge-mcp |
| PRE-002 | The monitoring subsystem has collected at least one data point |
| PRE-003 | The `--metrics` and `--json` flags are both present |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | A single JSON object is emitted on stdout containing the metric snapshot |
| POST-002 | The snapshot includes: latency histogram, error rates (protocol + tool), throughput, alert states |
| POST-003 | The snapshot includes a `timestamp` field indicating when the snapshot was taken |
| POST-004 | The JSON conforms to the metric snapshot schema defined in interface-definitions |
| POST-005 | The command exits immediately after emitting the snapshot (no streaming) |
| POST-006 | If no metric data is available yet, emit a snapshot with zero/empty values and `data_available: false` |

## Invariants

| ID | Invariant |
|----|-----------|
| DI-013 | JSON data output goes to stdout; human-readable diagnostics go to stderr |

## JSON Schema

```json
{
  "server": "server-name",
  "timestamp": "2026-03-29T11:25:00Z",
  "data_available": true,
  "latency": {
    "p50_ms": 42,
    "p95_ms": 120,
    "p99_ms": 250,
    "histogram": [
      {"bucket_ms": 10, "count": 5},
      {"bucket_ms": 50, "count": 45},
      {"bucket_ms": 100, "count": 30},
      {"bucket_ms": 500, "count": 15},
      {"bucket_ms": 1000, "count": 5}
    ]
  },
  "error_rates": {
    "protocol": 0.02,
    "tool": 0.05,
    "combined": 0.07,
    "window_seconds": 60
  },
  "throughput": {
    "current_msg_per_sec": 12.5,
    "window_seconds": 60
  },
  "alerts": {
    "latency": {"state": "normal", "threshold_ms": 5000},
    "error_rate": {"state": "breached", "threshold": 0.10, "since": "2026-03-29T11:20:00Z"},
    "throughput": {"state": "normal", "threshold_min": 0}
  },
  "connection": {
    "status": "connected",
    "since": "2026-03-29T10:00:00Z",
    "transport": "stdio"
  }
}
```

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Server exists in config but never connected | `data_available: false`, zero values, connection status `disconnected` |
| EC-002 | Server was connected but is now disconnected | Snapshot shows last-known metrics with stale `timestamp` and `connection.status: "disconnected"` |
| EC-003 | `--metrics` without `--json` | Human-readable table on stdout (not JSON) |
| EC-004 | `--metrics --json` for non-existent server | Error JSON on stdout, exit 2 |
| EC-005 | Very high metric cardinality (many histogram buckets) | Histogram buckets are pre-aggregated; bounded to ≤ 20 buckets |
| EC-006 | Concurrent metric update during snapshot | Snapshot is consistent (taken under read lock or copy-on-read) |

## Canonical Test Vectors

### Happy Path

| Input | Expected stdout | Exit Code |
|-------|----------------|-----------|
| `forge-mcp info srv --metrics --json` | Valid JSON matching schema above | 0 |
| `forge-mcp info srv --metrics --json \| jq .latency.p99_ms` | Integer (e.g., `250`) | 0 |
| `forge-mcp info srv --metrics --json \| jq .alerts.error_rate.state` | `"breached"` or `"normal"` | 0 |

### Edge Case

| Input | Expected stdout | Exit Code |
|-------|----------------|-----------|
| `forge-mcp info new-srv --metrics --json` (never connected) | `{"server":"new-srv","data_available":false,...}` | 0 |
| `forge-mcp info srv --metrics` (no --json) | Human-readable table | 0 |

### Error

| Input | Expected stdout | Exit Code |
|-------|----------------|-----------|
| `forge-mcp info nonexistent --metrics --json` | `{"error":"server_not_found",...}` | 2 |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | Output JSON validates against the metric snapshot schema | Schema validation test |
| VP-002 | Snapshot is point-in-time (no streaming, command exits after emit) | Integration test |
| VP-003 | `data_available` is false when no metrics have been collected | Unit test |
| VP-004 | Histogram bucket count is ≤ 20 | Unit test |
| VP-005 | Snapshot is consistent (no partial updates visible) | Concurrency test |

## Traceability

| Source | Target |
|--------|--------|
| CAP-015 | This contract |
| DI-013 | Invariant (JSON stdout) |
| BC-6.13.001 | Metric collection (data source) |
| BC-6.13.002 | Error rate tracking (data source) |
| BC-6.14.002 | Alert state machine (alert states in snapshot) |
| BC-6.15.001 | TUI visualization (alternative view of same data) |
