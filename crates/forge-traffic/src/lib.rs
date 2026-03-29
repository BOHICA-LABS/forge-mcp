//! # forge-traffic
//!
//! MCP traffic capture, recording, and replay for Forge MCP.
//!
//! This is an L1 crate with no internal dependencies (parallel to `forge-core`).
//!
//! ## Responsibilities
//! - Intercept and record MCP JSON-RPC message streams
//! - Serialize captured traffic to HAR-like or NDJSON formats for later analysis
//! - Provide replay capability for captured sessions (for regression testing)
//! - Compute traffic statistics: message rates, payload sizes, latency distributions
//! - Feed health and security crates with raw message observations
