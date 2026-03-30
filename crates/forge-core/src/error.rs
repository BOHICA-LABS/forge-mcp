//! Structured error taxonomy for forge-core.
//!
//! Error codes follow the `E-<CATEGORY>-<NNN>` convention from the PRD:
//! - `E-CON-*` : connection-layer errors
//! - `E-PRO-*` : protocol-layer errors

use thiserror::Error;

/// Top-level error type for forge-core operations.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum CoreError {
    // ── Connection errors ─────────────────────────────────────────────────

    /// E-CON-001: Server command not found or failed to spawn (stdio).
    #[error("E-CON-001: server command not found or failed to spawn: {message}")]
    ServerNotFound { message: String },

    /// E-CON-002: The server process exited unexpectedly.
    #[error("E-CON-002: server process exited with code {code}: {detail}")]
    ProcessExited { code: String, detail: String },

    /// E-CON-003: The connection timed out.
    #[error("E-CON-003: timeout after {seconds}s waiting for {url}")]
    Timeout { seconds: u64, url: String },

    /// E-CON-003 (stdio alias): Connection timed out during initialize handshake.
    /// Kept for backward compatibility with STORY-007 test code (`ForgeError::ConnectionTimeout`).
    #[error("E-CON-003: timeout after {seconds}s waiting for server initialize")]
    ConnectionTimeout { seconds: u64 },

    /// E-CON-004: TLS certificate error.
    #[error("E-CON-004: TLS certificate error for {url}: {cause}")]
    TlsError { url: String, cause: String },

    /// E-CON-007: Session lost and re-initialization failed.
    #[error("E-CON-007: session lost and re-initialization failed for {url}: {cause}")]
    SessionLostFatal { url: String, cause: String },

    /// E-CON-008: Connection is degraded — session was re-established after loss.
    ///
    /// This is an informational warning, not a hard error.
    #[error("E-CON-008: connection degraded — session re-established for {url}")]
    ConnectionDegraded { url: String },

    /// E-CON-009: HTTP authentication failed (401 Unauthorized).
    #[error("E-CON-009: authentication failed — HTTP 401 for {url}")]
    AuthenticationFailed { url: String },

    /// E-CON-010: Insecure HTTP scheme detected.
    ///
    /// Emitted as a warning to stderr; connection still proceeds.
    #[error("E-CON-010: insecure HTTP scheme for {url} — prefer HTTPS")]
    InsecureHttpScheme { url: String },

    /// E-CON-011: The server is unavailable (HTTP 503 or unreachable).
    #[error("E-CON-011: server unavailable — HTTP {status} for {url}")]
    ServerUnavailable { status: String, url: String },

    // ── DNS ───────────────────────────────────────────────────────────────

    /// E-CON-DNS: DNS resolution failed.
    #[error("E-CON-DNS: DNS resolution failed for {url}: {cause}")]
    DnsResolutionFailed { url: String, cause: String },

    // ── Protocol errors ───────────────────────────────────────────────────

    /// E-PRO-001: Protocol-level error (invalid JSON-RPC or unexpected response).
    #[error("E-PRO-001: MCP protocol error: {0}")]
    Protocol(String),

    /// E-PRO-003: Method not available — server lacks the required capability.
    ///
    /// Emitted by capability guards in `McpConnection` when the caller attempts
    /// to invoke a method that requires a server capability the server did not
    /// advertise during the initialize handshake.
    #[error("E-PRO-003: method {method} not available — server lacks {capability} capability")]
    CapabilityNotSupported {
        /// The JSON-RPC method that was attempted.
        method: String,
        /// The capability name that is missing (e.g., `"tools"`, `"resources"`).
        capability: String,
    },

    // ── Generic / other ───────────────────────────────────────────────────

    /// An I/O error that doesn't map to a more specific code.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Catch-all for errors propagated from rmcp that we haven't mapped yet.
    #[error("rmcp error: {0}")]
    Rmcp(String),
}

/// Convenience alias used throughout forge-core.
pub type Result<T> = std::result::Result<T, CoreError>;

// ── Backward-compat aliases ───────────────────────────────────────────────────
// STORY-007 tests import `ForgeError`; keep a type alias so existing test code
// compiles without modification.

/// Deprecated type alias — prefer [`CoreError`] for new code.
#[allow(dead_code)]
pub type ForgeError = CoreError;
