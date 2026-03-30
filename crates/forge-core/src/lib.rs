//! # forge-core
//!
//! Core MCP protocol abstractions for Forge MCP.
//!
//! This is an L1 crate — it depends only on `forge-config` (L0) plus `rmcp`.
//!
//! ## Responsibilities
//! - Wrap the `rmcp` SDK to provide a Forge-specific MCP client
//! - Define shared domain types: `McpConnection`, connection state machine
//! - Provide connection lifecycle management (connect, disconnect)
//! - Expose structured error types for all MCP protocol failures
//! - Establish the purity boundary: protocol I/O is impure; parsing/validation is pure; state transitions are pure

pub mod connection;
pub mod error;
pub mod events;
pub mod handler;
pub mod lifecycle;
pub mod llm_proxy;
pub mod pagination;
pub mod prompts;
pub mod protocol;
pub mod resources;
pub mod subscriptions;
pub mod transport;
pub mod types;
pub mod utilities;

// Convenience re-exports for callers.
pub use connection::MCP_LATEST_VERSION;
pub use connection::{ConnectionState, McpConnection, TransportKind};
pub use error::{CoreError, ForgeError, Result};
pub use events::{CaptureChannel, MessageCaptured, MessageDirection, capture_message};
pub use handler::{ClientCapabilityConfig, ForgeClientHandler, make_cancelled_result};
pub use lifecycle::{ConnectionLifecycleConfig, ConnectionManager, PingFn, close_with_timeout};
pub use llm_proxy::{LlmProxy, LlmProxyConfig};
pub use pagination::{MAX_PAGES, PaginationState, PaginationStep};
pub use prompts::{PromptCache, get_prompt, list_prompts_all};
pub use protocol::{ToolListInvalidator, ToolListWatcher, ToolResult, call_tool, list_tools};
pub use resources::{Resource, ResourceContent, ResourceData, list_resources, read_resource};
pub use subscriptions::{ResourceSubscription, subscribe_resource, subscribe_resource_with_sender};
pub use transport::{
    DEFAULT_CONNECT_TIMEOUT_SECS, connect_http, connect_http_with_config, connect_stdio,
    connect_stdio_with_config, connect_stdio_with_config_and_timeout, connect_stdio_with_timeout,
};
pub use types::{
    ConfigSource, ConflictRecord, ConflictSource, EditorKind, FeatureSet, HttpConfig,
    NegotiatedCapabilities, ServerEntry, ServerRegistry, SpecVersion, StdioConfig, Transport,
    TransportConfig, features_for_version,
};
pub use utilities::{complete, list_roots, notify_roots_list_changed, set_log_level};
