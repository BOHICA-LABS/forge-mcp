//! Mock MCP HTTP server implementation.
//!
//! Uses rmcp's `StreamableHttpService` + `LocalSessionManager` to provide a
//! spec-compliant Streamable-HTTP MCP server for integration testing.
//!
//! Key features:
//! - Random port binding (port 0) for CI isolation
//! - `Mcp-Session-Id` header management (handled transparently by rmcp)
//! - SSE stream for server-initiated messages
//! - Configurable session expiry simulation
//! - Configurable request-count-based session loss simulation

use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};

use rmcp::{
    ServerHandler,
    handler::server::wrapper::Parameters,
    model::{
        ListPromptsResult, ListResourcesResult, PaginatedRequestParams, Prompt,
        ServerCapabilities, ServerInfo,
    },
    schemars,
    tool, tool_handler, tool_router,
    handler::server::router::tool::ToolRouter,
};
use rmcp::model::RawResource;
use rmcp::transport::streamable_http_server::{
    SessionId, SessionManager,
    session::{ServerSseMessage, local::LocalSessionManager},
    StreamableHttpServerConfig, StreamableHttpService,
};
use serde::Deserialize;
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio_util::sync::CancellationToken;
use tracing::{info, warn};

use crate::config::HttpMockConfig;

// ---------------------------------------------------------------------------
// Tool parameter types
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct EchoRequest {
    #[schemars(description = "The message to echo back")]
    pub message: String,
}

#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AddRequest {
    #[schemars(description = "First number")]
    pub a: i64,
    #[schemars(description = "Second number")]
    pub b: i64,
}

// ---------------------------------------------------------------------------
// Session expiry tracker
// ---------------------------------------------------------------------------

/// Tracks per-session request counts and expiry times.
#[derive(Debug, Default)]
pub struct SessionTracker {
    request_counts: RwLock<HashMap<String, u64>>,
    expiry_at: RwLock<HashMap<String, u64>>,
    forgotten: RwLock<HashSet<String>>,
}

impl SessionTracker {
    /// Record a request. Returns `true` if the session should be forgotten.
    pub async fn record_request(&self, session_id: &str, forget_after: Option<u64>) -> bool {
        let mut counts = self.request_counts.write().await;
        let count = counts.entry(session_id.to_owned()).or_insert(0);
        *count += 1;
        let current = *count;
        drop(counts);

        if let Some(limit) = forget_after
            && current >= limit {
                self.forgotten
                    .write()
                    .await
                    .insert(session_id.to_owned());
                return true;
            }
        false
    }

    /// Register a new session for TTL tracking.
    pub async fn register(&self, session_id: &str, ttl: Option<Duration>) {
        if let Some(ttl) = ttl {
            let now_ms = now_millis();
            let expiry = now_ms + ttl.as_millis() as u64;
            self.expiry_at
                .write()
                .await
                .insert(session_id.to_owned(), expiry);
        }
    }

    /// Returns `true` if the session has expired or been forgotten.
    pub async fn is_expired(&self, session_id: &str) -> bool {
        if self.forgotten.read().await.contains(session_id) {
            return true;
        }
        if let Some(&expiry_ms) = self.expiry_at.read().await.get(session_id) {
            return now_millis() >= expiry_ms;
        }
        false
    }

    pub async fn remove(&self, session_id: &str) {
        self.request_counts.write().await.remove(session_id);
        self.expiry_at.write().await.remove(session_id);
        self.forgotten.write().await.remove(session_id);
    }
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("time is monotonic")
        .as_millis() as u64
}

// ---------------------------------------------------------------------------
// Session manager wrapper with expiry simulation
// ---------------------------------------------------------------------------

/// Wraps `LocalSessionManager` and adds configurable session-expiry /
/// session-loss simulation.
///
/// When a session is expired or forgotten, it is evicted from the inner manager
/// so that the next `has_session` check returns false, causing rmcp to respond
/// with HTTP 404 — matching the MCP spec for an unknown session.
#[derive(Debug)]
pub struct ExpirableSessionManager {
    inner: Arc<LocalSessionManager>,
    tracker: Arc<SessionTracker>,
    config: crate::config::SessionConfig,
}

impl ExpirableSessionManager {
    pub fn new(config: crate::config::SessionConfig) -> Arc<Self> {
        Arc::new(Self {
            inner: Arc::new(LocalSessionManager::default()),
            tracker: Arc::new(SessionTracker::default()),
            config,
        })
    }
}

impl SessionManager for ExpirableSessionManager {
    type Error = <LocalSessionManager as SessionManager>::Error;
    type Transport = <LocalSessionManager as SessionManager>::Transport;

    async fn create_session(&self) -> Result<(SessionId, Self::Transport), Self::Error> {
        let (id, transport) = self.inner.create_session().await?;
        self.tracker.register(id.as_ref(), self.config.ttl()).await;
        info!(session_id = %id, "session created");
        Ok((id, transport))
    }

    async fn initialize_session(
        &self,
        id: &SessionId,
        message: rmcp::model::ClientJsonRpcMessage,
    ) -> Result<rmcp::model::ServerJsonRpcMessage, Self::Error> {
        self.inner.initialize_session(id, message).await
    }

    async fn close_session(&self, id: &SessionId) -> Result<(), Self::Error> {
        self.tracker.remove(id.as_ref()).await;
        self.inner.close_session(id).await
    }

    async fn has_session(&self, id: &SessionId) -> Result<bool, Self::Error> {
        let inner_has = self.inner.has_session(id).await?;
        if !inner_has {
            return Ok(false);
        }
        if self.tracker.is_expired(id.as_ref()).await {
            warn!(session_id = %id, "session expired/forgotten — evicting");
            let _ = self.inner.close_session(id).await;
            self.tracker.remove(id.as_ref()).await;
            return Ok(false);
        }
        Ok(true)
    }

    async fn create_stream(
        &self,
        id: &SessionId,
        message: rmcp::model::ClientJsonRpcMessage,
    ) -> Result<impl futures_core::Stream<Item = ServerSseMessage> + Send + 'static, Self::Error>
    {
        let should_forget = self
            .tracker
            .record_request(id.as_ref(), self.config.forget_after_n_requests)
            .await;
        if should_forget {
            warn!(session_id = %id, "session forgotten after request limit — evicting");
            let _ = self.inner.close_session(id).await;
        }
        self.inner.create_stream(id, message).await
    }

    async fn create_standalone_stream(
        &self,
        id: &SessionId,
    ) -> Result<impl futures_core::Stream<Item = ServerSseMessage> + Send + 'static, Self::Error>
    {
        self.inner.create_standalone_stream(id).await
    }

    async fn resume(
        &self,
        id: &SessionId,
        last_event_id: String,
    ) -> Result<impl futures_core::Stream<Item = ServerSseMessage> + Send + 'static, Self::Error>
    {
        self.inner.resume(id, last_event_id).await
    }

    async fn accept_message(
        &self,
        id: &SessionId,
        message: rmcp::model::ClientJsonRpcMessage,
    ) -> Result<(), Self::Error> {
        self.inner.accept_message(id, message).await
    }
}

// ---------------------------------------------------------------------------
// Mock MCP Handler
// ---------------------------------------------------------------------------

/// The MCP handler that responds to all protocol operations.
#[derive(Debug, Clone)]
pub struct MockHttpHandler {
    #[allow(dead_code)] // used by the #[tool_router] macro dispatch
    tool_router: ToolRouter<Self>,
    config: HttpMockConfig,
}

impl MockHttpHandler {
    pub fn new(config: HttpMockConfig) -> Self {
        Self {
            tool_router: Self::tool_router(),
            config,
        }
    }
}

#[tool_router]
impl MockHttpHandler {
    /// Echo a message back to the caller.
    #[tool(description = "Echo the provided message back")]
    fn echo(&self, Parameters(EchoRequest { message }): Parameters<EchoRequest>) -> String {
        message
    }

    /// Add two integers.
    #[tool(description = "Add two integers and return the sum")]
    fn add(&self, Parameters(AddRequest { a, b }): Parameters<AddRequest>) -> String {
        (a + b).to_string()
    }
}

#[tool_handler]
impl ServerHandler for MockHttpHandler {
    fn get_info(&self) -> ServerInfo {
        // The type-state builder doesn't support conditional enabling, so we
        // always advertise all capabilities. The handlers check the config and
        // return method_not_found if the capability is disabled.
        ServerInfo::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .enable_prompts()
                .build(),
        )
        .with_instructions("forge-test-http-server — mock MCP server for HTTP transport tests")
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<ListResourcesResult, rmcp::ErrorData> {
        if !self.config.capabilities.resources {
            return Err(rmcp::ErrorData::invalid_request(
                "resources capability not enabled",
                None,
            ));
        }
        Ok(ListResourcesResult {
            resources: vec![rmcp::model::Annotated::new(
                RawResource::new("test://mock-resource", "Mock Resource")
                    .with_description("A mock resource for testing"),
                None,
            )],
            next_cursor: None,
            ..Default::default()
        })
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: rmcp::service::RequestContext<rmcp::RoleServer>,
    ) -> Result<ListPromptsResult, rmcp::ErrorData> {
        if !self.config.capabilities.prompts {
            return Err(rmcp::ErrorData::invalid_request(
                "prompts capability not enabled",
                None,
            ));
        }
        Ok(ListPromptsResult {
            prompts: vec![Prompt::new(
                "mock-prompt",
                Some("A mock prompt for testing"),
                None,
            )],
            next_cursor: None,
            ..Default::default()
        })
    }
}

// ---------------------------------------------------------------------------
// Server startup helper
// ---------------------------------------------------------------------------

pub struct MockHttpServer;

impl MockHttpServer {
    /// Start the mock HTTP server on a random port.
    ///
    /// Returns the bound `SocketAddr` and a `JoinHandle` for the server task.
    /// Cancel `ct` to shut the server down gracefully.
    pub async fn start(
        config: HttpMockConfig,
        ct: CancellationToken,
    ) -> Result<(SocketAddr, JoinHandle<()>), Box<dyn std::error::Error + Send + Sync>> {
        let stateful = config.stateful;
        let session_cfg = config.session.clone();
        let handler_config = config.clone();

        let session_manager = ExpirableSessionManager::new(session_cfg);

        let service = StreamableHttpService::new(
            move || Ok(MockHttpHandler::new(handler_config.clone())),
            session_manager,
            StreamableHttpServerConfig::default()
                .with_stateful_mode(stateful)
                .with_sse_keep_alive(None)
                .with_cancellation_token(ct.child_token()),
        );

        let router = axum::Router::new().nest_service("/mcp", service);

        // Bind to port 0 — OS assigns a random available port (CI-safe)
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        info!(%addr, "mock HTTP server bound");

        let handle = tokio::spawn({
            let ct = ct.clone();
            async move {
                let _ = axum::serve(listener, router)
                    .with_graceful_shutdown(async move { ct.cancelled_owned().await })
                    .await;
            }
        });

        Ok((addr, handle))
    }
}
