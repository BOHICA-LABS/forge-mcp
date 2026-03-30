//! Structured error types for forge-core.
//!
//! Error codes follow the taxonomy defined in the architecture:
//!   E-CON-001 — connection refused / not found
//!   E-CON-002 — server process exited unexpectedly
//!   E-CON-003 — connection timeout
//!   E-PRO-001 — protocol framing error (bad JSON-RPC)

use thiserror::Error;

/// All errors that can arise from forge-core operations.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ForgeError {
    /// E-CON-001: The server command was not found or could not be spawned.
    #[error("E-CON-001: server command not found or failed to spawn: {message}")]
    ServerNotFound { message: String },

    /// E-CON-002: The server process exited unexpectedly.
    #[error("E-CON-002: server process exited with code {code:?}")]
    ServerProcessExited { code: Option<i32> },

    /// E-CON-003: Connection timed out before initialize completed.
    #[error("E-CON-003: timeout after {seconds}s waiting for server initialize")]
    ConnectionTimeout { seconds: u64 },

    /// E-PRO-001: Protocol framing error — bad JSON-RPC from the server.
    #[error("E-PRO-001: protocol error from server: {message}")]
    ProtocolError { message: String },

    /// Underlying I/O error (not otherwise classified).
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// rmcp service-layer error.
    #[error("rmcp error: {0}")]
    Rmcp(#[from] rmcp::ServiceError),
}

/// Convenience alias used throughout forge-core.
pub type Result<T> = std::result::Result<T, ForgeError>;
