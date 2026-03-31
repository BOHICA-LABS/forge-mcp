//! Per-message timing analysis (STORY-028).
//!
//! [`TimingAnalyzer`] is a pure, stateful component that processes a stream of
//! [`MessageCaptured`] events and produces [`TimedMessage`] records with
//! latency annotations.
//!
//! ## Design
//! - Pure computation — no I/O, no async (VP-006 purity classification).
//! - Matches requests to responses by JSON-RPC `id` field.
//! - Unmatched responses produce `latency_ms: None` (AC-003).
//! - Preserves received-order; flags reordered batch responses (AC-004).

use forge_core::events::MessageCaptured;

use crate::types::TimedMessage;

/// Pure timing analyzer for MCP message streams.
///
/// Feed [`MessageCaptured`] events in received order via [`process_message`].
/// Returns a [`TimedMessage`] for every event, with latency populated where
/// a matching request/response pair can be resolved.
///
/// ## Purity
/// This struct holds only immutable computation state (pending request map,
/// sequence counter). It performs no I/O and is safe to use in single-threaded
/// contexts.
pub struct TimingAnalyzer {
    // Implementation state is added in Phase 3 (TDD green pass).
    // Placeholder field to make the struct non-zero-sized and forward-
    // compatible with the implementation.
    _pending: std::collections::HashMap<String, std::time::Instant>,
    _sequence: u64,
}

impl TimingAnalyzer {
    /// Create a new, empty `TimingAnalyzer`.
    pub fn new() -> Self {
        Self {
            _pending: std::collections::HashMap::new(),
            _sequence: 0,
        }
    }

    /// Process a single captured message and return a [`TimedMessage`].
    ///
    /// For request messages, records the capture timestamp keyed by the
    /// JSON-RPC `id`. For response messages, looks up the matching request and
    /// computes `latency_ms`. Unmatched responses return `latency_ms: None`.
    ///
    /// Returns `None` if the message should be silently dropped (e.g. a
    /// duplicate response that has already been processed).
    pub fn process_message(&mut self, _msg: &MessageCaptured) -> Option<TimedMessage> {
        todo!("STORY-028: implement request/response matching and latency computation")
    }
}

impl Default for TimingAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
