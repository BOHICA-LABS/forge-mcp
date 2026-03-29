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

# BC-6.15.001 — Time-Series Metric Visualization in TUI

## Summary

The TUI dashboard displays real-time time-series visualizations of health metrics
using sparklines and histograms. Metrics include latency (p50/p99), throughput,
and error rate. The display auto-scales and refreshes at 1Hz.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | The TUI is active (`forge-mcp tui`) |
| PRE-002 | At least one server is connected and being monitored |
| PRE-003 | The terminal supports Unicode block characters (for sparklines) |
| PRE-004 | Metric data is available from BC-6.13.001 and BC-6.13.002 |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Sparklines are rendered for: latency p50, latency p99, throughput (msg/s), error rate |
| POST-002 | A latency distribution histogram is available (bucket view of latency values) |
| POST-003 | The y-axis auto-scales to fit the current data range with 10% headroom |
| POST-004 | The display refreshes at 1Hz (±100ms jitter acceptable) |
| POST-005 | Time axis labels show relative time (e.g., "30s ago", "1m ago", "5m ago") |
| POST-006 | Each server's metrics are displayed in a separate panel or tab |
| POST-007 | Alert state (normal/breached/recovered) is visually indicated (color or icon) |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | TUI rendering does not block metric collection or alert evaluation |
| INV-002 | Sparkline data points correspond 1:1 to metric sampling intervals |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Terminal too narrow for sparkline | Truncate oldest data points; show what fits |
| EC-002 | No metric data yet (just connected) | Show empty sparkline with "waiting for data" indicator |
| EC-003 | Metric value is 0 for entire window | Sparkline is flat at bottom; y-axis shows 0 |
| EC-004 | Extreme outlier (e.g., 100x normal latency) | Auto-scale accommodates; sparkline compresses normal values |
| EC-005 | Terminal resized during display | Re-render sparklines to fit new width on next refresh |
| EC-006 | Server disconnects while TUI is showing its metrics | Show "disconnected" status; freeze last-known sparkline data |
| EC-007 | 20+ servers connected | Scrollable list or paginated view; no overflow |
| EC-008 | Non-Unicode terminal (no block characters) | Fall back to ASCII art (`#`, `-`, `|`) for sparklines |

## Canonical Test Vectors

### Happy Path

| Scenario | Expected Display |
|----------|-----------------|
| Server with steady 50ms latency for 2 minutes | Flat sparkline at 50ms; y-axis range ~0-60ms |
| Server with latency spike from 50ms to 500ms | Sparkline shows spike; y-axis auto-scales to ~0-550ms |
| Throughput of 10 msg/s steady | Flat sparkline at 10; y-axis range ~0-12 |
| Error rate climbing from 0% to 15% | Rising sparkline; breached alert indicator if threshold exceeded |

### Edge Case

| Scenario | Expected Display |
|----------|-----------------|
| Terminal width = 40 columns | Sparkline shows ~30 data points (width minus margins) |
| All metrics at zero | Flat sparklines at bottom; "No activity" hint |
| Server disconnected | "Disconnected" badge; sparkline data frozen at last values |

### Error

| Scenario | Expected Behavior |
|----------|-------------------|
| Metric collection fails | Sparkline shows gap or "no data" marker for missing interval |
| Terminal output error | TUI exits gracefully with diagnostic on stderr |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | Refresh rate is 1Hz ± 100ms measured over 60 seconds | Timing test |
| VP-002 | Y-axis range encompasses all visible data points | Unit test |
| VP-003 | Sparkline data points match metric sampling count | Unit test |
| VP-004 | Terminal resize triggers re-render within 1 refresh cycle | Integration test |
| VP-005 | TUI rendering does not increase metric collection latency | Performance test |

## Traceability

| Source | Target |
|--------|--------|
| CAP-015 | This contract |
| BC-6.13.001 | Metric collection (data source for sparklines) |
| BC-6.13.002 | Error rate tracking (data source for error sparkline) |
| BC-6.14.002 | Alert state machine (visual alert indicators) |
| BC-6.15.002 | Metric export (CLI alternative to visual display) |
