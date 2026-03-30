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

pub mod daemon;
pub mod error;
pub mod pool;
pub mod session;
pub mod socket;

// Convenience re-exports.
pub use daemon::{
    DaemonClient, DaemonServer, SessionHandle,
    get_or_start_daemon, get_or_start_daemon_at, request_connection,
    DAEMON_START_TIMEOUT_SECS,
};
pub use error::{DaemonError, Result};
pub use pool::{ConnectionFactory, NamedSession, PoolConfig, PoolableConnection, SessionPool};
pub use session::SessionId;
pub use socket::{
    SocketConflictResolution, check_pid_lock, daemon_socket_path,
    pid_lock_path, remove_pid_lock, resolve_socket_conflict, write_pid_lock,
};
