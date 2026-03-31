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

use std::time::Instant;

/// Pure sliding-window throughput counter.
#[allow(dead_code)] // fields used by implementation (STORY-028 Phase 3)
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
    // Implementation state is added in Phase 3 (TDD green pass).
    // Placeholder to hold recorded timestamps.
    _timestamps: std::collections::VecDeque<Instant>,
}

impl ThroughputWindow {
    /// Create a new `ThroughputWindow` with the given window size in seconds.
    ///
    /// # Arguments
    /// * `window_secs` – Duration of the sliding window (e.g. `10` for a 10-second window).
    pub fn new(window_secs: u64) -> Self {
        Self {
            window_secs,
            _timestamps: std::collections::VecDeque::new(),
        }
    }

    /// Record a message arrival at the given `timestamp`.
    ///
    /// Appends the timestamp to the window and evicts entries older than
    /// `window_secs` from the front of the deque.
    pub fn record(&mut self, _timestamp: Instant) {
        todo!("STORY-028: implement sliding-window record")
    }

    /// Returns the current throughput in messages per second.
    ///
    /// Computed as: `count_in_window / window_secs`.
    /// Returns `0.0` when the window is empty.
    pub fn messages_per_second(&self) -> f64 {
        todo!("STORY-028: implement messages_per_second calculation")
    }
}
