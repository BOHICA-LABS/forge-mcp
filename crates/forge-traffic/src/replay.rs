//! Message sequence replay against a target server.
//!
//! Provides [`replay_sequence`] — the effectful function that sends each
//! captured client→server message to an explicitly designated target server
//! and returns the corresponding responses.
//!
//! ## Design invariants
//! - **DI-007**: Replay requires an explicit target designation; the caller
//!   must supply a connected `ReplayTarget`. The function never defaults to
//!   the original server.
//! - **DI-005**: Replayed message bytes are identical to the captured bytes.
//! - **INV-003**: Messages are sent in original wire order (temporal order).
//! - **INV-004**: The original capture buffer is never modified.
//! - **INV-005**: Replay responses are stored separately from the capture buffer.
//!
//! ## Error codes
//! - `E-CAP-002` — replay target server not connected.
//! - `E-RPL-001` — target server unreachable.
//! - `E-RPL-002` — connection lost mid-replay.
//!
//! STORY-032 | BC-4.10.003 | VP-006

use forge_core::events::{MessageCaptured, MessageDirection};

// ── Error type ────────────────────────────────────────────────────────────────

/// Errors specific to the replay subsystem.
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ReplayError {
    /// `E-CAP-002`: The target server is not connected.
    ///
    /// Returned by [`replay_sequence`] when the caller supplies a
    /// [`ReplayTarget::Disconnected`] handle.
    #[error("E-CAP-002: replay target server not connected")]
    TargetNotConnected,

    /// `E-RPL-001`: The target server is unreachable.
    #[error("E-RPL-001: target server unreachable: {address}")]
    TargetUnreachable { address: String },

    /// `E-RPL-002`: The connection was lost mid-replay after N messages.
    #[error("E-RPL-002: connection lost after {sent} of {total} messages")]
    ConnectionLostMidReplay { sent: usize, total: usize },
}

// ── Result type ──────────────────────────────────────────────────────────────

/// Convenience alias.
pub type ReplayResult<T> = std::result::Result<T, ReplayError>;

// ── ReplayTarget ─────────────────────────────────────────────────────────────

/// A handle to the target server for replay operations.
///
/// DI-007: the caller must explicitly supply a target — the replay function
/// never infers or defaults to the original server.
///
/// In the full implementation this wraps an `McpConnection`; during stubs we
/// use a simple enum so tests can drive error paths without a live process.
#[derive(Debug)]
pub enum ReplayTarget {
    /// Target is connected and ready to accept messages.
    Connected {
        /// Human-readable label (server name / URL).
        label: String,
        /// Whether this target is the same server that produced the original
        /// capture (EC-007 — allowed, but a warning is shown).
        is_original: bool,
    },
    /// Target is not connected (E-CAP-002).
    Disconnected,
}

impl ReplayTarget {
    /// Returns `true` if the target is in the `Connected` state.
    pub fn is_connected(&self) -> bool {
        matches!(self, ReplayTarget::Connected { .. })
    }

    /// Returns the label of the connected target, or `None` if disconnected.
    pub fn label(&self) -> Option<&str> {
        match self {
            ReplayTarget::Connected { label, .. } => Some(label),
            ReplayTarget::Disconnected => None,
        }
    }

    /// Returns `true` when the target is the original server (EC-007).
    pub fn is_original_server(&self) -> bool {
        match self {
            ReplayTarget::Connected { is_original, .. } => *is_original,
            ReplayTarget::Disconnected => false,
        }
    }
}

// ── ReplayResponse ────────────────────────────────────────────────────────────

/// A single response received from the target server during replay.
///
/// Replay responses are tagged with `replay: true` to distinguish them from
/// live traffic (AC-003).
#[derive(Debug, Clone)]
pub struct ReplayResponse {
    /// Index of the original message in the captured sequence (0-based).
    pub sequence_index: usize,
    /// The captured event that was replayed.
    pub original: MessageCaptured,
    /// The raw JSON response from the target server.
    ///
    /// `None` if the request timed out (EC-004 — "no response").
    pub response_payload: Option<serde_json::Value>,
    /// `true` — this response came from a replay, not live traffic (AC-003).
    pub replay: bool,
    /// Status of the response relative to the original captured response.
    pub status: ReplayStatus,
}

/// Classification of a replayed response vs. the original.
#[derive(Debug, Clone, PartialEq)]
pub enum ReplayStatus {
    /// Responses are byte-identical.
    Identical,
    /// Responses are structurally equivalent (e.g. differing timestamps only).
    Equivalent,
    /// Responses differ in semantically meaningful fields.
    Divergent,
    /// The target server did not respond within the timeout (EC-004).
    NoResponse,
    /// Replay was a notification — no response expected (EC-006).
    SentNotification,
    /// An error occurred while sending this message.
    Error(String),
}

// ── replay_sequence ───────────────────────────────────────────────────────────

/// Replay a captured message sequence against the designated target server.
///
/// # Behaviour
/// 1. Filters `messages` to client→server direction only (EC-005).
/// 2. Verifies `target` is connected — returns `Err(E-CAP-002)` if not (AC-002).
/// 3. Sends each message in temporal order (AC-004 / INV-003 / DI-007).
/// 4. Collects responses and tags them with `replay: true` (AC-003).
/// 5. Returns the collected `ReplayResponse` list.
///
/// # Arguments
/// - `target`   — the explicitly designated target server (DI-007).
/// - `messages` — the full capture sequence (both directions); the function
///   internally filters to `ClientToServer` only.
///
/// # Errors
/// - [`ReplayError::TargetNotConnected`] — target is not connected (E-CAP-002).
/// - [`ReplayError::TargetUnreachable`]  — initial connection check fails (E-RPL-001).
/// - [`ReplayError::ConnectionLostMidReplay`] — server disconnects mid-replay (E-RPL-002).
///
/// # Invariants preserved
/// - `messages` is consumed by reference; the caller's copy is unchanged (INV-004).
/// - Each message byte payload is sent unmodified (INV-002 / DI-005).
/// - Messages are sent in the order they appear in `messages` (INV-003).
pub async fn replay_sequence(
    target: &ReplayTarget,
    messages: &[MessageCaptured],
) -> ReplayResult<Vec<ReplayResponse>> {
    todo!("STORY-032: implement replay_sequence — AC-001, AC-002, AC-003, AC-004")
}

// ── Filter helper (pure) ─────────────────────────────────────────────────────

/// Filter a message slice to client→server messages only.
///
/// Server→client messages (EC-005) and all other directions are excluded.
/// Returns the filtered messages with their original indices (for sequence
/// position tracking).
pub fn filter_client_to_server(
    messages: &[MessageCaptured],
) -> Vec<(usize, &MessageCaptured)> {
    messages
        .iter()
        .enumerate()
        .filter(|(_, m)| m.direction == MessageDirection::ClientToServer)
        .collect()
}

// ── ComparisonReport ─────────────────────────────────────────────────────────

/// Summary comparison report produced after replay completes (POST-003).
#[derive(Debug, Clone)]
pub struct ComparisonReport {
    /// Total messages replayed (client→server only).
    pub total_replayed: usize,
    /// Count of identical responses.
    pub identical: usize,
    /// Count of structurally equivalent (timestamp-differing) responses.
    pub equivalent: usize,
    /// Count of divergent responses.
    pub divergent: usize,
    /// Count of timed-out responses (no response within timeout).
    pub no_response: usize,
    /// Individual replay results, ordered by sequence position.
    pub entries: Vec<ReplayResponse>,
}

impl ComparisonReport {
    /// Build a `ComparisonReport` from a completed replay response list.
    pub fn from_responses(responses: Vec<ReplayResponse>) -> Self {
        todo!("STORY-032: implement ComparisonReport::from_responses")
    }

    /// Returns the percentage of identical responses (0.0–100.0).
    pub fn match_percentage(&self) -> f64 {
        todo!("STORY-032: implement ComparisonReport::match_percentage")
    }
}

// ── ReplayProgress ────────────────────────────────────────────────────────────

/// Progress state for TUI display (POST-006).
#[derive(Debug, Clone, PartialEq)]
pub struct ReplayProgress {
    /// Total number of client→server messages to replay.
    pub total: usize,
    /// Number of messages sent so far.
    pub sent: usize,
    /// Number of responses received so far.
    pub received: usize,
}

impl ReplayProgress {
    /// Create a new `ReplayProgress` for `total` messages.
    pub fn new(total: usize) -> Self {
        todo!("STORY-032: implement ReplayProgress::new")
    }

    /// Increment the sent counter.
    pub fn increment_sent(&mut self) {
        todo!("STORY-032: implement ReplayProgress::increment_sent")
    }

    /// Increment the received counter.
    pub fn increment_received(&mut self) {
        todo!("STORY-032: implement ReplayProgress::increment_received")
    }
}
