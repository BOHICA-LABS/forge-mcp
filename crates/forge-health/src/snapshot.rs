//! Point-in-time metric snapshot produced from collector state.
//!
//! `MetricSnapshot` aggregates latency percentiles, throughput, error rate,
//! and alert state into a single immutable value (BC-6.13.001, AC-004).

use std::time::Instant;

use crate::latency::LatencyCollector;
use crate::throughput::ThroughputCollector;

/// Current alerting state derived from collector metrics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertState {
    /// All metrics within acceptable thresholds.
    Normal,
    /// One or more metrics approaching threshold limits.
    Warning,
    /// One or more metrics have breached threshold limits.
    Critical,
}

/// Immutable point-in-time snapshot of health metrics for a single MCP server.
///
/// Produced on demand from [`LatencyCollector`] and [`ThroughputCollector`]
/// state via [`MetricSnapshot::from_collectors`].
pub struct MetricSnapshot {
    /// Name of the MCP server this snapshot belongs to.
    pub server_name: String,
    /// Wall-clock timestamp when this snapshot was taken.
    pub timestamp: Instant,
    /// 50th-percentile request latency in milliseconds.
    pub latency_p50_ms: f64,
    /// 95th-percentile request latency in milliseconds.
    pub latency_p95_ms: f64,
    /// 99th-percentile request latency in milliseconds.
    pub latency_p99_ms: f64,
    /// Observed request throughput in requests per second.
    pub throughput_rps: f64,
    /// Observed error rate as a percentage (0.0–100.0).
    ///
    /// Always `0.0` until STORY-034 implements error-rate tracking.
    pub error_rate_pct: f64,
    /// Current alert state computed from the above metrics.
    ///
    /// Always [`AlertState::Normal`] until STORY-036 implements alert logic.
    pub alert_state: AlertState,
}

impl MetricSnapshot {
    /// Build a `MetricSnapshot` from live collector state.
    ///
    /// Queries each collector for its current values and stamps the snapshot
    /// with the current wall-clock time.
    ///
    /// # Notes
    /// - `error_rate_pct` is always `0.0` (implemented in STORY-034).
    /// - `alert_state` is always [`AlertState::Normal`] (implemented in STORY-036).
    pub fn from_collectors(
        server_name: &str,
        latency: &LatencyCollector,
        throughput: &ThroughputCollector,
    ) -> Self {
        Self {
            server_name: server_name.to_owned(),
            timestamp: Instant::now(),
            latency_p50_ms: latency.p50(),
            latency_p95_ms: latency.p95(),
            latency_p99_ms: latency.p99(),
            throughput_rps: throughput.messages_per_second(),
            error_rate_pct: 0.0,
            alert_state: AlertState::Normal,
        }
    }
}
