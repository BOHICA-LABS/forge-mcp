//! Windowed throughput counter for passive MCP message rate tracking.
//!
//! `ThroughputCollector` counts messages received within a sliding window and
//! exposes a messages-per-second rate (BC-6.13.001, AC-002).

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Passive windowed throughput collector.
///
/// Counts every observed `MessageCaptured` event and computes a
/// messages-per-second rate across a configurable sliding window.
pub struct ThroughputCollector {
    /// Width of the measurement window as a [`Duration`].
    window: Duration,
    /// Ring buffer of message arrival timestamps within the current window.
    timestamps: VecDeque<Instant>,
}

impl ThroughputCollector {
    /// Create a new `ThroughputCollector` with the given window size (seconds).
    pub fn new(window_secs: u64) -> Self {
        Self {
            window: Duration::from_secs(window_secs),
            timestamps: VecDeque::new(),
        }
    }

    /// Record a single message event in the sliding window.
    ///
    /// Pushes the current timestamp and evicts any entries older than the
    /// configured window.
    pub fn record_message(&mut self) {
        let now = Instant::now();
        self.timestamps.push_back(now);
        self.evict_stale(now);
    }

    /// Return the current messages-per-second rate.
    ///
    /// Returns `0.0` if no messages have been recorded within the window.
    pub fn messages_per_second(&self) -> f64 {
        if self.timestamps.is_empty() {
            return 0.0;
        }
        let count = self.timestamps.len() as f64;
        let window_secs = self.window.as_secs_f64();
        count / window_secs
    }

    /// Evict timestamps older than the sliding window relative to `now`.
    fn evict_stale(&mut self, now: Instant) {
        while let Some(&front) = self.timestamps.front() {
            if now.duration_since(front) > self.window {
                self.timestamps.pop_front();
            } else {
                break;
            }
        }
    }
}
