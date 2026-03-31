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
//!
//! ## Reordering Semantics (PRF-001)
//!
//! `TimedMessage.reordered` has the following precise semantics:
//!
//! | Scenario | Response.reordered | Request.reordered |
//! |----------|--------------------|-------------------|
//! | Normal (req → resp) | `false` | `false` |
//! | Out-of-order (resp → req) | `false` | `true` |
//! | Truly unmatched response | `false` | N/A (no request) |
//!
//! When a response arrives before its matching request, we cannot immediately
//! determine if it is out-of-order or genuinely unmatched.  We optimistically
//! emit it with `reordered: false` and track its ID in `orphan_responses`.
//! If the matching request later arrives, the **request** `TimedMessage` is
//! emitted with `reordered: true` — signalling that the pair was inverted.
//! Truly unmatched responses (whose request never arrives) retain `reordered:
//! false`, correctly distinguishing them from the out-of-order case.

use std::collections::{HashMap, HashSet};
use std::time::Instant;

use forge_core::events::MessageCaptured;

use crate::types::TimedMessage;

/// Entry stored in the pending-request map.
struct PendingRequest {
    /// Capture timestamp from the [`MessageCaptured`] event.
    timestamp: Instant,
}

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
    /// Map from JSON-RPC `id` (as string) to pending request metadata.
    pending: HashMap<String, PendingRequest>,
    /// Set of JSON-RPC `id` values whose first response has already been matched
    /// and consumed. Used to detect duplicate responses (EC-002).
    consumed: HashSet<String>,
    /// Set of JSON-RPC `id` values for responses that arrived before their
    /// matching request (orphan responses).  When the late request arrives,
    /// its `TimedMessage` is emitted with `reordered: true` to signal the
    /// inversion.  Truly unmatched responses (whose request never arrives)
    /// remain in this set but their already-emitted `TimedMessage` correctly
    /// has `reordered: false` (PRF-001 fix).
    orphan_responses: HashSet<String>,
}

impl TimingAnalyzer {
    /// Create a new, empty `TimingAnalyzer`.
    pub fn new() -> Self {
        Self {
            pending: HashMap::new(),
            consumed: HashSet::new(),
            orphan_responses: HashSet::new(),
        }
    }

    /// Process a single captured message and return a [`TimedMessage`].
    ///
    /// For request messages, records the capture timestamp keyed by the
    /// JSON-RPC `id`. For response messages, looks up the matching request and
    /// computes `latency_ms`. Unmatched responses return `latency_ms: None`.
    ///
    /// Returns `None` if the message should be silently dropped (e.g. a
    /// duplicate response that has already been processed — EC-002).
    ///
    /// ## Reordering flag placement
    ///
    /// The `reordered` flag is set on the **request** `TimedMessage` when its
    /// matching response had already arrived earlier (out-of-order).  A
    /// response emitted without a matching request always has `reordered:
    /// false` — we cannot distinguish "truly unmatched" from "out-of-order"
    /// at response-emit time, so we use the conservative/correct default and
    /// fix it when the late request arrives (PRF-001).
    pub fn process_message(&mut self, msg: &MessageCaptured) -> Option<TimedMessage> {
        // Extract the JSON-RPC `id` field from the payload, if present.
        let rpc_id = extract_rpc_id(&msg.payload);

        // Determine whether this message is a request or a response.
        // Heuristic: requests have a `method` field; responses have `result`
        // or `error`. Notifications also have `method` but no `id`.
        let is_response = msg.payload.get("result").is_some() || msg.payload.get("error").is_some();
        let is_request = msg.method.is_some() && rpc_id.is_some() && !is_response;

        if is_request {
            let id_key = rpc_id.expect("checked above");

            // Out-of-order detection (AC-004 / PRF-001): if a response for
            // this id already arrived (orphan), flag this request as reordered.
            let reordered = self.orphan_responses.remove(&id_key);

            // Store in pending map so a future response can compute latency.
            // If the response already arrived (reordered == true), there is no
            // pending entry to store — and no future response is expected —
            // so we skip insertion in that case.
            if !reordered {
                self.pending.insert(
                    id_key,
                    PendingRequest {
                        timestamp: msg.timestamp,
                    },
                );
            }

            Some(TimedMessage {
                id: msg.id,
                timestamp: msg.timestamp,
                direction: msg.direction.clone(),
                payload: msg.payload.clone(),
                latency_ms: None,
                reordered,
            })
        } else if is_response {
            let id_key = match rpc_id {
                Some(id) => id,
                // Response without an id — treat as unmatched (no reordering).
                None => {
                    return Some(TimedMessage {
                        id: msg.id,
                        timestamp: msg.timestamp,
                        direction: msg.direction.clone(),
                        payload: msg.payload.clone(),
                        latency_ms: None,
                        reordered: false,
                    });
                }
            };

            // Duplicate detection (EC-002): if this id was already consumed,
            // drop the duplicate silently.
            if self.consumed.contains(&id_key) {
                return None;
            }

            // Look up the pending request.
            if let Some(pending) = self.pending.remove(&id_key) {
                // Matched pair in normal order (request arrived first).
                // Mark as consumed so later duplicates are dropped.
                self.consumed.insert(id_key);

                let latency_ms = duration_ms(pending.timestamp, msg.timestamp);

                Some(TimedMessage {
                    id: msg.id,
                    timestamp: msg.timestamp,
                    direction: msg.direction.clone(),
                    payload: msg.payload.clone(),
                    latency_ms: Some(latency_ms),
                    reordered: false,
                })
            } else {
                // No pending request found.
                //
                // The response may be:
                //   (a) Genuinely unmatched (AC-003) — id never requested.
                //   (b) Out-of-order (AC-004) — request hasn't arrived yet.
                //
                // We cannot distinguish these at emission time.  Per PRF-001
                // the correct behaviour is:
                //   • Emit the response with `reordered: false` (optimistic).
                //   • Track the id in `orphan_responses`.
                //   • When/if the late request arrives, flag the REQUEST with
                //     `reordered: true` (see the `is_request` branch above).
                //   • Truly unmatched responses (request never arrives) keep
                //     `reordered: false` — correct by default.
                //
                // Mark as consumed immediately so duplicate responses to the
                // same orphan id are still dropped correctly (EC-002).
                self.consumed.insert(id_key.clone());
                self.orphan_responses.insert(id_key);

                Some(TimedMessage {
                    id: msg.id,
                    timestamp: msg.timestamp,
                    direction: msg.direction.clone(),
                    payload: msg.payload.clone(),
                    latency_ms: None,
                    reordered: false,
                })
            }
        } else {
            // Notification or unknown — no latency pairing (EC-001).
            Some(TimedMessage {
                id: msg.id,
                timestamp: msg.timestamp,
                direction: msg.direction.clone(),
                payload: msg.payload.clone(),
                latency_ms: None,
                reordered: false,
            })
        }
    }
}

impl Default for TimingAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Extract the JSON-RPC `id` as a `String` key from a payload `Value`.
///
/// The spec allows `id` to be a number, string, or null.  We normalise to a
/// `String` for use as a `HashMap` key.  Returns `None` if the field is absent
/// or null.
fn extract_rpc_id(payload: &serde_json::Value) -> Option<String> {
    match payload.get("id") {
        Some(serde_json::Value::Number(n)) => Some(n.to_string()),
        Some(serde_json::Value::String(s)) => Some(s.clone()),
        _ => None,
    }
}

/// Compute elapsed milliseconds between two `Instant` values as `f64`.
///
/// If `end` is before `start` (e.g. out-of-order timestamps), returns `0.0`
/// rather than panicking.
fn duration_ms(start: Instant, end: Instant) -> f64 {
    end.checked_duration_since(start)
        .map(|d| d.as_secs_f64() * 1000.0)
        .unwrap_or(0.0)
}
