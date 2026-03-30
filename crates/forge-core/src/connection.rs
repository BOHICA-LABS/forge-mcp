//! `McpConnection` — transport-agnostic connection wrapper.
//!
//! This module defines the `McpConnection` type that wraps an rmcp
//! `RunningService` and exposes a Forge-specific API for interacting with
//! a connected MCP server.
//!
//! ## Connection State Machine (pure core)
//!
//! ```text
//! Disconnected → Connecting → Connected
//!                           ↘
//!                            Error
//! ```
//!
//! The state machine itself is pure: transitions are functions over `ConnectionState`
//! that take no I/O arguments.  The effectful layer (`transport.rs`) drives the
//! transitions.

use std::fmt;

use rmcp::{
    ClientHandler, Peer, RoleClient,
    model::{
        CallToolRequestParams, CallToolResult, ListPromptsResult, ListResourcesResult,
        ListToolsResult, ServerInfo,
    },
    service::{QuitReason, RunningService},
};

use crate::error::{CoreError, Result};

// ── Connection state (pure) ──────────────────────────────────────────────────

/// The lifecycle state of an MCP connection.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ConnectionState {
    /// No connection attempt has been made yet.
    Disconnected,
    /// A connection attempt is in progress (subprocess spawned / HTTP request sent).
    Connecting,
    /// The initialize handshake completed successfully.
    Connected,
    /// The connection is being torn down gracefully.
    Disconnecting,
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

    /// Pure state transition: Connected → Disconnecting.
    pub fn start_disconnecting(self) -> Self {
        match self {
            ConnectionState::Connected => ConnectionState::Disconnecting,
            other => other,
        }
    }

    /// Pure state transition: Disconnecting → Disconnected.
    pub fn mark_disconnected(self) -> Self {
        match self {
            ConnectionState::Disconnecting => ConnectionState::Disconnected,
            other => other,
        }
    }

    /// Pure state transition: any state → Error.
    pub fn mark_error(self, reason: impl Into<String>) -> Self {
        ConnectionState::Error(reason.into())
    }

    /// Pure state transition: Error → Connecting (reconnect).
    pub fn start_reconnecting(self) -> Self {
        match self {
            ConnectionState::Error(_) => ConnectionState::Connecting,
            other => other,
        }
    }

    /// Returns `true` if the connection is currently live.
    pub fn is_connected(&self) -> bool {
        matches!(self, ConnectionState::Connected)
    }
}

impl fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Disconnected => write!(f, "Disconnected"),
            Self::Connecting => write!(f, "Connecting"),
            Self::Connected => write!(f, "Connected"),
            Self::Disconnecting => write!(f, "Disconnecting"),
            Self::Error(msg) => write!(f, "Error({msg})"),
        }
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
/// On drop, rmcp's `DropGuard` cancels the service background task.
pub struct McpConnection<H: ClientHandler = ()> {
    service: RunningService<RoleClient, H>,
    state: ConnectionState,
    /// Human-readable label for this connection (URL or "command args").
    label: String,
    /// Which transport backs this connection.
    transport_kind: TransportKind,
}

impl<H: ClientHandler> fmt::Debug for McpConnection<H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("McpConnection")
            .field("state", &self.state)
            .field("label", &self.label)
            .field("transport_kind", &self.transport_kind)
            .finish_non_exhaustive()
    }
}

impl<H: ClientHandler> McpConnection<H> {
    /// Create a new `McpConnection` from a running service.
    pub(crate) fn new(
        service: RunningService<RoleClient, H>,
        label: impl Into<String>,
        transport_kind: TransportKind,
    ) -> Self {
        Self {
            service,
            state: ConnectionState::Connected,
            label: label.into(),
            transport_kind,
        }
    }

    // ── Accessors ────────────────────────────────────────────────────────────

    /// Returns the current connection state.
    pub fn state(&self) -> &ConnectionState {
        &self.state
    }

    /// Returns the human-readable label for this connection.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Which transport backs this connection.
    pub fn transport_kind(&self) -> &TransportKind {
        &self.transport_kind
    }

    /// Returns a reference to the server info reported during the handshake.
    pub fn server_info(&self) -> Option<&ServerInfo> {
        self.service.peer().peer_info()
    }

    /// The MCP server's self-reported name (convenience accessor).
    pub fn server_name(&self) -> &str {
        self.service
            .peer()
            .peer_info()
            .map(|i| i.server_info.name.as_str())
            .unwrap_or("")
    }

    /// The MCP server's self-reported version (convenience accessor).
    pub fn server_version(&self) -> &str {
        self.service
            .peer()
            .peer_info()
            .map(|i| i.server_info.version.as_str())
            .unwrap_or("")
    }

    /// Returns a reference to the rmcp `Peer`.
    pub fn peer(&self) -> Option<&Peer<RoleClient>> {
        Some(self.service.peer())
    }

    /// Returns `true` if the underlying rmcp service reports it is closed.
    pub fn is_closed(&self) -> bool {
        self.service.is_closed()
    }

    // ── Queries ───────────────────────────────────────────────────────────────

    /// List the tools available on the connected server.
    pub async fn list_tools(&self) -> Result<ListToolsResult> {
        self.service
            .peer()
            .list_tools(None)
            .await
            .map_err(|e| CoreError::Protocol(e.to_string()))
    }

    /// List the resources available on the connected server.
    pub async fn list_resources(&self) -> Result<ListResourcesResult> {
        self.service
            .peer()
            .list_resources(None)
            .await
            .map_err(|e| CoreError::Protocol(e.to_string()))
    }

    /// List the prompts available on the connected server.
    pub async fn list_prompts(&self) -> Result<ListPromptsResult> {
        self.service
            .peer()
            .list_prompts(None)
            .await
            .map_err(|e| CoreError::Protocol(e.to_string()))
    }

    /// Call a tool on the connected server.
    pub async fn call_tool(
        &self,
        name: impl Into<std::borrow::Cow<'static, str>>,
        arguments: Option<serde_json::Value>,
    ) -> Result<CallToolResult> {
        let params = match arguments {
            Some(serde_json::Value::Object(map)) => {
                CallToolRequestParams::new(name).with_arguments(map)
            }
            _ => CallToolRequestParams::new(name),
        };

        self.service
            .peer()
            .call_tool(params)
            .await
            .map_err(|e| CoreError::Protocol(e.to_string()))
    }

    // ── Lifecycle ────────────────────────────────────────────────────────────

    /// Gracefully close the connection and wait for cleanup.
    pub async fn close(mut self) -> std::result::Result<QuitReason, tokio::task::JoinError> {
        self.state = ConnectionState::Disconnecting;
        self.service.close().await
    }

    /// Gracefully shut down the connection.
    ///
    /// Alias for [`close`] that returns a `CoreError` on failure — matches
    /// the STORY-007 API used by existing test code.
    pub async fn shutdown(mut self) -> crate::error::Result<()> {
        self.service
            .close()
            .await
            .map(|_| ())
            .map_err(|e| crate::error::CoreError::Protocol(e.to_string()))
    }

    /// Cancel the connection without waiting for cleanup.
    pub fn cancel(self) {
        self.service.cancellation_token().cancel();
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

    #[test]
    fn test_BC_1_02_003_state_machine_valid_transitions() {
        // Disconnected → Connecting → Connected → Disconnecting → Disconnected
        let s = ConnectionState::Disconnected.start_connecting();
        assert_eq!(s, ConnectionState::Connecting);
        let s = s.mark_connected();
        assert_eq!(s, ConnectionState::Connected);
        let s = s.start_disconnecting();
        assert_eq!(s, ConnectionState::Disconnecting);
        let s = s.mark_disconnected();
        assert_eq!(s, ConnectionState::Disconnected);
    }

    #[test]
    fn test_BC_1_02_003_reconnect_from_error() {
        // Error → Connecting (reconnect path)
        let s = ConnectionState::Connected.mark_error("oops");
        assert!(matches!(s, ConnectionState::Error(_)));
        let s = s.start_reconnecting();
        assert_eq!(s, ConnectionState::Connecting);
    }
}
