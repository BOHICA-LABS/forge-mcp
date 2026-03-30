//! Progress event bus for MCP `notifications/progress` dispatching.
//!
//! ## Responsibilities
//! - Broadcast `ProgressEvent` to all subscribers via tokio broadcast channel
//! - Track cancelled request tokens (E-PRO-006)
//! - Silently drop progress for cancelled tokens (orphan guard)
//!
//! ## Error codes
//! - `E-PRO-006`: orphaned progress notification silently dropped.

use std::collections::HashSet;
use std::sync::{Arc, Mutex};

use rmcp::model::ProgressToken;
use tokio::sync::broadcast;

// ── ProgressEvent ─────────────────────────────────────────────────────────────

/// A single progress notification from the MCP server.
///
/// Mirrors the MCP `notifications/progress` payload with Forge-friendly types.
#[derive(Debug, Clone, PartialEq)]
pub struct ProgressEvent {
    /// The request token this progress notification belongs to.
    pub token: ProgressToken,
    /// Current progress value.
    pub current: f64,
    /// Optional total, `None` if the server omitted it.
    pub total: Option<f64>,
}

// ── ProgressBus ───────────────────────────────────────────────────────────────

/// Broadcast bus for `notifications/progress` events.
///
/// Cheap to clone — the inner state is reference-counted.
///
/// ## Usage
/// ```ignore
/// let bus = ProgressBus::new();
/// let mut rx = bus.subscribe();
///
/// bus.dispatch(ProgressEvent { token, current: 1.0, total: Some(10.0) });
///
/// if let Ok(evt) = rx.try_recv() {
///     println!("progress: {}/{:?}", evt.current, evt.total);
/// }
/// ```
#[derive(Clone, Debug)]
pub struct ProgressBus {
    inner: Arc<ProgressBusInner>,
}

#[derive(Debug)]
struct ProgressBusInner {
    tx: broadcast::Sender<ProgressEvent>,
    cancelled: Mutex<HashSet<String>>,
}

/// Default channel capacity for the broadcast bus.
const BUS_CAPACITY: usize = 256;

impl ProgressBus {
    /// Create a new `ProgressBus` with the default channel capacity.
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(BUS_CAPACITY);
        Self {
            inner: Arc::new(ProgressBusInner {
                tx,
                cancelled: Mutex::new(HashSet::new()),
            }),
        }
    }

    /// Subscribe to progress events.
    ///
    /// Returns a [`broadcast::Receiver`] that will receive future events.
    /// Events dispatched before subscription are not replayed.
    pub fn subscribe(&self) -> broadcast::Receiver<ProgressEvent> {
        self.inner.tx.subscribe()
    }

    /// Dispatch a progress event to all current subscribers.
    ///
    /// If the event's token has been cancelled via [`cancel_request`], the
    /// event is silently dropped (E-PRO-006).  No error is returned or
    /// propagated — callers must not rely on knowing whether the event was
    /// delivered.
    ///
    /// If there are no active subscribers, the send is a no-op (the
    /// broadcast channel discards the message).
    pub fn dispatch(&self, event: ProgressEvent) {
        // Orphan / cancellation guard (E-PRO-006).
        let token_key = token_key(&event.token);
        {
            let cancelled = self
                .inner
                .cancelled
                .lock()
                .expect("cancelled lock poisoned");
            if cancelled.contains(&token_key) {
                // Silently drop — no error, no panic.
                return;
            }
        }

        // Best-effort send.  `send` fails only when there are zero receivers;
        // that is the "orphaned" scenario and we intentionally ignore it.
        let _ = self.inner.tx.send(event);
    }

    /// Mark a request token as cancelled.
    ///
    /// After this call, any [`dispatch`] for the same token will be silently
    /// dropped (E-PRO-006).  The actual `$/cancel` notification over the wire
    /// is sent separately via `rmcp::Peer::notify_cancelled()` in the
    /// connection layer; this method only updates the bus-level guard.
    pub fn cancel_request(&self, token: ProgressToken) {
        let key = token_key(&token);
        let mut cancelled = self
            .inner
            .cancelled
            .lock()
            .expect("cancelled lock poisoned");
        cancelled.insert(key);
    }

    /// Returns `true` if the given token has been cancelled.
    pub fn is_cancelled(&self, token: &ProgressToken) -> bool {
        let key = token_key(token);
        let cancelled = self
            .inner
            .cancelled
            .lock()
            .expect("cancelled lock poisoned");
        cancelled.contains(&key)
    }
}

impl Default for ProgressBus {
    fn default() -> Self {
        Self::new()
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Derive a stable string key from a `ProgressToken`.
fn token_key(token: &ProgressToken) -> String {
    let ProgressToken(inner) = token;
    inner.to_string()
}
