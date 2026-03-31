//! Core types for per-message timing analysis (STORY-028).

use std::time::Instant;

use forge_core::events::MessageDirection;
use uuid::Uuid;

/// A captured message augmented with timing and ordering metadata.
///
/// Produced by [`crate::timing::TimingAnalyzer`] for each message processed.
/// Latency is `None` when the response has no matching request (e.g. server-
/// initiated notifications, or late/duplicate responses — AC-003).
#[derive(Debug, Clone)]
pub struct TimedMessage {
    /// Capture event ID (copied from [`MessageCaptured::id`]).
    pub id: Uuid,
    /// Wall-clock timestamp at the moment of capture (from [`MessageCaptured::timestamp`]).
    pub timestamp: Instant,
    /// Message direction (client→server or server→client).
    pub direction: MessageDirection,
    /// Raw JSON-RPC payload, byte-identical to the wire representation (VP-006).
    pub payload: serde_json::Value,
    /// Round-trip latency in milliseconds, computed as the elapsed time between
    /// the matching request and this response `MessageCaptured` event.
    ///
    /// `None` when no matching request was found (AC-003).
    pub latency_ms: Option<f64>,
    /// `true` when this message arrived out of order relative to its request
    /// sequence number (DI-006 / AC-004, E-PRO-009 batch reordering).
    pub reordered: bool,
}
