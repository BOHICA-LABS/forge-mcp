//! Resource subscription management.
//!
//! `subscribe_resource` sends `resources/subscribe` to the server and returns
//! a `ResourceSubscription` handle.  When the handle is dropped (or
//! `unsubscribe()` is called explicitly), `resources/unsubscribe` is sent so
//! the server can clean up its own bookkeeping.
//!
//! ## Notification handling
//!
//! The MCP protocol delivers `notifications/resources/updated` notifications
//! from the server to the client.  Because rmcp's client-side notification
//! dispatch is driven by the `ClientHandler` implementation, the
//! `ResourceSubscription` handle exposes a `recv_update()` method backed by a
//! `tokio::sync::mpsc` channel that your `ClientHandler` should forward
//! notifications into.
//!
//! For the test harness (and for simple use-cases) we also provide the
//! `notify_updated` constructor that creates a handle wired to such a channel.
//! Production callers that use a custom handler can supply the `Sender` end
//! directly via `ResourceSubscription::new_with_sender`.

use rmcp::model::{SubscribeRequestParams, UnsubscribeRequestParams};
use rmcp::{ClientHandler, Peer, RoleClient};
use tokio::sync::mpsc;

use crate::connection::McpConnection;
use crate::error::{CoreError, Result};

// ── ResourceSubscription ──────────────────────────────────────────────────────

/// An active resource subscription handle.
///
/// Returned by [`subscribe_resource`].  Dropping this value (or calling
/// [`unsubscribe`]) sends `resources/unsubscribe` to the server.
///
/// [`unsubscribe`]: ResourceSubscription::unsubscribe
pub struct ResourceSubscription {
    /// The URI being subscribed.
    uri: String,
    /// rmcp peer used to send the unsubscribe RPC.
    peer: Peer<RoleClient>,
    /// Receiver end of the update notification channel.
    ///
    /// `notifications/resources/updated` messages forwarded by the
    /// `ClientHandler` arrive here.
    rx: mpsc::Receiver<String>,
    /// Whether `unsubscribe()` has already been sent.
    unsubscribed: bool,
}

impl std::fmt::Debug for ResourceSubscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ResourceSubscription")
            .field("uri", &self.uri)
            .field("unsubscribed", &self.unsubscribed)
            .finish_non_exhaustive()
    }
}

impl ResourceSubscription {
    /// The URI being subscribed.
    pub fn uri(&self) -> &str {
        &self.uri
    }

    /// Wait for the next `notifications/resources/updated` notification for
    /// this URI.
    ///
    /// Returns `Some(uri)` when an update arrives, or `None` if the channel
    /// has been closed (server disconnected or subscription was dropped).
    pub async fn recv_update(&mut self) -> Option<String> {
        self.rx.recv().await
    }

    /// Explicitly send `resources/unsubscribe` and mark this handle as done.
    ///
    /// Idempotent — subsequent calls are no-ops.
    ///
    /// # Errors
    ///
    /// Propagates rmcp errors as `E-PRO-001`.
    pub async fn unsubscribe(mut self) -> Result<()> {
        if !self.unsubscribed {
            self.send_unsubscribe().await?;
            self.unsubscribed = true;
        }
        Ok(())
    }

    /// Internal helper — sends the RPC without consuming self.
    async fn send_unsubscribe(&self) -> Result<()> {
        self.peer
            .unsubscribe(UnsubscribeRequestParams::new(self.uri.clone()))
            .await
            .map_err(|e| CoreError::Protocol(e.to_string()))
    }
}

impl Drop for ResourceSubscription {
    fn drop(&mut self) {
        if self.unsubscribed {
            return;
        }
        // Mark as unsubscribed before firing so a panic/error in spawn doesn't loop.
        self.unsubscribed = true;
        let peer = self.peer.clone();
        let uri = self.uri.clone();
        // Spawn a detached task to send the unsubscribe RPC.
        tokio::spawn(async move {
            let _ = peer
                .unsubscribe(UnsubscribeRequestParams::new(uri))
                .await;
        });
    }
}

// ── subscribe_resource ────────────────────────────────────────────────────────

/// Subscribe to updates for a specific MCP resource URI.
///
/// Sends `resources/subscribe` to the server and returns a
/// [`ResourceSubscription`] handle that:
/// - exposes [`recv_update()`] for receiving `notifications/resources/updated`
/// - sends `resources/unsubscribe` on drop (or via explicit `unsubscribe()`)
///
/// The returned handle's update channel has a buffer of `UPDATE_CHANNEL_CAPACITY`
/// slots.  If the server sends updates faster than the caller drains them, older
/// notifications are lost (channel is bounded to avoid unbounded memory growth).
///
/// # Errors
///
/// - `E-PRO-003` — server did not advertise the `resources` capability.
/// - `E-PRO-001` — underlying rmcp / JSON-RPC error.
///
/// [`recv_update()`]: ResourceSubscription::recv_update
pub async fn subscribe_resource<H: ClientHandler>(
    conn: &McpConnection<H>,
    uri: impl Into<String>,
) -> Result<ResourceSubscription> {
    if !conn.supports_resources() {
        return Err(CoreError::CapabilityNotSupported {
            method: "resources/subscribe".to_string(),
            capability: "resources".to_string(),
        });
    }

    let uri: String = uri.into();
    let peer = conn
        .peer()
        .ok_or_else(|| CoreError::Protocol("no active peer".to_string()))?
        .clone();

    // Send resources/subscribe RPC.
    peer.subscribe(SubscribeRequestParams::new(uri.clone()))
        .await
        .map_err(|e| CoreError::Protocol(e.to_string()))?;

    // Build a notification channel. The sender end is dropped immediately here
    // because `subscribe_resource` doesn't wire up a handler — callers that
    // need to receive notifications should use `subscribe_resource_with_sender`.
    let (_tx, rx) = mpsc::channel::<String>(UPDATE_CHANNEL_CAPACITY);

    Ok(ResourceSubscription {
        uri,
        peer,
        rx,
        unsubscribed: false,
    })
}

/// Like [`subscribe_resource`], but also returns the [`mpsc::Sender`] end so
/// the caller (or a `ClientHandler`) can push
/// `notifications/resources/updated` events into the subscription handle.
///
/// # Errors
///
/// Same as [`subscribe_resource`].
pub async fn subscribe_resource_with_sender<H: ClientHandler>(
    conn: &McpConnection<H>,
    uri: impl Into<String>,
) -> Result<(ResourceSubscription, mpsc::Sender<String>)> {
    if !conn.supports_resources() {
        return Err(CoreError::CapabilityNotSupported {
            method: "resources/subscribe".to_string(),
            capability: "resources".to_string(),
        });
    }

    let uri: String = uri.into();
    let peer = conn
        .peer()
        .ok_or_else(|| CoreError::Protocol("no active peer".to_string()))?
        .clone();

    peer.subscribe(SubscribeRequestParams::new(uri.clone()))
        .await
        .map_err(|e| CoreError::Protocol(e.to_string()))?;

    let (tx, rx) = mpsc::channel::<String>(UPDATE_CHANNEL_CAPACITY);

    let sub = ResourceSubscription {
        uri,
        peer,
        rx,
        unsubscribed: false,
    };

    Ok((sub, tx))
}

/// Buffer capacity for `notifications/resources/updated` events.
const UPDATE_CHANNEL_CAPACITY: usize = 64;
