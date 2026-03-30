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
        CallToolRequestParams, CallToolResult, GetPromptRequestParams, GetPromptResult,
        ListPromptsResult, ListResourcesResult, ListToolsResult, ServerCapabilities, ServerInfo,
    },
    service::{QuitReason, RunningService},
};

use crate::error::{CoreError, Result};
use crate::events::{MessageCaptured, MessageDirection, capture_message};
use crate::types::{FeatureSet, NegotiatedCapabilities, features_for_version};

// ── Constants ────────────────────────────────────────────────────────────────

/// The MCP spec version that Forge MCP proposes in every `initialize` request.
///
/// Aligned with `rmcp 1.3`'s `ProtocolVersion::LATEST` (`"2025-06-18"`).
pub const MCP_LATEST_VERSION: &str = "2025-06-18";

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
///
/// After the MCP `initialize` / `initialized` handshake completes, the
/// negotiated capabilities are stored here and exposed through the
/// `supports_*` accessor methods. These checks are **pure** — they read the
/// stored data with no I/O.
///
/// ## Version-based degradation (STORY-015)
///
/// In addition to the server-advertised capability flags, `McpConnection` also
/// applies a **version-based feature mask** derived from the server's reported
/// `protocol_version`.  The `supports_*` methods return `true` only when
/// **both** the server capability flag is set **and** the negotiated spec
/// version allows the feature.
///
/// If the negotiated version differs from the client's proposed version,
/// `E-CON-006` is recorded in `version_warning` at construction time.
/// Callers may inspect `version_warning()` to surface this to users.
pub struct McpConnection<H: ClientHandler = ()> {
    service: RunningService<RoleClient, H>,
    state: ConnectionState,
    /// Human-readable label for this connection (URL or "command args").
    label: String,
    /// Which transport backs this connection.
    transport_kind: TransportKind,
    /// The negotiated capabilities from the MCP initialize handshake.
    ///
    /// Populated by `transport.rs` immediately after `serve()` completes.
    /// The `supports_*` methods consult this field; capability guards return
    /// `Err(E-PRO-003)` when the required capability is absent.
    capabilities: NegotiatedCapabilities,
    /// Version-based feature mask derived from `capabilities.protocol_version`.
    ///
    /// Computed once at construction via `features_for_version()`.  Pure field —
    /// no I/O involved.
    version_features: FeatureSet,
    /// If the server's negotiated version differed from the client's proposed
    /// version, this holds the `E-CON-006` warning error (informational only —
    /// the connection is live and using the server-reported version).
    version_warning: Option<CoreError>,
    /// The root URIs that this client advertises to servers via `roots/list`.
    ///
    /// Populated at construction from `ClientCapabilityConfig::root_paths`.
    /// Used by `protocol::list_roots()` to return the current roots without a
    /// network round-trip (roots are a client-side concept).
    pub(crate) root_uris: Vec<String>,
    /// Optional broadcast sender for transparent message capture (STORY-027).
    ///
    /// When `Some`, every JSON-RPC message flowing through this connection
    /// (both directions) is broadcast on this channel.  When `None`, capture
    /// is disabled and no overhead is incurred.
    pub(crate) capture_tx: Option<tokio::sync::broadcast::Sender<MessageCaptured>>,
}

impl<H: ClientHandler> fmt::Debug for McpConnection<H> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("McpConnection")
            .field("state", &self.state)
            .field("label", &self.label)
            .field("transport_kind", &self.transport_kind)
            .field("protocol_version", &self.capabilities.protocol_version)
            .finish_non_exhaustive()
    }
}

impl<H: ClientHandler> McpConnection<H> {
    /// Create a new `McpConnection` from a running service and negotiated capabilities.
    ///
    /// The `proposed_version` parameter is the spec version string that the client
    /// sent in its `initialize` request.  If it differs from
    /// `capabilities.protocol_version` (what the server reported back), an
    /// `E-CON-006` warning is stored in `version_warning`.
    pub(crate) fn new(
        service: RunningService<RoleClient, H>,
        label: impl Into<String>,
        transport_kind: TransportKind,
        capabilities: NegotiatedCapabilities,
    ) -> Self {
        // Derive the version-based feature mask immediately (pure computation).
        let version_features = features_for_version(&capabilities.protocol_version);

        // Record E-CON-006 if the server reported a different version than what
        // we proposed.  The `proposed_version` is not stored on
        // `NegotiatedCapabilities` — transport.rs sets `protocol_version` to the
        // *server-reported* value.  We detect mismatch by comparing against the
        // current MCP_LATEST_VERSION constant.
        let version_warning = if capabilities.protocol_version != MCP_LATEST_VERSION {
            Some(CoreError::ProtocolVersionMismatch {
                proposed: MCP_LATEST_VERSION.to_string(),
                negotiated: capabilities.protocol_version.clone(),
            })
        } else {
            None
        };

        Self {
            service,
            state: ConnectionState::Connected,
            label: label.into(),
            transport_kind,
            capabilities,
            version_features,
            version_warning,
            root_uris: vec![],
            capture_tx: None,
        }
    }

    /// Like `new` but also records the root URIs that will be returned by
    /// `protocol::list_roots()`.
    pub(crate) fn new_with_roots(
        service: RunningService<RoleClient, H>,
        label: impl Into<String>,
        transport_kind: TransportKind,
        capabilities: NegotiatedCapabilities,
        root_uris: Vec<String>,
    ) -> Self {
        let mut conn = Self::new(service, label, transport_kind, capabilities);
        conn.root_uris = root_uris;
        conn
    }

    /// Create a `McpConnection` with an explicit proposed version for mismatch
    /// detection.  This variant is used in tests and by transport code that
    /// knows the exact version string it sent.
    #[allow(dead_code)]
    pub(crate) fn new_with_proposed_version(
        service: RunningService<RoleClient, H>,
        label: impl Into<String>,
        transport_kind: TransportKind,
        capabilities: NegotiatedCapabilities,
        proposed_version: &str,
    ) -> Self {
        let version_features = features_for_version(&capabilities.protocol_version);

        let version_warning = if capabilities.protocol_version != proposed_version {
            Some(CoreError::ProtocolVersionMismatch {
                proposed: proposed_version.to_string(),
                negotiated: capabilities.protocol_version.clone(),
            })
        } else {
            None
        };

        Self {
            service,
            state: ConnectionState::Connected,
            label: label.into(),
            transport_kind,
            capabilities,
            version_features,
            version_warning,
            root_uris: vec![],
            capture_tx: None,
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

    // ── Capability negotiation accessors (pure) ───────────────────────────────

    /// Returns the `ServerCapabilities` the server advertised during `initialize`.
    ///
    /// Callers should prefer the `supports_*` methods for boolean capability checks.
    pub fn server_capabilities(&self) -> &ServerCapabilities {
        &self.capabilities.server
    }

    /// Returns the protocol version string agreed during `initialize`
    /// (e.g. `"2025-11-25"`).
    pub fn protocol_version(&self) -> &str {
        &self.capabilities.protocol_version
    }

    /// Returns the version-based feature mask for this connection.
    ///
    /// Pure accessor — computed at construction from `protocol_version`.
    pub fn version_features(&self) -> &FeatureSet {
        &self.version_features
    }

    /// Returns the `E-CON-006` version mismatch warning if one was recorded,
    /// or `None` if the server's version matched the proposed version.
    ///
    /// This is purely informational — the connection is live regardless.
    pub fn version_warning(&self) -> Option<&CoreError> {
        self.version_warning.as_ref()
    }

    /// Returns `true` if there was a protocol version mismatch between what
    /// the client proposed and what the server reported.
    pub fn has_version_mismatch(&self) -> bool {
        self.version_warning.is_some()
    }

    /// Returns the `file://` URIs of the root paths this client is configured
    /// to advertise to servers via `roots/list`.
    ///
    /// Returns an empty slice when no root paths were configured or when the
    /// `roots` capability was not advertised.
    ///
    /// Used by [`crate::protocol::list_roots`].
    pub fn root_uris(&self) -> Vec<String> {
        self.root_uris.clone()
    }

    /// Returns `true` if the server advertised the `tools` capability
    /// **and** the negotiated spec version supports tools.
    ///
    /// Pure check — no I/O.
    pub fn supports_tools(&self) -> bool {
        self.capabilities.server.tools.is_some() && self.version_features.tools
    }

    /// Returns `true` if the server advertised the `resources` capability
    /// **and** the negotiated spec version supports resources.
    pub fn supports_resources(&self) -> bool {
        self.capabilities.server.resources.is_some() && self.version_features.resources
    }

    /// Returns `true` if the server advertised the `prompts` capability
    /// **and** the negotiated spec version supports prompts.
    pub fn supports_prompts(&self) -> bool {
        self.capabilities.server.prompts.is_some() && self.version_features.prompts
    }

    /// Returns `true` if the server advertised the `sampling` client-side capability.
    ///
    /// Note: `sampling` is a *client* capability (servers call `sampling/createMessage`
    /// on clients that advertise it). This method checks whether *we* advertised
    /// sampling support to the server during `initialize`.
    pub fn supports_sampling(&self) -> bool {
        self.capabilities.client.sampling.is_some()
    }

    /// Returns `true` if we advertised the `elicitation` client capability.
    ///
    /// `elicitation` is a *client* capability — servers call `elicitation/create`
    /// on clients that advertise it. This checks whether *we* advertised it,
    /// **and** the negotiated spec version supports elicitation.
    pub fn supports_elicitation(&self) -> bool {
        self.capabilities.client.elicitation.is_some() && self.version_features.elicitation
    }

    /// Returns `true` if we advertised the `roots` client capability.
    ///
    /// `roots` is a *client* capability — servers call `roots/list` on clients
    /// that advertise it. This checks whether *we* advertised it.
    pub fn supports_roots(&self) -> bool {
        self.capabilities.client.roots.is_some()
    }

    /// Returns `true` if the server advertised the `logging` capability
    /// **and** the negotiated spec version supports logging.
    pub fn supports_logging(&self) -> bool {
        self.capabilities.server.logging.is_some() && self.version_features.logging
    }

    /// Returns `true` if the negotiated spec version supports streamable-HTTP transport.
    ///
    /// Streamable HTTP is only available in spec versions `2025-11-25` and later.
    pub fn supports_streamable_http(&self) -> bool {
        self.version_features.streamable_http
    }

    // ── Queries ───────────────────────────────────────────────────────────────

    /// List the tools available on the connected server.
    ///
    /// # Errors
    /// Returns `Err(E-PRO-003)` immediately (no network round-trip) if the
    /// server did not advertise the `tools` capability during `initialize`, or
    /// if the negotiated spec version does not support tools.
    pub async fn list_tools(&self) -> Result<ListToolsResult> {
        if !self.supports_tools() {
            return Err(CoreError::CapabilityNotSupported {
                method: "tools/list".to_string(),
                capability: "tools".to_string(),
            });
        }
        // AC-001: capture outbound request.
        if let Some(tx) = &self.capture_tx {
            capture_message(
                tx,
                MessageDirection::ClientToServer,
                Some("tools/list".to_string()),
                serde_json::json!({"jsonrpc":"2.0","method":"tools/list","params":null}),
            );
        }
        let result = self
            .service
            .peer()
            .list_tools(None)
            .await
            .map_err(|e| CoreError::Protocol(e.to_string()))?;
        // AC-001: capture inbound response.
        if let Some(tx) = &self.capture_tx {
            capture_message(
                tx,
                MessageDirection::ServerToClient,
                None,
                serde_json::to_value(&result).unwrap_or(serde_json::Value::Null),
            );
        }
        Ok(result)
    }

    /// List the resources available on the connected server.
    ///
    /// # Errors
    /// Returns `Err(E-PRO-003)` if the server did not advertise `resources`, or
    /// if the negotiated spec version does not support resources.
    pub async fn list_resources(&self) -> Result<ListResourcesResult> {
        if !self.supports_resources() {
            return Err(CoreError::CapabilityNotSupported {
                method: "resources/list".to_string(),
                capability: "resources".to_string(),
            });
        }
        // AC-001: capture outbound request.
        if let Some(tx) = &self.capture_tx {
            capture_message(
                tx,
                MessageDirection::ClientToServer,
                Some("resources/list".to_string()),
                serde_json::json!({"jsonrpc":"2.0","method":"resources/list","params":null}),
            );
        }
        let result = self
            .service
            .peer()
            .list_resources(None)
            .await
            .map_err(|e| CoreError::Protocol(e.to_string()))?;
        // AC-001: capture inbound response.
        if let Some(tx) = &self.capture_tx {
            capture_message(
                tx,
                MessageDirection::ServerToClient,
                None,
                serde_json::to_value(&result).unwrap_or(serde_json::Value::Null),
            );
        }
        Ok(result)
    }

    /// List the prompts available on the connected server.
    ///
    /// # Errors
    /// Returns `Err(E-PRO-003)` if the server did not advertise `prompts`, or
    /// if the negotiated spec version does not support prompts.
    pub async fn list_prompts(&self) -> Result<ListPromptsResult> {
        if !self.supports_prompts() {
            return Err(CoreError::CapabilityNotSupported {
                method: "prompts/list".to_string(),
                capability: "prompts".to_string(),
            });
        }
        // AC-001: capture outbound request.
        if let Some(tx) = &self.capture_tx {
            capture_message(
                tx,
                MessageDirection::ClientToServer,
                Some("prompts/list".to_string()),
                serde_json::json!({"jsonrpc":"2.0","method":"prompts/list","params":null}),
            );
        }
        let result = self
            .service
            .peer()
            .list_prompts(None)
            .await
            .map_err(|e| CoreError::Protocol(e.to_string()))?;
        // AC-001: capture inbound response.
        if let Some(tx) = &self.capture_tx {
            capture_message(
                tx,
                MessageDirection::ServerToClient,
                None,
                serde_json::to_value(&result).unwrap_or(serde_json::Value::Null),
            );
        }
        Ok(result)
    }

    /// Retrieve a prompt by name with optional arguments.
    ///
    /// # Errors
    /// Returns `Err(E-PRO-003)` immediately (no network round-trip) if the
    /// server did not advertise the `prompts` capability during `initialize`.
    /// Returns `Err(E-PRO-001)` on transport / protocol errors.
    pub async fn get_prompt(
        &self,
        name: impl Into<std::borrow::Cow<'static, str>>,
        arguments: Option<serde_json::Value>,
    ) -> Result<GetPromptResult> {
        if !self.supports_prompts() {
            return Err(CoreError::CapabilityNotSupported {
                method: "prompts/get".to_string(),
                capability: "prompts".to_string(),
            });
        }

        let name_str = name.into().into_owned();
        let params = match arguments {
            Some(serde_json::Value::Object(map)) => {
                GetPromptRequestParams::new(name_str.clone()).with_arguments(map)
            }
            _ => GetPromptRequestParams::new(name_str.clone()),
        };

        // AC-001: capture outbound request.
        if let Some(tx) = &self.capture_tx {
            capture_message(
                tx,
                MessageDirection::ClientToServer,
                Some("prompts/get".to_string()),
                serde_json::json!({"jsonrpc":"2.0","method":"prompts/get","params":{"name": name_str}}),
            );
        }
        let result = self
            .service
            .peer()
            .get_prompt(params)
            .await
            .map_err(|e| CoreError::Protocol(e.to_string()))?;
        // AC-001: capture inbound response.
        if let Some(tx) = &self.capture_tx {
            capture_message(
                tx,
                MessageDirection::ServerToClient,
                None,
                serde_json::to_value(&result).unwrap_or(serde_json::Value::Null),
            );
        }
        Ok(result)
    }

    /// Guard for `elicitation/create` — returns `Err(E-PRO-003)` if the
    /// negotiated spec version is older than `2025-11-25`.
    ///
    /// Callers should call this before attempting any `elicitation/create`
    /// RPC to receive an immediate, descriptive error instead of a protocol
    /// failure from the server.
    pub fn guard_elicitation(&self) -> Result<()> {
        if !self.supports_elicitation() {
            return Err(CoreError::CapabilityNotSupported {
                method: "elicitation/create".to_string(),
                capability: format!(
                    "elicitation (requires spec ≥ 2025-06-18; negotiated {})",
                    self.capabilities.protocol_version
                ),
            });
        }
        Ok(())
    }

    /// Call a tool on the connected server.
    pub async fn call_tool(
        &self,
        name: impl Into<std::borrow::Cow<'static, str>>,
        arguments: Option<serde_json::Value>,
    ) -> Result<CallToolResult> {
        let name_cow: std::borrow::Cow<'static, str> = name.into();
        let params = match &arguments {
            Some(serde_json::Value::Object(map)) => {
                CallToolRequestParams::new(name_cow.clone()).with_arguments(map.clone())
            }
            _ => CallToolRequestParams::new(name_cow.clone()),
        };

        // AC-001: capture outbound request.
        if let Some(tx) = &self.capture_tx {
            capture_message(
                tx,
                MessageDirection::ClientToServer,
                Some("tools/call".to_string()),
                serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "tools/call",
                    "params": {"name": name_cow.as_ref(), "arguments": arguments}
                }),
            );
        }
        let result = self
            .service
            .peer()
            .call_tool(params)
            .await
            .map_err(|e| CoreError::Protocol(e.to_string()))?;
        // AC-001: capture inbound response.
        if let Some(tx) = &self.capture_tx {
            capture_message(
                tx,
                MessageDirection::ServerToClient,
                None,
                serde_json::to_value(&result).unwrap_or(serde_json::Value::Null),
            );
        }
        Ok(result)
    }

    // ── Message capture (STORY-027) ───────────────────────────────────────────

    /// Install a capture channel on this connection.
    ///
    /// After calling this, every JSON-RPC message flowing through the connection
    /// (both directions) will be broadcast on `tx`.
    pub fn install_capture_hook(&mut self, tx: tokio::sync::broadcast::Sender<MessageCaptured>) {
        self.capture_tx = Some(tx);
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
    #![allow(non_snake_case)]
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
