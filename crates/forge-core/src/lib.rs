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
pub mod transport;
pub mod types;

// Convenience re-exports for callers.
pub use connection::{ConnectionState, McpConnection, TransportKind};
pub use error::{CoreError, ForgeError, Result};
pub use transport::{connect_http, connect_stdio, connect_stdio_with_timeout, DEFAULT_CONNECT_TIMEOUT_SECS};
pub use types::{
    ConfigSource, EditorKind, HttpConfig, ServerEntry, StdioConfig, Transport,
    TransportConfig,
};
