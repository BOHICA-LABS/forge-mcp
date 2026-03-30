//! Protocol utility functions — `list_roots`, `set_log_level`, `complete`.
//!
//! This module is **effectful** — every function performs a network round-trip
//! over the supplied `McpConnection`.  All capability guards are applied before
//! any I/O so callers get a fast-fail on missing capabilities.
//!
//! ## AC coverage
//! | AC  | Function |
//! |-----|---------|
//! | AC-001 | [`list_roots`] |
//! | AC-002 | [`set_log_level`] |
//! | AC-003 | [`complete`] |
//! | AC-004 | Log message routing is handled by [`ForgeClientHandler::on_logging_message`] in `handler.rs` |

use rmcp::{
    ClientHandler,
    model::{
        ArgumentInfo, CompleteRequestParams, CompleteResult, LoggingLevel, Reference,
        Root, SetLevelRequestParams,
    },
};

use crate::connection::McpConnection;
use crate::error::{CoreError, Result};

// ── AC-001: roots management ──────────────────────────────────────────────────

/// Return the client's configured root paths.
///
/// This is a **client-side** operation — it reads the root paths that the
/// `ForgeClientHandler` would return to a server calling `roots/list`, but
/// returns them directly to the caller without any network round-trip.
///
/// The roots are derived from the `ClientCapabilityConfig::root_paths` that
/// the handler was constructed with.
///
/// # Errors
/// Returns `Err(E-PRO-003)` if the connection did not advertise the `roots`
/// client capability during the MCP `initialize` handshake.
pub fn list_roots<H: ClientHandler>(conn: &McpConnection<H>) -> Result<Vec<Root>> {
    if !conn.supports_roots() {
        return Err(CoreError::CapabilityNotSupported {
            method: "roots/list".to_string(),
            capability: "roots".to_string(),
        });
    }
    // The roots are available on the handler's config; we synthesise them
    // from the root_uris stored on the connection's handler config.
    // Since McpConnection doesn't directly expose the handler, we expose
    // the root_uris via `root_uris()` below in the helper.
    Ok(conn.root_uris()
        .into_iter()
        .map(Root::new)
        .collect())
}

/// Send a `notifications/roots/list_changed` notification to the server.
///
/// Clients send this notification when their configured root paths change so
/// the server can re-issue `roots/list` to retrieve the updated list.
///
/// # Errors
/// Returns `Err(E-PRO-003)` if the connection did not advertise the `roots`
/// client capability.
/// Returns `Err(E-PRO-001)` if the notification could not be sent (transport
/// error).
pub async fn notify_roots_list_changed<H: ClientHandler>(
    conn: &McpConnection<H>,
) -> Result<()> {
    if !conn.supports_roots() {
        return Err(CoreError::CapabilityNotSupported {
            method: "notifications/roots/list_changed".to_string(),
            capability: "roots".to_string(),
        });
    }
    let peer = conn.peer().ok_or_else(|| CoreError::Protocol("no peer".to_string()))?;
    peer.notify_roots_list_changed()
        .await
        .map_err(|e| CoreError::Protocol(e.to_string()))
}

// ── AC-002: logging level control ────────────────────────────────────────────

/// Send a `logging/setLevel` request to the server.
///
/// After this call, the server will send `notifications/message` entries at or
/// above `level` to the client via the `on_logging_message` handler (see
/// `ForgeClientHandler` in `handler.rs`).
///
/// # Errors
/// Returns `Err(E-PRO-003)` if the server did not advertise the `logging`
/// capability during the MCP `initialize` handshake.
/// Returns `Err(E-PRO-001)` on transport error.
pub async fn set_log_level<H: ClientHandler>(
    conn: &McpConnection<H>,
    level: LoggingLevel,
) -> Result<()> {
    if !conn.supports_logging() {
        return Err(CoreError::CapabilityNotSupported {
            method: "logging/setLevel".to_string(),
            capability: "logging".to_string(),
        });
    }
    let peer = conn.peer().ok_or_else(|| CoreError::Protocol("no peer".to_string()))?;
    peer.set_level(SetLevelRequestParams::new(level))
        .await
        .map_err(|e| CoreError::Protocol(e.to_string()))
}

// ── AC-003: completion ───────────────────────────────────────────────────────

/// Send a `completion/complete` request and return the suggestions.
///
/// Returns suggested completions for `argument` in the context of `reference`
/// (either a prompt name or a resource URI template).
///
/// # Arguments
/// * `reference` — the prompt or resource being completed
/// * `argument`  — the argument being completed (name + partial value)
///
/// # Errors
/// Returns `Err(E-PRO-001)` on transport error (no capability guard — the
/// MCP spec allows any server to handle completion; capability is optional).
pub async fn complete<H: ClientHandler>(
    conn: &McpConnection<H>,
    reference: Reference,
    argument: ArgumentInfo,
) -> Result<CompleteResult> {
    let peer = conn.peer().ok_or_else(|| CoreError::Protocol("no peer".to_string()))?;
    let params = CompleteRequestParams::new(reference, argument);
    peer.complete(params)
        .await
        .map_err(|e| CoreError::Protocol(e.to_string()))
}
