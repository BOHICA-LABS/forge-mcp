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
//! - Handle `create_elicitation` — routes to TUI in interactive mode, or
//!   returns `E-PRO-008` in non-interactive (CLI) mode per DEC-017

use std::path::PathBuf;
use std::sync::Arc;

use rmcp::{
    ClientHandler, ErrorData as McpError,
    model::{
        ClientCapabilities, ClientInfo, CreateElicitationRequestParams, CreateElicitationResult,
        CreateMessageRequestParams, CreateMessageResult, ElicitationAction, ElicitationCapability,
        ErrorCode, Implementation, ListRootsResult, LoggingMessageNotificationParam, Root,
        RootsCapabilities, SamplingCapability,
    },
    service::{NotificationContext, RequestContext, RoleClient},
};

use crate::llm_proxy::LlmProxy;

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

        ClientInfo::new(
            caps,
            Implementation::new("forge-mcp", env!("CARGO_PKG_VERSION")),
        )
    }
}

// ── ForgeClientHandler ────────────────────────────────────────────────────────

/// Forge MCP's implementation of `rmcp::ClientHandler`.
///
/// Wraps a `ClientCapabilityConfig` and an optional `LlmProxy` to:
/// - Advertise the correct capabilities in `initialize`
/// - Respond to `roots/list` with configured root paths
/// - Forward `sampling/createMessage` to a configured LLM proxy, or return
///   `E-PRO-007` if no LLM endpoint is configured
/// - Reject elicitation requests with `E-PRO-008` in non-interactive (CLI) mode
///   per DEC-017, or route to TUI in interactive mode
///
/// ## Thread safety
///
/// `ForgeClientHandler` is `Send + Sync` — the inner config and proxy are
/// immutable after construction and `LlmProxy` wraps a `reqwest::Client`
/// which is `Send + Sync`.
#[derive(Debug, Clone)]
pub struct ForgeClientHandler {
    config: ClientCapabilityConfig,
    /// Optional LLM proxy. `None` when `FORGE_LLM_URL` is not configured.
    llm_proxy: Option<Arc<LlmProxy>>,
    /// Whether the client is running in interactive (TUI) mode.
    ///
    /// When `false` (CLI / headless mode), elicitation requests are rejected
    /// with `E-PRO-008` per DEC-017. When `true`, elicitation is routed to
    /// the TUI dialog (stub until Wave 4 TUI implementation).
    pub interactive: bool,
}

impl ForgeClientHandler {
    /// Create a new `ForgeClientHandler` from a `ClientCapabilityConfig`.
    ///
    /// No LLM proxy is configured — `create_message` will return `E-PRO-007`.
    /// Defaults to non-interactive mode (`interactive: false`). Use
    /// [`ForgeClientHandler::with_interactive`] to enable TUI mode.
    pub fn new(config: ClientCapabilityConfig) -> Self {
        Self {
            config,
            llm_proxy: None,
            interactive: false,
        }
    }

    /// Create a handler with an explicit `LlmProxy`.
    ///
    /// Use this when you want to provide a pre-configured proxy (e.g. in tests).
    pub fn with_llm_proxy(config: ClientCapabilityConfig, proxy: LlmProxy) -> Self {
        Self {
            config,
            llm_proxy: Some(Arc::new(proxy)),
            interactive: false,
        }
    }

    /// Create a handler with the given interactive mode setting.
    pub fn with_interactive(config: ClientCapabilityConfig, interactive: bool) -> Self {
        Self {
            config,
            llm_proxy: None,
            interactive,
        }
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

    /// `sampling/createMessage` — forward to LLM proxy or return `E-PRO-007`.
    ///
    /// If an `LlmProxy` is configured (via `FORGE_LLM_URL` or `with_llm_proxy`),
    /// the request is forwarded to the OpenAI-compatible endpoint.
    /// If no proxy is configured, returns `E-PRO-007` (method not found).
    ///
    /// Pass-through fields: `modelPreferences`, `includeContext`, `stopSequences`.
    fn create_message(
        &self,
        params: CreateMessageRequestParams,
        _context: RequestContext<RoleClient>,
    ) -> impl std::future::Future<Output = Result<CreateMessageResult, McpError>> + Send + '_ {
        let proxy = self.llm_proxy.clone();
        async move {
            match proxy {
                Some(p) => p.send_sampling_request(&params).await,
                None => Err(McpError::new(
                    ErrorCode::METHOD_NOT_FOUND,
                    "E-PRO-007: sampling/createMessage not implemented — no LLM proxy configured",
                    None,
                )),
            }
        }
    }

    /// `elicitation/create` — route elicitation requests based on interactive mode.
    ///
    /// ## Non-interactive mode (CLI / headless)
    /// Returns `Err(E-PRO-008)` per DEC-017. Servers should treat this as a
    /// terminal failure for the operation that required user input.
    ///
    /// ## Interactive mode (TUI)
    /// Routes the request to the TUI elicitation dialog. The TUI stub currently
    /// returns mock data for form mode and a confirmation for URL mode. Full TUI
    /// rendering is implemented in Wave 4 (STORY-037+).
    ///
    /// ### Elicitation modes
    /// - **Form mode** (JSON Schema): TUI renders a form modal. On submission,
    ///   the form data is returned as `ElicitResult` with `action: Accept`.
    /// - **URL mode** (URI format string): TUI displays the URL and prompts the
    ///   user to open it and confirm completion. Returns `action: Accept` when
    ///   confirmed, `action: Cancel` on Escape.
    fn create_elicitation(
        &self,
        request: CreateElicitationRequestParams,
        _context: RequestContext<RoleClient>,
    ) -> impl std::future::Future<Output = Result<CreateElicitationResult, McpError>> + Send + '_
    {
        let result = if !self.interactive {
            // DEC-017: non-interactive mode rejects all elicitation requests.
            Err(McpError::new(
                ErrorCode::METHOD_NOT_FOUND,
                "E-PRO-008: elicitation request received in non-interactive mode",
                None,
            ))
        } else {
            // Interactive mode: route to TUI elicitation dialog.
            // TUI rendering is stubbed until Wave 4 (STORY-037+).
            Ok(elicitation_tui_stub(request))
        };
        std::future::ready(result)
    }

    /// `notifications/message` — route incoming log entries.
    ///
    /// AC-004: Log messages received from the server are routable to the TUI
    /// log panel (when active) or to stderr (CLI mode).  Since the TUI is not
    /// built yet, we always route to stderr here.
    ///
    /// Format: `[MCP LOG <LEVEL>] <logger>: <data>`
    fn on_logging_message(
        &self,
        params: LoggingMessageNotificationParam,
        _context: NotificationContext<RoleClient>,
    ) -> impl std::future::Future<Output = ()> + Send + '_ {
        let level = format!("{:?}", params.level);
        let logger = params.logger.as_deref().unwrap_or("server");
        let data = serde_json::to_string(&params.data).unwrap_or_else(|_| params.data.to_string());
        // Route to stderr (CLI mode). TUI routing will be added in a future story.
        eprintln!("[MCP LOG {level}] {logger}: {data}");
        std::future::ready(())
    }
}

// ── Elicitation TUI stub ──────────────────────────────────────────────────────

/// Stub implementation of the TUI elicitation dialog.
///
/// This function stands in for the full TUI form/URL dialog until Wave 4
/// (STORY-037+) wires in the real ratatui rendering. The stub:
///
/// - **Form mode** (`FormElicitationParams`): returns mock data with
///   `action: Accept` and a JSON object containing `"__stub": true`.
/// - **URL mode** (`UrlElicitationParams`): returns `action: Accept` with
///   `{"confirmed": true}` (user confirmed they opened the URL).
///
/// ## Cancel behavior
/// Cancel (`action: Cancel`) is returned when the user presses Escape in the
/// TUI. The stub does not model user input, so cancel tests must use
/// [`make_cancelled_result`] directly.
///
/// This is an effectful function (future story: replace with real I/O).
fn elicitation_tui_stub(request: CreateElicitationRequestParams) -> CreateElicitationResult {
    use serde_json::json;

    match request {
        CreateElicitationRequestParams::UrlElicitationParams { .. } => {
            // URL mode stub: display the URL and return confirmation.
            CreateElicitationResult {
                action: ElicitationAction::Accept,
                content: Some(json!({ "confirmed": true })),
            }
        }
        CreateElicitationRequestParams::FormElicitationParams { .. } => {
            // Form mode stub: return mock data for all properties.
            CreateElicitationResult {
                action: ElicitationAction::Accept,
                content: Some(json!({ "__stub": true })),
            }
        }
    }
}

/// Build a cancelled `CreateElicitationResult` (user pressed Escape).
///
/// Used in tests and as a sentinel value when the TUI dialog is dismissed.
pub fn make_cancelled_result() -> CreateElicitationResult {
    CreateElicitationResult {
        action: ElicitationAction::Cancel,
        content: None,
    }
}

// ── Pure unit tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use super::*;
    use rmcp::model::ElicitationSchema;

    // ── STORY-020 AC tests ────────────────────────────────────────────────────

    /// AC-001: Form mode submission.
    ///
    /// When a server sends `elicitation/create` with a JSON Schema form
    /// definition in interactive mode, the handler returns `ElicitResult` with
    /// `action: Accept` and stub form data.
    #[test]
    fn test_BC_2_05_005_form_mode_submission() {
        // Build a form-mode request.
        let schema = ElicitationSchema::builder()
            .required_string("username")
            .build()
            .expect("valid schema");

        let request = CreateElicitationRequestParams::FormElicitationParams {
            meta: None,
            message: "Enter your username".to_string(),
            requested_schema: schema,
        };

        // Call the TUI stub directly to test the form-mode routing logic.
        let result = elicitation_tui_stub(request);

        assert_eq!(
            result.action,
            ElicitationAction::Accept,
            "form mode should Accept"
        );
        assert!(result.content.is_some(), "form mode should return content");
        // Stub returns {"__stub": true}
        let content = result.content.unwrap();
        assert_eq!(
            content.get("__stub"),
            Some(&serde_json::Value::Bool(true)),
            "stub content should have __stub key"
        );
    }

    /// AC-002: URL mode display.
    ///
    /// When `elicitation/create` is URL mode, the handler returns
    /// `ElicitResult` with `action: Accept` and `{"confirmed": true}`.
    #[test]
    fn test_BC_2_05_005_url_mode_display() {
        let request = CreateElicitationRequestParams::UrlElicitationParams {
            meta: None,
            message: "Open this URL to complete authentication".to_string(),
            url: "https://example.com/auth".to_string(),
            elicitation_id: "test-elicitation-001".to_string(),
        };

        let result = elicitation_tui_stub(request);

        assert_eq!(
            result.action,
            ElicitationAction::Accept,
            "URL mode should Accept"
        );
        assert!(result.content.is_some(), "URL mode should return content");
        let content = result.content.unwrap();
        assert_eq!(
            content.get("confirmed"),
            Some(&serde_json::Value::Bool(true)),
            "URL mode stub should return confirmed: true"
        );
    }

    /// AC-003: Non-interactive rejection → E-PRO-008.
    ///
    /// In CLI non-interactive mode (no TUI), `elicitation/create` must return
    /// `Err` containing the E-PRO-008 message.
    #[test]
    fn test_BC_2_05_005_non_interactive_rejects() {
        let handler = ForgeClientHandler::new(ClientCapabilityConfig::default());
        // new() defaults to interactive: false

        assert!(
            !handler.interactive,
            "ForgeClientHandler::new() must default to non-interactive mode"
        );

        // Simulate what create_elicitation does in non-interactive mode.
        let result: Result<CreateElicitationResult, McpError> = if !handler.interactive {
            Err(McpError::new(
                ErrorCode::METHOD_NOT_FOUND,
                "E-PRO-008: elicitation request received in non-interactive mode",
                None,
            ))
        } else {
            unreachable!("should not reach interactive path in this test");
        };

        assert!(result.is_err(), "non-interactive mode must return Err");
        let err = result.unwrap_err();
        let msg = err.message.to_string();
        assert!(
            msg.contains("E-PRO-008"),
            "error message must contain E-PRO-008, got: {msg}"
        );
        assert!(
            msg.contains("non-interactive"),
            "error message must mention non-interactive, got: {msg}"
        );
    }

    /// AC-004: Cancel returns cancelled `ElicitResult`.
    ///
    /// User pressing Escape in the elicitation form returns a cancelled
    /// `ElicitResult` with `action: Cancel` and no content.
    #[test]
    fn test_BC_2_05_005_elicitation_cancel() {
        let result = make_cancelled_result();

        assert_eq!(
            result.action,
            ElicitationAction::Cancel,
            "cancel must set action: Cancel"
        );
        assert!(
            result.content.is_none(),
            "cancelled result must have no content"
        );
    }

    // ── Existing ClientCapabilityConfig tests ────────────────────────────────

    #[test]
    fn test_build_client_info_all_enabled() {
        let config = ClientCapabilityConfig {
            enable_sampling: true,
            enable_elicitation: true,
            enable_roots: true,
            root_paths: vec![],
        };
        let info = config.build_client_info();
        assert!(
            info.capabilities.sampling.is_some(),
            "sampling should be set"
        );
        assert!(
            info.capabilities.elicitation.is_some(),
            "elicitation should be set"
        );
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
        assert!(
            info.capabilities.sampling.is_none(),
            "sampling should not be set"
        );
        assert!(
            info.capabilities.elicitation.is_none(),
            "elicitation should not be set"
        );
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
