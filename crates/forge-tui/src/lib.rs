//! # forge-tui
//!
//! Terminal UI for Forge MCP — interactive MCP server inspection dashboard.
//!
//! This is an L4 crate — it depends on `forge-core` (L1), `forge-traffic` (L1),
//! `forge-health` (L2), and `forge-security` (L2).
//!
//! ## Responsibilities
//! - Render a full-screen ratatui dashboard with server list, tool inspector,
//!   traffic viewer, health charts, and security findings panels
//! - Handle keyboard navigation and modal interactions via crossterm events
//! - Provide a `Tui` type that owns the terminal and event loop
//! - Support live refresh from daemon IPC or direct server connections
//! - Export screenshots to PNG/SVG for documentation and reporting
