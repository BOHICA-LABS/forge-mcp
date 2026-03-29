//! # forge-config
//!
//! Configuration management for Forge MCP.
//!
//! This is the L0 foundation crate — it has no internal dependencies.
//! All other crates may depend on this crate.
//!
//! ## Responsibilities
//! - Load and validate forge-mcp configuration files (TOML/JSON)
//! - Provide typed configuration structs for all subsystems
//! - Support environment variable overrides
//! - Expose a unified `Config` type consumed by higher-level crates
