//! # forge-discovery
//!
//! MCP server discovery for Forge MCP.
//!
//! This is an L2 crate — it depends on `forge-core` (L1) and `forge-config` (L0).
//!
//! ## Responsibilities
//! - Scan filesystem for MCP server configurations (Claude Desktop, Cursor, VS Code)
//! - Probe well-known network ports for MCP servers (SSE + stdio transports)
//! - Query MCP registries (e.g., mcp.so) for known server listings
//! - Return a deduplicated, ranked list of `DiscoveredServer` entries
//! - Support incremental re-discovery and change notifications
