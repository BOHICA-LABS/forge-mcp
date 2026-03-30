//! Error types for forge-daemon.

use thiserror::Error;

/// Top-level error type for the forge-daemon crate.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum DaemonError {
    /// The daemon socket was not found after the timeout period.
    #[error("daemon socket not available after {seconds}s wait")]
    SocketNotAvailable { seconds: u64 },

    /// Failed to bind the daemon socket.
    #[error("failed to bind daemon socket at {path}: {cause}")]
    SocketBind { path: String, cause: String },

    /// E-DAE-002: Socket conflict detected — another daemon is alive at the path.
    #[error("E-DAE-002: socket conflict at {path}: daemon already running")]
    SocketConflict { path: String },

    /// E-DAE-003: Stale socket was found and removed.
    #[error("E-DAE-003: stale socket removed at {path}")]
    StaleSocketRemoved { path: String },

    /// E-DAE-004: Cannot acquire the daemon lock (permission denied or locked).
    #[error("E-DAE-004: cannot acquire lock at {path}: {cause}")]
    LockAcquireFailed { path: String, cause: String },

    /// IPC communication error.
    #[error("IPC error: {0}")]
    Ipc(String),

    /// Failed to create a new connection.
    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    /// Pool is in an inconsistent state.
    #[error("pool error: {0}")]
    Pool(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization/deserialization error.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// The daemon failed to start within the configured timeout.
    #[error("daemon start timeout after {seconds}s")]
    StartTimeout { seconds: u64 },

    /// Named session was not found (E-CON-003).
    #[error("session '{name}' not found")]
    SessionNotFound { name: String },
}

/// Convenience `Result` alias.
pub type Result<T> = std::result::Result<T, DaemonError>;
