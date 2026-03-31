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

---

# Demo Evidence Report — STORY-029

**Story:** STORY-029 — Capture Buffer Management with Bounded Memory  
**Epic:** EPIC-04  
**Behavioral Contract:** BC-4.09.003  
**Verification Property:** VP-005  
**Recorded:** 2026-03-30  
**Recorder:** demo-recorder agent  
**Product Type:** Rust library (forge-traffic crate)  
**Recording Tool:** VHS 0.10.0  

---

## Coverage Summary

| Item | Type | Tests Covered | .gif | .webm | Status |
|------|------|---------------|------|-------|--------|
| AC-001 | Acceptance Criterion | `test_BC_4_09_003_ring_buffer_stores_messages` | ✅ | ✅ | PASS |
| AC-002 | Acceptance Criterion | `test_BC_4_09_003_fifo_eviction` | ✅ | ✅ | PASS |
| AC-003 | Acceptance Criterion | `test_BC_4_09_003_memory_bound` | ✅ | ✅ | PASS |
| AC-004 | Acceptance Criterion | `test_BC_4_09_003_eviction_warning_emitted` | ✅ | ✅ | PASS |
| AC-005 | Acceptance Criterion | `test_BC_4_09_003_append_is_constant_time` | ✅ | ✅ | PASS |
| EC-001 | Edge Case | `test_zero_capacity_buffer` | ✅ | ✅ | PASS |
| EC-002 | Edge Case | `test_empty_buffer_operations` | ✅ | ✅ | PASS |

**Total:** 7/7 items recorded · 14/14 output files generated · 7/7 tests passing

---

## AC-001 — Ring Buffer Stores Messages

**Traces to:** BC-4.09.003 postcondition  
**Test:** `test_BC_4_09_003_ring_buffer_stores_messages`  
**Command:** `cargo test -p forge-traffic --test buffer_tests test_BC_4_09_003_ring_buffer_stores_messages -- --nocapture`

**What it demonstrates:**
- `RingBuffer::new(10, usize::MAX)` creates a buffer with capacity 10
- 5 pushes produce no evictions (`push()` returns `None`)
- `len()` == 5 after 5 pushes
- `iter()` yields all 5 items in insertion order (oldest → newest): `[1, 2, 3, 4, 5]`

**Recordings:**
- ![AC-001 GIF](AC-001-ring-buffer-stores-messages.gif)
- [AC-001 WebM](AC-001-ring-buffer-stores-messages.webm)
- [AC-001 Tape](AC-001-ring-buffer-stores-messages.tape)

---

## AC-002 — FIFO Eviction

**Traces to:** BC-4.09.003 postcondition, VP-005  
**Test:** `test_BC_4_09_003_fifo_eviction`  
**Command:** `cargo test -p forge-traffic --test buffer_tests test_BC_4_09_003_fifo_eviction -- --nocapture`

**What it demonstrates:**
- Buffer of capacity 3 fills with `[A, B, C]`
- `push("D")` evicts `A` (oldest) and returns `Some("A")` — strict FIFO
- `len()` remains 3 after eviction
- `iter()` yields `[B, C, D]` in order

**Recordings:**
- ![AC-002 GIF](AC-002-fifo-eviction.gif)
- [AC-002 WebM](AC-002-fifo-eviction.webm)
- [AC-002 Tape](AC-002-fifo-eviction.tape)

---

## AC-003 — Memory Bound

**Traces to:** BC-4.09.003 invariant, NFR-012  
**Test:** `test_BC_4_09_003_memory_bound`  
**Command:** `cargo test -p forge-traffic --test buffer_tests test_BC_4_09_003_memory_bound -- --nocapture`

**What it demonstrates:**
- `RingBuffer::new(usize::MAX, 1000)` creates a memory-bounded buffer (1000 bytes max)
- 200 `String` pushes: `memory_usage_bytes()` ≤ 1000 after every push
- Eviction triggers automatically when byte limit reached, not just count limit
- No assertion panics → invariant upheld throughout

**Recordings:**
- ![AC-003 GIF](AC-003-memory-bound.gif)
- [AC-003 WebM](AC-003-memory-bound.webm)
- [AC-003 Tape](AC-003-memory-bound.tape)

---

## AC-004 — Eviction Warning Threshold

**Traces to:** BC-4.09.003, E-CAP-001  
**Test:** `test_BC_4_09_003_eviction_warning_emitted`  
**Command:** `cargo test -p forge-traffic --test buffer_tests test_BC_4_09_003_eviction_warning_emitted -- --nocapture`

**What it demonstrates:**
- Buffer of capacity 10; 9 pushes → `len() >= 90%` threshold (9 ≥ 9)
- `capacity()` returns the configured value (10)
- Overflow pushes (items 9–19) keep `len() ≤ capacity` at all times
- No panics during overflow — eviction path is stable

**Recordings:**
- ![AC-004 GIF](AC-004-eviction-warning.gif)
- [AC-004 WebM](AC-004-eviction-warning.webm)
- [AC-004 Tape](AC-004-eviction-warning.tape)

---

## AC-005 — O(1) Append

**Traces to:** BC-4.09.003, AD-006  
**Test:** `test_BC_4_09_003_append_is_constant_time`  
**Command:** `cargo test -p forge-traffic --test buffer_tests test_BC_4_09_003_append_is_constant_time -- --nocapture`

**What it demonstrates:**
- `RingBuffer::new(100, usize::MAX)` — capacity 100
- 1,000 pushes complete without panic or stack overflow
- `len()` == 100 (capacity) after fill — buffer retains newest 100
- `is_full()` == true — ring is at capacity
- Constant-time property: no shifting/copying on eviction (ring cursor advance only)

**Recordings:**
- ![AC-005 GIF](AC-005-o1-append.gif)
- [AC-005 WebM](AC-005-o1-append.webm)
- [AC-005 Tape](AC-005-o1-append.tape)

---

## EC-001 — Zero-Capacity Buffer (Edge Case)

**Scenario:** Buffer created with `capacity = 0`  
**Test:** `test_zero_capacity_buffer`  
**Command:** `cargo test -p forge-traffic --test buffer_tests test_zero_capacity_buffer -- --nocapture`

**What it demonstrates:**
- `RingBuffer::new(0, usize::MAX)` is a valid configuration (headless mode)
- Every `push(i)` returns `Some(i)` — item is immediately evicted
- `len()` remains 0 after each push
- No panics, no undefined behavior

**Recordings:**
- ![EC-001 GIF](EC-001-zero-capacity.gif)
- [EC-001 WebM](EC-001-zero-capacity.webm)
- [EC-001 Tape](EC-001-zero-capacity.tape)

---

## EC-002 — Empty Buffer Operations (Edge Case)

**Scenario:** Fresh buffer with no items pushed  
**Test:** `test_empty_buffer_operations`  
**Command:** `cargo test -p forge-traffic --test buffer_tests test_empty_buffer_operations -- --nocapture`

**What it demonstrates:**
- `len()` == 0 on fresh buffer
- `is_empty()` == true
- `is_full()` == false
- `memory_usage_bytes()` == 0
- `iter()` yields 0 items
- `capacity()` returns the configured value (10)

**Recordings:**
- ![EC-002 GIF](EC-002-empty-buffer.gif)
- [EC-002 WebM](EC-002-empty-buffer.webm)
- [EC-002 Tape](EC-002-empty-buffer.tape)

---

## Test Suite Baseline

All 7 tests passed before recording began:

```
running 7 tests
test test_BC_4_09_003_eviction_warning_emitted ... ok
test test_BC_4_09_003_fifo_eviction ... ok
test test_BC_4_09_003_ring_buffer_stores_messages ... ok
test test_empty_buffer_operations ... ok
test test_zero_capacity_buffer ... ok
test test_BC_4_09_003_memory_bound ... ok
test test_BC_4_09_003_append_is_constant_time ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

---

## File Manifest

| File | Size | Description |
|------|------|-------------|
| `AC-001-ring-buffer-stores-messages.gif` | 103K | AC-001 GIF recording |
| `AC-001-ring-buffer-stores-messages.webm` | 53K | AC-001 WebM recording |
| `AC-001-ring-buffer-stores-messages.tape` | 890B | AC-001 VHS script |
| `AC-002-fifo-eviction.gif` | 98K | AC-002 GIF recording |
| `AC-002-fifo-eviction.webm` | 77K | AC-002 WebM recording |
| `AC-002-fifo-eviction.tape` | 829B | AC-002 VHS script |
| `AC-003-memory-bound.gif` | 99K | AC-003 GIF recording |
| `AC-003-memory-bound.webm` | 77K | AC-003 WebM recording |
| `AC-003-memory-bound.tape` | 824B | AC-003 VHS script |
| `AC-004-eviction-warning.gif` | 107K | AC-004 GIF recording |
| `AC-004-eviction-warning.webm` | 51K | AC-004 WebM recording |
| `AC-004-eviction-warning.tape` | 874B | AC-004 VHS script |
| `AC-005-o1-append.gif` | 106K | AC-005 GIF recording |
| `AC-005-o1-append.webm` | 51K | AC-005 WebM recording |
| `AC-005-o1-append.tape` | 839B | AC-005 VHS script |
| `EC-001-zero-capacity.gif` | 91K | EC-001 GIF recording |
| `EC-001-zero-capacity.webm` | 72K | EC-001 WebM recording |
| `EC-001-zero-capacity.tape` | 813B | EC-001 VHS script |
| `EC-002-empty-buffer.gif` | 97K | EC-002 GIF recording |
| `EC-002-empty-buffer.webm` | 76K | EC-002 WebM recording |
| `EC-002-empty-buffer.tape` | 823B | EC-002 VHS script |
| `evidence-report.md` | — | This report |

---

## Traceability Matrix

| AC / EC | Behavioral Contract | Verification Property | Recording | Result |
|---------|--------------------|-----------------------|-----------|--------|
| AC-001 | BC-4.09.003 (stores messages) | — | AC-001-ring-buffer-stores-messages | ✅ PASS |
| AC-002 | BC-4.09.003 (FIFO eviction) | VP-005 | AC-002-fifo-eviction | ✅ PASS |
| AC-003 | BC-4.09.003 (memory bound) | NFR-012 | AC-003-memory-bound | ✅ PASS |
| AC-004 | BC-4.09.003 (E-CAP-001 threshold) | — | AC-004-eviction-warning | ✅ PASS |
| AC-005 | BC-4.09.003 (O(1) append) | AD-006 | AC-005-o1-append | ✅ PASS |
| EC-001 | BC-4.09.003 (zero capacity) | — | EC-001-zero-capacity | ✅ PASS |
| EC-002 | BC-4.09.003 (empty buffer) | — | EC-002-empty-buffer | ✅ PASS |

