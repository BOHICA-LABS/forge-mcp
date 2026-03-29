//! # forge-core
//!
//! Core MCP protocol abstractions for Forge MCP.
//!
//! This is an L1 crate — it depends only on `forge-config` (L0).
//!
//! ## Responsibilities
//! - Wrap the `rmcp` SDK to provide a Forge-specific MCP client
//! - Define shared domain types: `McpServer`, `McpTool`, `McpResource`, `McpPrompt`
//! - Provide connection lifecycle management (connect, disconnect, reconnect)
//! - Expose structured error types for all MCP protocol failures
//! - Establish the purity boundary: protocol I/O is impure; parsing/validation is pure
