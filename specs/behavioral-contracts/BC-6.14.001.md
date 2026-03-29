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

# BC-6.14.001 — Configurable Alerting Thresholds

## Summary

Users configure alerting thresholds for monitored metrics. Thresholds can be set
globally (applying to all servers) or per-server (overriding global). Configuration
is loaded from the config file's `[monitoring]` section and can be overridden at
runtime via CLI flags.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | A valid configuration file exists (or defaults are used) |
| PRE-002 | The monitoring subsystem is active and collecting metrics |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Thresholds are configurable for: `latency_ms` (p99), `error_rate` (0.0–1.0), `throughput_min` (msg/sec) |
| POST-002 | Global thresholds apply to all servers unless a per-server override exists |
| POST-003 | Per-server thresholds override global for that server only |
| POST-004 | CLI flags (`--alert-latency`, `--alert-error-rate`, `--alert-throughput`) override config file values |
| POST-005 | Override priority: CLI flags > per-server config > global config > built-in defaults |
| POST-006 | Built-in defaults: latency_ms=5000, error_rate=0.1, throughput_min=0 (disabled) |
| POST-007 | Threshold changes take effect immediately (no restart required for config reload) |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | Threshold values are validated: latency_ms > 0, error_rate in [0.0, 1.0], throughput_min ≥ 0 |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | No config file, no CLI flags | Built-in defaults used |
| EC-002 | Global threshold set, no per-server | Global applies to all servers |
| EC-003 | Per-server threshold set for one server | Override for that server, global for others |
| EC-004 | CLI flag overrides per-server config | CLI wins (highest priority) |
| EC-005 | Invalid threshold value in config (e.g., error_rate=2.0) | Config error: report on stderr, exit 3; do not silently clamp |
| EC-006 | Threshold set to 0 for latency_ms | Config error: latency must be > 0 |
| EC-007 | Throughput threshold set to 0 | Valid: effectively disables throughput alerting |
| EC-008 | Config file reloaded while monitoring is active | New thresholds take effect on next evaluation cycle |

## Configuration Schema

```toml
[monitoring]
latency_ms = 5000        # p99 latency threshold in milliseconds
error_rate = 0.10        # error rate threshold (0.0 to 1.0)
throughput_min = 0       # minimum msg/sec (0 = disabled)

[monitoring.servers.my-server]
latency_ms = 2000        # per-server override
error_rate = 0.05
```

## Canonical Test Vectors

### Happy Path

| Config | CLI Flags | Server | Effective Thresholds |
|--------|-----------|--------|---------------------|
| global: latency=5000 | (none) | any | latency=5000, error=0.10, throughput=0 |
| global: latency=5000, srv1: latency=2000 | (none) | srv1 | latency=2000, error=0.10, throughput=0 |
| global: latency=5000, srv1: latency=2000 | (none) | srv2 | latency=5000, error=0.10, throughput=0 |
| global: latency=5000 | --alert-latency 1000 | any | latency=1000, error=0.10, throughput=0 |

### Edge Case

| Config | Expected Behavior |
|--------|-------------------|
| (no config file) | Defaults: latency=5000, error=0.10, throughput=0 |
| error_rate = -0.5 | Config error, exit 3 |
| error_rate = 1.5 | Config error, exit 3 |
| latency_ms = 0 | Config error, exit 3 |

### Error

| Scenario | Expected Behavior | Exit Code |
|----------|-------------------|-----------|
| Config file is invalid TOML | Parse error on stderr | 3 |
| Unknown key in `[monitoring]` section | Warning on stderr (not error); continue with known keys | 0 |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | Override priority: CLI > per-server > global > default | Unit test |
| VP-002 | Invalid threshold values are rejected at config load | Unit test |
| VP-003 | Each metric type (latency, error_rate, throughput) is independently configurable | Unit test |
| VP-004 | Per-server overrides do not affect other servers | Unit test |

## Traceability

| Source | Target |
|--------|--------|
| CAP-014 | This contract |
| BC-6.13.001 | Metric collection (these thresholds evaluate those metrics) |
| BC-6.13.002 | Error rate tracking (error_rate threshold) |
| BC-6.14.002 | Alert state machine (thresholds trigger state transitions) |
