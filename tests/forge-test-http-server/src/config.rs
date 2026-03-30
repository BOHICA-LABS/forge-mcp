//! Configuration for the mock HTTP MCP server.
//!
//! Mirrors the `MockConfig` pattern established by STORY-002 (forge-test-server)
//! but adapted for HTTP / Streamable-HTTP transport specifics.

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Configures which MCP capabilities the mock server advertises.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityConfig {
    /// Whether to advertise the `tools` capability.
    pub tools: bool,
    /// Whether to advertise the `resources` capability.
    pub resources: bool,
    /// Whether to advertise the `prompts` capability.
    pub prompts: bool,
    /// Whether to advertise the `logging` capability.
    pub logging: bool,
}

impl Default for CapabilityConfig {
    fn default() -> Self {
        Self {
            tools: true,
            resources: true,
            prompts: true,
            logging: false,
        }
    }
}

/// Configures how the server manages sessions.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SessionConfig {
    /// TTL in milliseconds after which a session is considered expired.
    /// `None` means sessions never expire.
    pub ttl_ms: Option<u64>,

    /// Forget the session after this many requests (simulates mid-stream session loss).
    /// `None` means sessions persist for their full TTL.
    pub forget_after_n_requests: Option<u64>,
}

impl SessionConfig {
    pub fn ttl(&self) -> Option<Duration> {
        self.ttl_ms.map(Duration::from_millis)
    }
}

/// Configures what notifications the server sends via SSE.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotificationConfig {
    /// Send a progress notification after each tool call.
    pub progress_on_tool_call: bool,
    /// Send a `tools/list_changed` notification once after initialization.
    pub list_changed_on_init: bool,
    /// Delay in milliseconds before sending notifications (0 = immediate).
    pub delay_ms: u64,
}

/// Top-level configuration for the mock HTTP MCP server.
///
/// This is the HTTP-transport equivalent of `MockConfig` from STORY-002.
/// Load it from the `MOCK_HTTP_CONFIG` environment variable (JSON-encoded).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpMockConfig {
    /// Which MCP capabilities to advertise.
    pub capabilities: CapabilityConfig,
    /// Session lifecycle configuration.
    pub session: SessionConfig,
    /// Notification scheduling configuration.
    pub notifications: NotificationConfig,
    /// Whether to run the server in stateful mode (the default, and recommended).
    /// Setting this to false switches rmcp to stateless mode (no session IDs).
    pub stateful: bool,
}

impl Default for HttpMockConfig {
    fn default() -> Self {
        Self {
            capabilities: CapabilityConfig::default(),
            session: SessionConfig::default(),
            notifications: NotificationConfig::default(),
            stateful: true,
        }
    }
}
