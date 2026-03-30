//! `ForgeClientHandler` — configurable MCP client handler.
//!
//! This module implements the `rmcp::ClientHandler` trait for Forge MCP.
//!
//! ## Design
//!
//! `ForgeClientHandler` wraps a `ClientCapabilityConfig` to:
//! - Build the correct `ClientInfo` (capabilities) based on runtime config
//! - Handle `list_roots` requests by returning configured root paths
//! - Handle `create_message` (sampling) as a stub — returns `E-PRO-007`
//!   until a real LLM proxy integration is wired in
//! - Handle `create_elicitation` as a stub — returns decline in non-interactive
//!   mode per DEC-017 / EC-002

use std::path::PathBuf;

use rmcp::{
    ClientHandler,
    ErrorData as McpError,
    model::{
        ClientCapabilities, ClientInfo, CreateElicitationRequestParams, CreateElicitationResult,
        CreateMessageRequestParams, CreateMessageResult, ElicitationAction, ElicitationCapability,
        ErrorCode, Implementation, ListRootsResult, Root, RootsCapabilities, SamplingCapability,
    },
    service::{RequestContext, RoleClient},
};

// ── ClientCapabilityConfig ────────────────────────────────────────────────────

/// Runtime configuration controlling which client capabilities Forge MCP advertises
/// in the MCP `initialize` handshake.
///
/// Per AC-004 / BC-2.04.002, capabilities are determined at runtime from config —
/// not hard-coded. If no LLM proxy is configured, `sampling` must not be advertised
/// so servers don't send sampling requests that cannot be handled.
///
/// ## Defaults
///
/// `ClientCapabilityConfig::default()` enables all three capabilities (sampling,
/// elicitation, roots) to preserve backward-compatibility with STORY-013's
/// `forge_client_info()` behaviour. New code that wants selective advertisement
/// should construct the config explicitly.
#[derive(Debug, Clone)]
pub struct ClientCapabilityConfig {
    /// Advertise `sampling: {}` in `initialize`. Allows servers to call
    /// `sampling/createMessage` on this client. Set to `false` when no
    /// LLM proxy endpoint is configured.
    pub enable_sampling: bool,

    /// Advertise `elicitation: {}` in `initialize`. Allows servers to call
    /// `elicitation/create` on this client. Set to `false` for headless /
    /// non-interactive deployments per DEC-017.
    pub enable_elicitation: bool,

    /// Advertise `roots: { listChanged: true }` in `initialize`. Allows servers
    /// to call `roots/list`. Root change notifications are sent when `root_paths`
    /// changes at runtime.
    pub enable_roots: bool,

    /// The filesystem root paths returned when a server calls `roots/list`.
    /// Only used when `enable_roots` is `true`.
    pub root_paths: Vec<PathBuf>,
}

impl Default for ClientCapabilityConfig {
    /// All capabilities enabled, no root paths configured.
    ///
    /// Matches the STORY-013 `forge_client_info()` behaviour so existing
    /// connect_stdio / connect_http callers see no change.
    fn default() -> Self {
        Self {
            enable_sampling: true,
            enable_elicitation: true,
            enable_roots: true,
            root_paths: vec![],
        }
    }
}

impl ClientCapabilityConfig {
    /// Build the `ClientInfo` (identity + capabilities) to send in `initialize`.
    ///
    /// Pure function — no I/O.
    ///
    /// We use `Default::default()` + field mutation because `ClientCapabilities`
    /// is `#[non_exhaustive]` and cannot be constructed with a struct literal
    /// outside of its defining crate.
    pub fn build_client_info(&self) -> ClientInfo {
        let mut caps = ClientCapabilities::default();

        caps.sampling = if self.enable_sampling {
            Some(SamplingCapability::default())
        } else {
            None
        };

        caps.elicitation = if self.enable_elicitation {
            Some(ElicitationCapability::default())
        } else {
            None
        };

        caps.roots = if self.enable_roots {
            Some(RootsCapabilities {
                list_changed: Some(true),
            })
        } else {
            None
        };

        ClientInfo::new(caps, Implementation::new("forge-mcp", env!("CARGO_PKG_VERSION")))
    }
}

// ── ForgeClientHandler ────────────────────────────────────────────────────────

/// Forge MCP's implementation of `rmcp::ClientHandler`.
///
/// Wraps a `ClientCapabilityConfig` to:
/// - Advertise the correct capabilities in `initialize`
/// - Respond to `roots/list` with configured root paths
/// - Decline sampling / elicitation requests with stub errors (to be replaced
///   with real implementations in later stories)
///
/// ## Thread safety
///
/// `ForgeClientHandler` is `Send + Sync` — the inner config is immutable after
/// construction.
#[derive(Debug, Clone)]
pub struct ForgeClientHandler {
    config: ClientCapabilityConfig,
}

impl ForgeClientHandler {
    /// Create a new `ForgeClientHandler` from a `ClientCapabilityConfig`.
    pub fn new(config: ClientCapabilityConfig) -> Self {
        Self { config }
    }

    /// Build the `ClientInfo` to use during the MCP initialize handshake.
    pub fn client_info(&self) -> ClientInfo {
        self.config.build_client_info()
    }
}

impl ClientHandler for ForgeClientHandler {
    /// Return the configured `ClientInfo` so rmcp includes it in `initialize`.
    fn get_info(&self) -> ClientInfo {
        self.config.build_client_info()
    }

    /// `roots/list` — return configured root paths.
    ///
    /// Returns the `root_paths` from the config as rmcp `Root` objects.
    /// Root URIs use the `file://` scheme per the MCP spec.
    fn list_roots(
        &self,
        _context: RequestContext<RoleClient>,
    ) -> impl std::future::Future<Output = Result<ListRootsResult, McpError>> + Send + '_ {
        let roots: Vec<Root> = self
            .config
            .root_paths
            .iter()
            .map(|p| {
                let uri = format!("file://{}", p.to_string_lossy());
                Root::new(uri)
            })
            .collect();
        std::future::ready(Ok(ListRootsResult::new(roots)))
    }

    /// `sampling/createMessage` — stub returning not-implemented.
    ///
    /// Returns `E-PRO-007` until a real LLM proxy is wired in (future story).
    /// Per AC-004, if the client did not advertise sampling then servers should
    /// not call this at all.
    fn create_message(
        &self,
        _params: CreateMessageRequestParams,
        _context: RequestContext<RoleClient>,
    ) -> impl std::future::Future<Output = Result<CreateMessageResult, McpError>> + Send + '_ {
        std::future::ready(Err(McpError::new(
            ErrorCode::METHOD_NOT_FOUND,
            "E-PRO-007: sampling/createMessage not implemented — no LLM proxy configured",
            None,
        )))
    }

    /// `elicitation/create` — stub returning decline in non-interactive mode.
    ///
    /// Per DEC-017 / EC-002: in CLI non-interactive mode, elicitation requests
    /// are automatically declined. Full TUI implementation is a future story.
    fn create_elicitation(
        &self,
        _request: CreateElicitationRequestParams,
        _context: RequestContext<RoleClient>,
    ) -> impl std::future::Future<Output = Result<CreateElicitationResult, McpError>> + Send + '_
    {
        std::future::ready(Ok(CreateElicitationResult {
            action: ElicitationAction::Decline,
            content: None,
        }))
    }
}

// ── Pure unit tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use super::*;

    #[test]
    fn test_build_client_info_all_enabled() {
        let config = ClientCapabilityConfig {
            enable_sampling: true,
            enable_elicitation: true,
            enable_roots: true,
            root_paths: vec![],
        };
        let info = config.build_client_info();
        assert!(info.capabilities.sampling.is_some(), "sampling should be set");
        assert!(info.capabilities.elicitation.is_some(), "elicitation should be set");
        assert!(info.capabilities.roots.is_some(), "roots should be set");
        let roots = info.capabilities.roots.as_ref().unwrap();
        assert_eq!(roots.list_changed, Some(true), "listChanged must be true");
    }

    #[test]
    fn test_build_client_info_all_disabled() {
        let config = ClientCapabilityConfig {
            enable_sampling: false,
            enable_elicitation: false,
            enable_roots: false,
            root_paths: vec![],
        };
        let info = config.build_client_info();
        assert!(info.capabilities.sampling.is_none(), "sampling should not be set");
        assert!(info.capabilities.elicitation.is_none(), "elicitation should not be set");
        assert!(info.capabilities.roots.is_none(), "roots should not be set");
    }

    #[test]
    fn test_build_client_info_sampling_only() {
        let config = ClientCapabilityConfig {
            enable_sampling: true,
            enable_elicitation: false,
            enable_roots: false,
            root_paths: vec![],
        };
        let info = config.build_client_info();
        assert!(info.capabilities.sampling.is_some());
        assert!(info.capabilities.elicitation.is_none());
        assert!(info.capabilities.roots.is_none());
    }

    #[test]
    fn test_default_config_all_enabled() {
        let config = ClientCapabilityConfig::default();
        assert!(config.enable_sampling);
        assert!(config.enable_elicitation);
        assert!(config.enable_roots);
    }
}
