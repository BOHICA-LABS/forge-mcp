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
subsystem: "TUI Dashboard"
capability: "CAP-008"
lifecycle_status: active
introduced: v0.1.0
---

# BC-3.08.002 — Sparkline and Histogram Health Metric Visualization

## Summary

The health metrics pane renders real-time latency and throughput data using sparkline widgets for time-series visualization and a histogram for latency distribution. Sparklines auto-scale their y-axis to fit the data range. All visualizations update at 1Hz frequency. Visualization uses ratatui's Sparkline and BarChart widgets.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | Health metrics pane is visible in the layout (BC-3.06.001) |
| PRE-002 | At least one server connection is active with timing data available (BC-4.09.002) |
| PRE-003 | Timing subsystem is producing latency and throughput metrics |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Latency sparkline shows the last N data points (N = pane width in characters) |
| POST-002 | Throughput sparkline shows messages/second over the same time window |
| POST-003 | Histogram shows latency distribution in configurable bucket ranges |
| POST-004 | Y-axis auto-scales: maximum y = max(data points) × 1.1 (10% headroom) |
| POST-005 | Each sparkline has a label showing the current value and min/max/avg |
| POST-006 | Visualizations update at 1Hz (±100ms tolerance) |
| POST-007 | When no data is available for a metric, the sparkline shows a flat line at zero with "No data" label |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | Sparkline data points correspond to real captured metrics (never synthesized or interpolated) |
| INV-002 | Y-axis minimum is always 0 (latency and throughput cannot be negative) |
| INV-003 | Update frequency does not exceed 1Hz regardless of data arrival rate |
| INV-004 | Visualization rendering completes within the frame budget (≤16ms) |

## Edge Cases

| ID | Scenario | Expected Behavior | Source |
|----|----------|-------------------|--------|
| EC-001 | Only one data point available | Sparkline shows single bar; histogram shows single bucket | — |
| EC-002 | All latency values are identical (e.g., 50ms) | Sparkline shows flat line; histogram shows single tall bar | — |
| EC-003 | Latency spike (100x normal) | Y-axis rescales to accommodate spike; previous data appears compressed | — |
| EC-004 | No metrics for >30 seconds | Show "Stale data (last update: {timestamp})" warning | — |
| EC-005 | Pane resized to very narrow (< 10 chars wide) | Show abbreviated metric: current value only, no sparkline | — |
| EC-006 | Multiple servers connected | Show metrics for the currently selected server; label identifies which server | — |
| EC-007 | Throughput is 0 msgs/sec (idle connection) | Sparkline shows zero values; label shows "0 msg/s" | — |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-HP-001 | 60 latency samples: [10, 12, 11, 15, 10, ...] ms | Sparkline with 60 bars, y-axis scaled to ~16.5ms, label "Latency: 11ms (avg: 11.6ms, min: 10ms, max: 15ms)" |
| TV-HP-002 | Throughput: [100, 120, 95, 110] msgs/sec | Sparkline with 4 bars, label "Throughput: 110 msg/s" |
| TV-HP-003 | Latency distribution: 80% < 20ms, 15% 20-50ms, 5% > 50ms | Three-bucket histogram with bars proportional to percentages |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-EC-001 | Single data point: latency = 25ms | One-bar sparkline, histogram with one bucket |
| TV-EC-002 | Latency spike: [..., 10, 10, 10, 1000, 10, 10] | Y-axis rescales to 1100ms; spike clearly visible; previous values appear near bottom |
| TV-EC-003 | Health metrics pane width = 8 characters | Show "12ms" current value only; no sparkline |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-ERR-001 | Metrics source returns error | Show "Metrics unavailable" in pane; retry on next 1Hz tick |
| TV-ERR-002 | NaN in metrics data | Skip NaN values; do not render them in sparkline; log warning |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | For all data sets: sparkline y-axis max ≥ max(data) | Property-based test |
| VP-002 | Update frequency measured over 60 seconds is 1Hz ±10% | Timing test |
| VP-003 | Sparkline rendering with N=1000 data points completes within frame budget | Performance test |
| VP-004 | Histogram bucket counts sum to total number of data points | Invariant test |

## Traceability

| Item | Reference |
|------|-----------|
| L2 Capability | CAP-008 (TUI Data Display) |
| Related BCs | BC-4.09.002 (timing data source), BC-3.06.001 (pane layout), BC-3.08.005 (accessibility) |
