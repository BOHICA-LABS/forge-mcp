//! Integration tests for STORY-033: Passive Latency & Throughput Metric Collection.
//!
//! All tests in this file must FAIL until the stubs in:
//!   - `forge-health/src/latency.rs`
//!   - `forge-health/src/throughput.rs`
//!   - `forge-health/src/snapshot.rs`
//! are implemented (Red Gate — BC-6.13.001, VP-007, VP-008).

use forge_health::{AlertState, LatencyCollector, MetricSnapshot, ThroughputCollector};
use proptest::prelude::*;

// ─── Acceptance Criteria Tests ───────────────────────────────────────────────

/// AC-001 (BC-6.13.001 postcondition — latency histogram)
///
/// Records ten latencies evenly spaced 10..=100 ms and asserts that the
/// HDR histogram produces correct percentile estimates:
///   p50 ≈ 50 ms  (±5%)
///   p95 ≈ 95 ms  (±10%)
///   p99 ≈ 99 ms  (±10%)
#[test]
fn test_BC_6_13_001_latency_histogram_computed() {
    let mut collector = LatencyCollector::new(60);

    for i in 1..=10u64 {
        collector.record_latency(i as f64 * 10.0); // 10, 20, ..., 100 ms
    }

    let p50 = collector.p50();
    let p95 = collector.p95();
    let p99 = collector.p99();

    // p50 ≈ 50 ms ±5%
    assert!(
        (p50 - 50.0).abs() / 50.0 <= 0.05,
        "p50 {p50} not within 5% of 50.0"
    );
    // p95 ≈ 95 ms ±10%
    assert!(
        (p95 - 95.0).abs() / 95.0 <= 0.10,
        "p95 {p95} not within 10% of 95.0"
    );
    // p99 ≈ 99 ms ±10%
    assert!(
        (p99 - 99.0).abs() / 99.0 <= 0.10,
        "p99 {p99} not within 10% of 99.0"
    );
}

/// AC-002 (BC-6.13.001 postcondition — throughput counter)
///
/// Creates a ThroughputCollector with a 10-second window, records 100
/// messages, and asserts that the computed rate is > 0 rps.
#[test]
fn test_BC_6_13_001_throughput_counter_updates() {
    let mut collector = ThroughputCollector::new(10);

    for _ in 0..100 {
        collector.record_message();
    }

    let rps = collector.messages_per_second();
    assert!(
        rps > 0.0,
        "messages_per_second should be > 0 after recording 100 messages, got {rps}"
    );
}

/// AC-004 (BC-6.13.001 — MetricSnapshot structure)
///
/// Creates LatencyCollector and ThroughputCollector, records representative
/// data, then asserts that `MetricSnapshot::from_collectors` returns a
/// snapshot with:
///   - correct server_name
///   - non-zero latency percentiles
///   - throughput_rps > 0
#[test]
fn test_BC_6_13_001_metric_snapshot_structure() {
    let mut latency = LatencyCollector::new(60);
    let mut throughput = ThroughputCollector::new(10);

    // Record enough data for non-zero metrics
    for ms in [10.0_f64, 50.0, 100.0, 200.0, 500.0] {
        latency.record_latency(ms);
    }
    for _ in 0..50 {
        throughput.record_message();
    }

    let snapshot = MetricSnapshot::from_collectors("test-server", &latency, &throughput);

    assert_eq!(snapshot.server_name, "test-server", "server_name mismatch");
    assert!(
        snapshot.latency_p50_ms > 0.0,
        "latency_p50_ms should be > 0, got {}",
        snapshot.latency_p50_ms
    );
    assert!(
        snapshot.latency_p95_ms > 0.0,
        "latency_p95_ms should be > 0, got {}",
        snapshot.latency_p95_ms
    );
    assert!(
        snapshot.latency_p99_ms > 0.0,
        "latency_p99_ms should be > 0, got {}",
        snapshot.latency_p99_ms
    );
    assert!(
        snapshot.throughput_rps > 0.0,
        "throughput_rps should be > 0, got {}",
        snapshot.throughput_rps
    );
}

// ─── Edge Case Tests ──────────────────────────────────────────────────────────

/// EC-001: Zero messages — all metrics must be zero, no panic.
#[test]
fn test_zero_messages_all_metrics_zero() {
    let latency = LatencyCollector::new(60);
    let throughput = ThroughputCollector::new(10);

    assert_eq!(latency.p50(), 0.0, "p50 should be 0.0 on empty collector");
    assert_eq!(latency.p95(), 0.0, "p95 should be 0.0 on empty collector");
    assert_eq!(latency.p99(), 0.0, "p99 should be 0.0 on empty collector");
    assert_eq!(
        throughput.messages_per_second(),
        0.0,
        "messages_per_second should be 0.0 on empty collector"
    );
}

/// EC-002: Single message — histogram collapses to a single point.
///
/// With exactly one recorded value (42.0 ms), all percentiles must equal
/// that value.
#[test]
fn test_single_message_histogram() {
    let mut collector = LatencyCollector::new(60);
    collector.record_latency(42.0);

    assert_eq!(
        collector.p50(),
        42.0,
        "p50 should equal the sole recorded value"
    );
    assert_eq!(
        collector.p95(),
        42.0,
        "p95 should equal the sole recorded value"
    );
    assert_eq!(
        collector.p99(),
        42.0,
        "p99 should equal the sole recorded value"
    );
}

/// EC-003: Very high latency (60 000 ms / 60 s) — no overflow, exact value stored.
#[test]
fn test_very_high_latency_no_overflow() {
    let mut collector = LatencyCollector::new(60);
    collector.record_latency(60_000.0); // 60 seconds in ms

    let p50 = collector.p50();
    assert!(
        p50 > 0.0,
        "p50 should be > 0 after recording 60 000 ms, got {p50}"
    );
    // HDR histogram has ~1% precision; allow 1% tolerance
    assert!(
        (p50 - 60_000.0).abs() / 60_000.0 <= 0.01,
        "p50 {p50} should be within 1% of 60 000.0 ms"
    );
}

/// reset() must clear all recorded observations so subsequent percentile
/// queries return 0.0.
#[test]
fn test_reset_clears_histogram() {
    let mut collector = LatencyCollector::new(60);

    for ms in [10.0_f64, 50.0, 100.0] {
        collector.record_latency(ms);
    }

    collector.reset();

    assert_eq!(collector.p50(), 0.0, "p50 should be 0.0 after reset");
    assert_eq!(collector.p95(), 0.0, "p95 should be 0.0 after reset");
    assert_eq!(collector.p99(), 0.0, "p99 should be 0.0 after reset");
}

// ─── Property-Based Tests (VP-008) ───────────────────────────────────────────

proptest! {
    /// VP-008: HDR histogram percentile ordering invariant.
    ///
    /// For any non-empty sequence of latencies in [1.0, 10 000.0] ms:
    ///   1. p50 ≤ p95 ≤ p99           (monotonicity)
    ///   2. All percentiles ≥ min(values) and ≤ max(values)  (range containment)
    #[test]
    fn proptest_latency_histogram_correctness(
        // Generate 1–500 latency values in 1..=10000 ms
        latencies in prop::collection::vec(1.0_f64..=10_000.0_f64, 1..=500)
    ) {
        let mut collector = LatencyCollector::new(60);
        for &ms in &latencies {
            collector.record_latency(ms);
        }

        let p50 = collector.p50();
        let p95 = collector.p95();
        let p99 = collector.p99();

        let min_val = latencies.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_val = latencies.iter().cloned().fold(f64::NEG_INFINITY, f64::max);

        // Monotonicity: p50 ≤ p95 ≤ p99
        prop_assert!(p50 <= p95, "p50 ({p50}) > p95 ({p95})");
        prop_assert!(p95 <= p99, "p95 ({p95}) > p99 ({p99})");

        // Range containment: all percentiles within [min, max] of recorded data
        // Allow 1% HDR precision headroom
        let tolerance = max_val * 0.01 + 1.0;
        prop_assert!(
            p50 >= min_val - tolerance && p50 <= max_val + tolerance,
            "p50 ({p50}) out of range [{min_val}, {max_val}]"
        );
        prop_assert!(
            p95 >= min_val - tolerance && p95 <= max_val + tolerance,
            "p95 ({p95}) out of range [{min_val}, {max_val}]"
        );
        prop_assert!(
            p99 >= min_val - tolerance && p99 <= max_val + tolerance,
            "p99 ({p99}) out of range [{min_val}, {max_val}]"
        );
    }
}
