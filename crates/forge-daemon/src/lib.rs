//! # forge-daemon
//!
//! Background daemon for persistent MCP server monitoring for Forge MCP.
//!
//! This is an L3 crate — it depends on `forge-core` (L1) and `forge-discovery` (L2).
//!
//! ## Responsibilities
//! - Run as a long-lived background process (launchd / systemd / Windows service)
//! - Continuously monitor discovered MCP servers via polling and event subscriptions
//! - Persist monitoring state to disk across restarts
//! - Expose a Unix domain socket IPC interface for the CLI and TUI to query
//! - Coordinate discovery, health checks, and security scans on a configurable schedule
