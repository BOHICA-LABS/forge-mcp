//! # forge-security
//!
//! MCP server security auditing and threat detection for Forge MCP.
//!
//! This is an L2 crate — it depends on `forge-traffic` (L1) and `forge-core` (L1).
//!
//! ## Responsibilities
//! - Detect prompt injection patterns in MCP tool descriptions and responses
//! - Audit tool permission scopes against declared capabilities
//! - Flag exfiltration signals: unusual data volumes, suspicious endpoints
//! - Generate structured security findings with severity ratings
//! - Provide a `SecurityReport` type consumed by the TUI and daemon
