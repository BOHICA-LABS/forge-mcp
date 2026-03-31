//! Sliding-window throughput measurement (STORY-028).
//!
//! [`ThroughputWindow`] tracks message arrival timestamps within a configurable
//! time window and reports the current throughput in messages per second.
//!
//! ## Design
//! - Pure computation — no I/O, no async (VP-006 purity classification).
//! - Configurable window size in seconds (default 10 s per AC-002).
//! - Updated on every new [`MessageCaptured`] event via [`record`].
//! - Expired entries are lazily evicted on each [`record`] or
//!   [`messages_per_second`] call.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

/// Pure sliding-window throughput counter.
///
/// Records message arrival timestamps and computes the instantaneous
/// throughput (messages/second) over the configured window.
///
/// ## Example
/// ```ignore
/// let mut window = ThroughputWindow::new(10);
/// window.record(Instant::now());
/// let mps = window.messages_per_second();
/// ```
pub struct ThroughputWindow {
    /// Window duration in seconds.
    window_secs: u64,
    /// Recorded timestamps within the sliding window.
    timestamps: VecDeque<Instant>,
}

impl ThroughputWindow {
    /// Create a new `ThroughputWindow` with the given window size in seconds.
    ///
    /// # Arguments
    /// * `window_secs` – Duration of the sliding window (e.g. `10` for a 10-second window).
    pub fn new(window_secs: u64) -> Self {
        Self {
            window_secs,
            timestamps: VecDeque::new(),
        }
    }

    /// Record a message arrival at the given `timestamp`.
    ///
    /// Appends the timestamp to the window and evicts entries older than
    /// `window_secs` from the front of the deque.
    pub fn record(&mut self, timestamp: Instant) {
        self.timestamps.push_back(timestamp);
        self.evict(timestamp);
    }

    /// Returns the current throughput in messages per second.
    ///
    /// Computed as: `count_in_window / window_secs`.
    /// Returns `0.0` when the window is empty.
    pub fn messages_per_second(&self) -> f64 {
        if self.timestamps.is_empty() {
            return 0.0;
        }
        self.timestamps.len() as f64 / self.window_secs as f64
    }

    /// Evict timestamps outside the sliding window relative to `reference`.
    fn evict(&mut self, reference: Instant) {
        let window = Duration::from_secs(self.window_secs);
        // Remove from front while the entry is older than `window_secs` ago.
        while let Some(&front) = self.timestamps.front() {
            if reference.duration_since(front) > window {
                self.timestamps.pop_front();
            } else {
                break;
            }
        }
    }
}
