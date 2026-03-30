//! `McpConnection` — a live connection to an MCP server.
//!
//! ## Connection State Machine (pure core)
//!
//! ```text
//! Disconnected → Connecting → Connected
//!                          ↘
//!                           Error
//! ```
//!
//! The state machine itself is pure: transitions are functions over `ConnectionState`
//! that take no I/O arguments.  The effectful layer (`transport.rs`) drives the
//! transitions.

use rmcp::model::{ServerCapabilities, ServerInfo};
use rmcp::service::RunningService;
use rmcp::{Peer, RoleClient};

use crate::error::{ForgeError, Result};

// ── Connection state (pure) ──────────────────────────────────────────────────

/// The lifecycle state of an MCP connection.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConnectionState {
    /// No connection attempt has been made yet.
    Disconnected,
    /// A connection attempt is in progress (subprocess spawned, waiting for initialize).
    Connecting,
    /// The initialize handshake completed successfully.
    Connected,
    /// The connection failed or was lost.
    Error(String),
}

impl ConnectionState {
    /// Pure state transition: Disconnected → Connecting.
    pub fn start_connecting(self) -> Self {
        match self {
            ConnectionState::Disconnected => ConnectionState::Connecting,
            other => other, // no-op if already connecting/connected/errored
        }
    }

    /// Pure state transition: Connecting → Connected.
    pub fn mark_connected(self) -> Self {
        match self {
            ConnectionState::Connecting => ConnectionState::Connected,
            other => other,
        }
    }

    /// Pure state transition: any state → Error.
    pub fn mark_error(self, reason: impl Into<String>) -> Self {
        ConnectionState::Error(reason.into())
    }

    /// Returns `true` if the connection is currently live.
    pub fn is_connected(&self) -> bool {
        matches!(self, ConnectionState::Connected)
    }
}

// ── Transport type ───────────────────────────────────────────────────────────

/// Which transport this connection uses.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TransportKind {
    /// JSON-RPC over child-process stdin/stdout.
    Stdio,
    /// JSON-RPC over HTTP (streamable-HTTP / SSE).
    Http,
}

// ── McpConnection ────────────────────────────────────────────────────────────

/// A live, typed handle to an MCP server connection.
///
/// Wraps rmcp's `RunningService` and exposes Forge-specific metadata.
/// On drop, rmcp's `DropGuard` cancels the service background task and the
/// `TokioChildProcess`'s `ChildWithCleanup` kills the subprocess.
pub struct McpConnection {
    /// Human-readable name of the server (`ServerInfo.server_info.name`).
    server_name: String,
    /// Version string from `ServerInfo`.
    server_version: String,
    /// Raw capabilities + info returned during the initialize handshake.
    server_info: ServerInfo,
    /// Which transport backs this connection.
    transport_kind: TransportKind,
    /// Current lifecycle state.
    state: ConnectionState,
    /// The underlying rmcp running service.
    ///
    /// `Option` so we can take ownership for graceful shutdown without dropping `self`.
    running: Option<RunningService<RoleClient, ()>>,
}

impl std::fmt::Debug for McpConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpConnection")
            .field("server_name", &self.server_name)
            .field("server_version", &self.server_version)
            .field("transport_kind", &self.transport_kind)
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

impl McpConnection {
    /// Create a `McpConnection` from an already-established rmcp service.
    ///
    /// Called by `transport::connect_stdio` after a successful initialize.
    pub(crate) fn from_running_service(
        running: RunningService<RoleClient, ()>,
        transport_kind: TransportKind,
    ) -> Self {
        // `peer_info()` returns `Option<&ServerInfo>`.  After a successful
        // `serve_client` the info is always populated; fall back to a synthetic
        // default in the unlikely case it is missing.
        let server_info: ServerInfo = running
            .peer_info()
            .cloned()
            .unwrap_or_else(|| ServerInfo::new(ServerCapabilities::default()));

        let server_name = server_info.server_info.name.clone();
        let server_version = server_info.server_info.version.clone();

        Self {
            server_name,
            server_version,
            server_info,
            transport_kind,
            state: ConnectionState::Connected,
            running: Some(running),
        }
    }

    // ── Accessors ────────────────────────────────────────────────────────────

    /// The MCP server's self-reported name.
    pub fn server_name(&self) -> &str {
        &self.server_name
    }

    /// The MCP server's self-reported version string.
    pub fn server_version(&self) -> &str {
        &self.server_version
    }

    /// Full `ServerInfo` returned during the initialize handshake.
    pub fn server_info(&self) -> &ServerInfo {
        &self.server_info
    }

    /// Which transport backs this connection.
    pub fn transport_kind(&self) -> &TransportKind {
        &self.transport_kind
    }

    /// Current connection lifecycle state.
    pub fn state(&self) -> &ConnectionState {
        &self.state
    }

    /// Returns a reference to the rmcp `Peer`, if still connected.
    pub fn peer(&self) -> Option<&Peer<RoleClient>> {
        self.running.as_ref().map(|r| r.peer())
    }

    // ── Lifecycle ────────────────────────────────────────────────────────────

    /// Gracefully shut down the connection.
    ///
    /// Cancels the rmcp service (which closes the transport and kills the child
    /// process in the `TokioChildProcess` transport), then waits for the
    /// background task to finish.
    pub async fn shutdown(mut self) -> Result<()> {
        if let Some(mut running) = self.running.take() {
            running
                .close()
                .await
                .map_err(|_| ForgeError::ServerProcessExited { code: None })?;
        }
        Ok(())
    }

    /// Returns `true` if the underlying rmcp service reports it is closed.
    pub fn is_closed(&self) -> bool {
        self.running
            .as_ref()
            .map(|r| r.is_closed())
            .unwrap_or(true)
    }
}

// ── Pure state machine tests ─────────────────────────────────────────────────

#[cfg(test)]
mod state_machine_tests {
    use super::*;

    #[test]
    fn test_disconnected_to_connecting() {
        let s = ConnectionState::Disconnected;
        assert_eq!(s.start_connecting(), ConnectionState::Connecting);
    }

    #[test]
    fn test_connecting_to_connected() {
        let s = ConnectionState::Connecting;
        assert_eq!(s.mark_connected(), ConnectionState::Connected);
    }

    #[test]
    fn test_connected_to_error() {
        let s = ConnectionState::Connected;
        assert_eq!(
            s.mark_error("oops"),
            ConnectionState::Error("oops".to_string())
        );
    }

    #[test]
    fn test_disconnected_stays_disconnected_on_mark_connected() {
        // Only Connecting → Connected; Disconnected stays Disconnected.
        let s = ConnectionState::Disconnected;
        assert_eq!(s.mark_connected(), ConnectionState::Disconnected);
    }

    #[test]
    fn test_is_connected_true_only_when_connected() {
        assert!(!ConnectionState::Disconnected.is_connected());
        assert!(!ConnectionState::Connecting.is_connected());
        assert!(ConnectionState::Connected.is_connected());
        assert!(!ConnectionState::Error("x".into()).is_connected());
    }
}
