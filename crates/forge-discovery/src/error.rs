//! Error types for forge-discovery.

use std::path::PathBuf;

use thiserror::Error;

/// Errors returned by the discovery layer.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum DiscoveryError {
    /// The home directory (`$HOME` / `%APPDATA%`) could not be determined.
    ///
    /// Error code: E-CFG-001
    #[error("E-CFG-001: home directory is not set or cannot be determined")]
    HomeDirectoryMissing,

    /// A config path contains non-UTF-8 bytes and cannot be represented.
    ///
    /// Error code: E-CFG-002
    #[error("E-CFG-002: config path contains non-UTF-8 bytes: {path:?}")]
    NonUtf8Path { path: PathBuf },
}

/// Errors returned by the config parser (`parse_config`).
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConfigError {
    /// The config file contains invalid JSON and cannot be parsed.
    ///
    /// Error code: E-CFG-003
    #[error("E-CFG-003: JSON parse error at {path}: {source}")]
    JsonParseError {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },

    /// The config file is valid JSON but contains neither `mcpServers` nor `servers`.
    ///
    /// Error code: E-CFG-004
    /// Processing continues for other discovered config files (SR-004).
    #[error("E-CFG-004: no recognized MCP schema (mcpServers or servers) in {path}")]
    NoRecognizedSchema { path: PathBuf },

    /// An `env` map entry has a value that is not a JSON string.
    ///
    /// Error code: E-CFG-007
    #[error("E-CFG-007: env value for key '{key}' in server '{server}' at {path} is not a string")]
    EnvValueNotString {
        path: PathBuf,
        server: String,
        key: String,
    },
}
