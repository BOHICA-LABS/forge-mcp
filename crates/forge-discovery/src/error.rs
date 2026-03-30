//! Error types for forge-discovery.

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
    NonUtf8Path { path: std::path::PathBuf },
}
