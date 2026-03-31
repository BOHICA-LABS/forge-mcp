//! Windowed throughput counter for passive MCP message rate tracking.
//!
//! `ThroughputCollector` counts messages received within a sliding window and
//! exposes a messages-per-second rate (BC-6.13.001, AC-002).

/// Passive windowed throughput collector.
///
/// Counts every observed `MessageCaptured` event and computes a
/// messages-per-second rate across a configurable sliding window.
pub struct ThroughputCollector {
    /// Width of the measurement window in seconds.
    pub window_secs: u64,
}

impl ThroughputCollector {
    /// Create a new `ThroughputCollector` with the given window size.
    pub fn new(window_secs: u64) -> Self {
        Self { window_secs }
    }

    /// Record a single message event in the sliding window.
    pub fn record_message(&mut self) {
        todo!("STORY-033: implement windowed message counter")
    }

    /// Return the current messages-per-second rate.
    pub fn messages_per_second(&self) -> f64 {
        todo!("STORY-033: implement messages-per-second computation")
    }
}
