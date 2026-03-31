# STORY-033 Demo Evidence Report

Generated: 2026-03-31

## Summary

This directory contains demo recordings and evidence artifacts for STORY-033
(forge-health metrics subsystem). Files are organized by Acceptance Criterion (AC),
Edge Case (EC), and Verification Property (VP).

---

## Acceptance Criteria Coverage

### AC-001 — Latency Histogram

Demonstrates that the latency histogram correctly records and reports
per-request latency distributions.

| File | Type | Description |
|------|------|-------------|
| `AC-001-latency-histogram.tape` | VHS tape | Demo script source |
| `AC-001-latency-histogram.webm` | Video | Full-resolution recording |
| `AC-001-latency-histogram.gif` | GIF | Animated preview |

### AC-002 — Throughput Counter

Demonstrates that the throughput counter correctly accumulates message counts
and resets on demand.

| File | Type | Description |
|------|------|-------------|
| `AC-002-throughput-counter.tape` | VHS tape | Demo script source |
| `AC-002-throughput-counter.webm` | Video | Full-resolution recording |
| `AC-002-throughput-counter.gif` | GIF | Animated preview |

### AC-004 — Metric Snapshot

Demonstrates that `MetricSnapshot` captures a point-in-time consistent view
of all registered metrics.

| File | Type | Description |
|------|------|-------------|
| `AC-004-metric-snapshot.tape` | VHS tape | Demo script source |
| `AC-004-metric-snapshot.webm` | Video | Full-resolution recording |
| `AC-004-metric-snapshot.gif` | GIF | Animated preview |

---

## Edge Cases

### EC-001 — Zero Messages

Demonstrates correct behavior when no messages have been processed
(histogram and counter both return zero-state snapshots).

| File | Type | Description |
|------|------|-------------|
| `EC-001-zero-messages.tape` | VHS tape | Demo script source |
| `EC-001-zero-messages.webm` | Video | Full-resolution recording |
| `EC-001-zero-messages.gif` | GIF | Animated preview |

### EC-002 — Single Message

Demonstrates correct behavior with exactly one message processed
(boundary condition for statistical calculations).

| File | Type | Description |
|------|------|-------------|
| `EC-002-single-message.tape` | VHS tape | Demo script source |
| `EC-002-single-message.webm` | Video | Full-resolution recording |
| `EC-002-single-message.gif` | GIF | Animated preview |

### EC-003 — High Latency

Demonstrates that extremely high latency values are recorded without overflow
or precision loss.

| File | Type | Description |
|------|------|-------------|
| `EC-003-high-latency.tape` | VHS tape | Demo script source |

### EC-004 — Reset Clears

Demonstrates that a metric reset fully clears accumulated state, with
subsequent observations starting from a clean baseline.

| File | Type | Description |
|------|------|-------------|
| `EC-004-reset-clears.tape` | VHS tape | Demo script source |

---

## Verification Properties

### VP-008 — Proptest Round-Trip

Property-based test demonstrating that metric values survive a
serialize → deserialize round-trip without loss.

| File | Type | Description |
|------|------|-------------|
| `VP-008-proptest.tape` | VHS tape | Demo script source |

---

## Auxiliary / Scaffolding

| File | Notes |
|------|-------|
| `test-simple.tape` | Initial scaffolding tape used during development |
| `test-sleep.tape` | Timing/sleep behavior verification |
| `test-sleep.webm` | Video recording of sleep test |
| `test-sleep.gif` | GIF preview of sleep test |

---

## Proptest Regressions

Regression seeds for property-based tests are stored alongside the test module:

```
crates/forge-health/tests/metric_tests.proptest-regressions
```

These seeds are committed so that previously-found failures are always replayed
on future runs, preventing regression.
