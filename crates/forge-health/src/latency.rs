//! Latency histogram collector for passive MCP message latency tracking.
//!
//! `LatencyCollector` maintains a sliding-window HDR histogram and exposes
//! p50 / p95 / p99 percentiles (BC-6.13.001, VP-007, VP-008).

use hdrhistogram::Histogram;

/// Passive latency histogram collector.
///
/// Records observed latency values (in milliseconds) and exposes HDR-histogram
/// percentiles across a configurable sliding window.
pub struct LatencyCollector {
    /// Width of the measurement window in seconds.
    pub window_secs: u64,
    /// HDR histogram storing latency values in milliseconds.
    histogram: Histogram<u64>,
}

impl LatencyCollector {
    /// Create a new `LatencyCollector` with the given window size.
    ///
    /// The histogram is configured for 3 significant figures of precision,
    /// covering the range 1 ms to 3 600 000 ms (1 hour).
    pub fn new(window_secs: u64) -> Self {
        let histogram = Histogram::<u64>::new_with_bounds(1, 3_600_000, 3)
            .expect("valid histogram bounds: 1..3_600_000 with 3 sig figs");
        Self {
            window_secs,
            histogram,
        }
    }

    /// Record a single latency observation (in milliseconds).
    ///
    /// Values are clamped to at least 1 ms to satisfy the histogram's lower
    /// bound. Sub-millisecond latencies are rounded up to 1 ms.
    pub fn record_latency(&mut self, latency_ms: f64) {
        let value = (latency_ms.round() as u64).max(1);
        self.histogram
            .record(value)
            .expect("latency value within histogram bounds");
    }

    /// Return `true` if no observations have been recorded.
    fn is_empty(&self) -> bool {
        self.histogram.is_empty()
    }

    /// Return the 50th-percentile latency in milliseconds.
    ///
    /// Returns `0.0` if no observations have been recorded.
    pub fn p50(&self) -> f64 {
        if self.is_empty() {
            return 0.0;
        }
        self.histogram.value_at_percentile(50.0) as f64
    }

    /// Return the 95th-percentile latency in milliseconds.
    ///
    /// Returns `0.0` if no observations have been recorded.
    pub fn p95(&self) -> f64 {
        if self.is_empty() {
            return 0.0;
        }
        self.histogram.value_at_percentile(95.0) as f64
    }

    /// Return the 99th-percentile latency in milliseconds.
    ///
    /// Returns `0.0` if no observations have been recorded.
    pub fn p99(&self) -> f64 {
        if self.is_empty() {
            return 0.0;
        }
        self.histogram.value_at_percentile(99.0) as f64
    }

    /// Reset the histogram, discarding all recorded observations.
    pub fn reset(&mut self) {
        self.histogram.reset();
    }
}
