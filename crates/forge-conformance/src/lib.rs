//! # forge-conformance
//!
//! MCP specification conformance testing for Forge MCP.
//!
//! This is an L3 crate — it depends on `forge-core` (L1).
//!
//! ## Responsibilities
//! - Execute the official MCP conformance test suite against a target server
//! - Validate JSON-RPC envelope structure, method names, and error codes
//! - Check capability negotiation: tools/list, resources/list, prompts/list
//! - Verify pagination, streaming, and cancellation protocol compliance
//! - Produce a `ConformanceReport` with per-test pass/fail/skip results
