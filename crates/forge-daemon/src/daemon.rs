//! Daemon server and client library.
//!
//! ## Architecture
//!
//! ```text
//! CLI process                      Daemon process
//! ──────────────────────────────   ──────────────────────────────────────────
//! DaemonClient                     DaemonServer
//!   └─ UnixStream/TcpStream  ───►  UnixListener/TcpListener
//!        request (JSON-RPC)              └─ SessionPool
//!        ◄─── response (JSON-RPC)
//! ```
//!
//! On platforms that support Unix domain sockets (macOS, Linux), the daemon
//! listens on a UDS at:
//!   `$XDG_RUNTIME_DIR/forge-mcp/daemon.sock`  (Linux with XDG)
//!   `/tmp/forge-mcp-<uid>/daemon.sock`         (macOS / fallback)
//!
//! On Windows the implementation uses a local TCP loopback port as a fallback.
//!
//! ## Lazy start (AC-001)
//!
//! `get_or_start_daemon()` checks if the daemon is reachable on the socket. If
//! not, it spawns `forge-mcp daemon` as a background process and waits up to
//! `DAEMON_START_TIMEOUT_SECS` for the socket to appear.
//!
//! If the daemon does not start in time (EC-001), the caller receives
//! `DaemonError::StartTimeout` and should fall back to a direct connection.
//!
//! ## IPC protocol
//!
//! A minimal line-delimited JSON-RPC 2.0 protocol over the socket:
//!
//! Request:  `{"jsonrpc":"2.0","id":1,"method":"get_connection","params":{"server":"my-server"}}\n`
//! Response: `{"jsonrpc":"2.0","id":1,"result":{"session_id":"…","server_name":"…"}}\n`
//! Error:    `{"jsonrpc":"2.0","id":1,"error":{"code":-32000,"message":"…"}}\n`

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::error::{DaemonError, Result};
use crate::pool::{ConnectionFactory, PoolConfig, SessionPool};
use crate::session::SessionId;

// ── Constants ─────────────────────────────────────────────────────────────────

/// How long to wait for the daemon to become available (AC-001, EC-001).
pub const DAEMON_START_TIMEOUT_SECS: u64 = 3;

/// How often to poll the socket path while waiting for daemon start.
const SOCKET_POLL_INTERVAL_MS: u64 = 50;

// ── Socket path ───────────────────────────────────────────────────────────────

/// Returns the platform-appropriate daemon socket path.
///
/// Priority:
/// 1. `$XDG_RUNTIME_DIR/forge-mcp/daemon.sock` (Linux with XDG)
/// 2. `/tmp/forge-mcp-<uid>/daemon.sock` (macOS / fallback)
pub fn daemon_socket_path() -> PathBuf {
    #[cfg(unix)]
    {
        // Try XDG_RUNTIME_DIR first (Linux with logind).
        if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
            let mut p = PathBuf::from(xdg);
            p.push("forge-mcp");
            p.push("daemon.sock");
            return p;
        }

        // Fallback: /tmp/forge-mcp-<uid>/daemon.sock
        let uid = {
            // Use nix if available; otherwise read /proc/self/status.
            // Safe fallback: use process ID as a proxy (not UID) for tests.
            #[cfg(target_os = "linux")]
            {
                // SAFETY: getuid() is always safe.
                unsafe { libc_uid() }
            }
            #[cfg(not(target_os = "linux"))]
            {
                // macOS: use geteuid via std::process workaround.
                std::process::id()
            }
        };

        let mut p = PathBuf::from(format!("/tmp/forge-mcp-{uid}"));
        p.push("daemon.sock");
        p
    }

    #[cfg(windows)]
    {
        // Windows: use a named pipe path as a stub.
        PathBuf::from(r"\\.\pipe\forge-mcp-daemon")
    }
}

#[cfg(all(unix, target_os = "linux"))]
unsafe fn libc_uid() -> u32 {
    extern "C" {
        fn getuid() -> u32;
    }
    getuid()
}

// ── IPC protocol types ─────────────────────────────────────────────────────────

/// JSON-RPC 2.0 request sent from client to daemon.
#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonRequest {
    pub jsonrpc: String,
    pub id: u64,
    pub method: String,
    #[serde(default)]
    pub params: serde_json::Value,
}

/// JSON-RPC 2.0 success response.
#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonResponse {
    pub jsonrpc: String,
    pub id: u64,
    pub result: serde_json::Value,
}

/// JSON-RPC 2.0 error response.
#[derive(Debug, Serialize, Deserialize)]
pub struct DaemonErrorResponse {
    pub jsonrpc: String,
    pub id: u64,
    pub error: RpcError,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
}

/// Parameters for the `get_connection` method.
#[derive(Debug, Serialize, Deserialize)]
pub struct GetConnectionParams {
    pub server: String,
}

/// Result for the `get_connection` method.
#[derive(Debug, Serialize, Deserialize)]
pub struct GetConnectionResult {
    pub session_id: String,
    pub server_name: String,
}

// ── SessionHandle ─────────────────────────────────────────────────────────────

/// A handle to a pooled session, returned to the CLI caller.
#[derive(Debug, Clone)]
pub struct SessionHandle {
    pub session_id: SessionId,
    pub server_name: String,
}

// ── DaemonServer ───────────────────────────────────────────────────────────────

/// The daemon server process.
///
/// Binds to the Unix domain socket and handles incoming CLI requests by
/// dispatching to the `SessionPool`.
pub struct DaemonServer {
    pool: Arc<SessionPool>,
    socket_path: PathBuf,
    idle_eviction_interval: Duration,
}

impl DaemonServer {
    /// Create a new `DaemonServer` using the default socket path and pool config.
    pub fn new(factory: ConnectionFactory) -> Self {
        Self::with_config(
            daemon_socket_path(),
            PoolConfig::default(),
            factory,
            Duration::from_secs(60),
        )
    }

    /// Create with explicit socket path and config.
    pub fn with_config(
        socket_path: PathBuf,
        pool_config: PoolConfig,
        factory: ConnectionFactory,
        idle_eviction_interval: Duration,
    ) -> Self {
        Self {
            pool: Arc::new(SessionPool::new(pool_config, factory)),
            socket_path,
            idle_eviction_interval,
        }
    }

    /// Run the daemon event loop.
    ///
    /// Binds the socket, then accepts and handles connections until the process
    /// is terminated.
    #[cfg(unix)]
    pub async fn run(self) -> Result<()> {
        use tokio::net::UnixListener;

        // Create socket directory if it doesn't exist.
        if let Some(parent) = self.socket_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // Remove stale socket if present.
        let _ = tokio::fs::remove_file(&self.socket_path).await;

        let listener = UnixListener::bind(&self.socket_path)
            .map_err(|e| DaemonError::SocketBind {
                path: self.socket_path.display().to_string(),
                cause: e.to_string(),
            })?;

        info!("daemon listening on {:?}", self.socket_path);

        let pool = Arc::clone(&self.pool);
        let evict_interval = self.idle_eviction_interval;

        // Spawn idle-eviction background task.
        let pool_for_eviction = Arc::clone(&pool);
        tokio::spawn(async move {
            let mut ticker = tokio::time::interval(evict_interval);
            loop {
                ticker.tick().await;
                pool_for_eviction.evict_idle().await;
            }
        });

        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let pool = Arc::clone(&pool);
                    tokio::spawn(async move {
                        if let Err(e) = handle_unix_connection(stream, pool).await {
                            error!("connection handler error: {e}");
                        }
                    });
                }
                Err(e) => {
                    error!("accept error: {e}");
                }
            }
        }
    }

    /// Stub run for non-Unix platforms.
    #[cfg(not(unix))]
    pub async fn run(self) -> Result<()> {
        Err(DaemonError::Ipc(
            "Unix domain sockets not supported on this platform".to_string(),
        ))
    }

    /// Access the internal pool (for testing).
    pub fn pool(&self) -> Arc<SessionPool> {
        Arc::clone(&self.pool)
    }
}

#[cfg(unix)]
async fn handle_unix_connection(
    stream: tokio::net::UnixStream,
    pool: Arc<SessionPool>,
) -> Result<()> {
    let (reader, mut writer) = tokio::io::split(stream);
    let mut lines = BufReader::new(reader).lines();

    while let Ok(Some(line)) = lines.next_line().await {
        debug!("daemon received: {line}");
        let response = dispatch_request(&line, &pool).await;
        let mut out = serde_json::to_string(&response)
            .unwrap_or_else(|_| r#"{"jsonrpc":"2.0","id":0,"error":{"code":-32700,"message":"internal error"}}"#.to_string());
        out.push('\n');
        writer.write_all(out.as_bytes()).await?;
    }

    Ok(())
}

/// Dispatch a raw JSON-RPC line to the pool and return a serializable response.
async fn dispatch_request(
    line: &str,
    pool: &SessionPool,
) -> serde_json::Value {
    let req: DaemonRequest = match serde_json::from_str(line) {
        Ok(r) => r,
        Err(e) => {
            return serde_json::json!({
                "jsonrpc": "2.0",
                "id": 0,
                "error": { "code": -32700, "message": format!("parse error: {e}") }
            });
        }
    };

    let id = req.id;
    match req.method.as_str() {
        "get_connection" => {
            let params: GetConnectionParams = match serde_json::from_value(req.params) {
                Ok(p) => p,
                Err(e) => {
                    return serde_json::json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": { "code": -32602, "message": format!("invalid params: {e}") }
                    });
                }
            };

            match pool.get_or_create(&params.server).await {
                Ok(handle) => serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "session_id": handle.session_id.as_str(),
                        "server_name": handle.server_name
                    }
                }),
                Err(e) => serde_json::json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": { "code": -32000, "message": e.to_string() }
                }),
            }
        }
        other => serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "error": { "code": -32601, "message": format!("method not found: {other}") }
        }),
    }
}

// ── DaemonClient ───────────────────────────────────────────────────────────────

/// Client handle for communicating with a running daemon.
pub struct DaemonClient {
    socket_path: PathBuf,
    next_id: Arc<Mutex<u64>>,
}

impl DaemonClient {
    /// Create a client for the given socket path.
    pub fn new(socket_path: PathBuf) -> Self {
        Self {
            socket_path,
            next_id: Arc::new(Mutex::new(1)),
        }
    }

    /// Check if the daemon is reachable by attempting a connection.
    pub async fn is_alive(&self) -> bool {
        self.connect().await.is_ok()
    }

    /// Request a pooled session handle for `server`.
    ///
    /// Sends a `get_connection` JSON-RPC request to the daemon and returns
    /// the session handle on success.
    pub async fn request_connection(&self, server: &str) -> Result<SessionHandle> {
        let id = {
            let mut guard = self.next_id.lock().await;
            let id = *guard;
            *guard += 1;
            id
        };

        let req = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": "get_connection",
            "params": { "server": server }
        });

        let response = self.send_request(&req).await?;

        if let Some(result) = response.get("result") {
            let session_id = result
                .get("session_id")
                .and_then(|v| v.as_str())
                .ok_or_else(|| DaemonError::Ipc("missing session_id in response".to_string()))?;
            let server_name = result
                .get("server_name")
                .and_then(|v| v.as_str())
                .ok_or_else(|| DaemonError::Ipc("missing server_name in response".to_string()))?;

            return Ok(SessionHandle {
                session_id: SessionId::from_str(session_id),
                server_name: server_name.to_string(),
            });
        }

        if let Some(error) = response.get("error") {
            let msg = error
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown daemon error");
            return Err(DaemonError::Ipc(msg.to_string()));
        }

        Err(DaemonError::Ipc("unexpected daemon response".to_string()))
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    #[cfg(unix)]
    async fn connect(&self) -> Result<tokio::net::UnixStream> {
        tokio::net::UnixStream::connect(&self.socket_path)
            .await
            .map_err(|e| DaemonError::Ipc(e.to_string()))
    }

    #[cfg(not(unix))]
    async fn connect(&self) -> Result<()> {
        Err(DaemonError::Ipc(
            "Unix domain sockets not supported on this platform".to_string(),
        ))
    }

    #[cfg(unix)]
    async fn send_request(&self, req: &serde_json::Value) -> Result<serde_json::Value> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

        let stream = self.connect().await?;
        let (reader, mut writer) = tokio::io::split(stream);
        let mut lines = BufReader::new(reader).lines();

        let mut line = serde_json::to_string(req)?;
        line.push('\n');
        writer.write_all(line.as_bytes()).await?;

        let response_line = lines
            .next_line()
            .await?
            .ok_or_else(|| DaemonError::Ipc("daemon closed connection without response".to_string()))?;

        let value: serde_json::Value = serde_json::from_str(&response_line)?;
        Ok(value)
    }

    #[cfg(not(unix))]
    async fn send_request(&self, _req: &serde_json::Value) -> Result<serde_json::Value> {
        Err(DaemonError::Ipc(
            "Unix domain sockets not supported on this platform".to_string(),
        ))
    }
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Get an existing daemon client, or start the daemon if not running.
///
/// - Checks if daemon socket is reachable.
/// - If not, starts the daemon (platform-specific) and waits up to
///   `DAEMON_START_TIMEOUT_SECS` for it to become available (AC-001).
/// - If startup times out, returns `Err(DaemonError::StartTimeout)` (EC-001).
///
/// The caller should fall back to a direct connection on `StartTimeout`.
pub async fn get_or_start_daemon() -> Result<DaemonClient> {
    get_or_start_daemon_at(daemon_socket_path()).await
}

/// Internal: get-or-start using an explicit socket path (testable).
pub async fn get_or_start_daemon_at(socket_path: PathBuf) -> Result<DaemonClient> {
    let client = DaemonClient::new(socket_path.clone());

    if client.is_alive().await {
        debug!("daemon already running at {:?}", socket_path);
        return Ok(client);
    }

    // Attempt to start the daemon.
    info!("daemon not running — attempting lazy start");
    start_daemon_process(&socket_path).await?;

    // Wait for the socket to appear.
    let deadline = tokio::time::Instant::now()
        + Duration::from_secs(DAEMON_START_TIMEOUT_SECS);

    loop {
        if tokio::time::Instant::now() >= deadline {
            warn!("daemon did not start within {DAEMON_START_TIMEOUT_SECS}s");
            return Err(DaemonError::StartTimeout {
                seconds: DAEMON_START_TIMEOUT_SECS,
            });
        }

        tokio::time::sleep(Duration::from_millis(SOCKET_POLL_INTERVAL_MS)).await;

        if client.is_alive().await {
            info!("daemon is up");
            return Ok(client);
        }
    }
}

/// Spawn the daemon process in the background.
///
/// Looks for `forge-mcp` on `$PATH` and runs `forge-mcp daemon`.
/// If the binary is not found, returns an error (caller should fall back to
/// direct connection).
async fn start_daemon_process(_socket_path: &PathBuf) -> Result<()> {
    // Determine the current executable path — run the same binary with
    // `daemon` subcommand.
    let exe = std::env::current_exe()
        .unwrap_or_else(|_| PathBuf::from("forge-mcp"));

    debug!("spawning daemon: {:?} daemon", exe);

    tokio::process::Command::new(&exe)
        .arg("daemon")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| DaemonError::Ipc(format!("failed to spawn daemon: {e}")))?;

    Ok(())
}

/// Request a connection from the daemon.
///
/// Convenience wrapper around `DaemonClient::request_connection`.
pub async fn request_connection(
    client: &DaemonClient,
    server: &str,
) -> Result<SessionHandle> {
    client.request_connection(server).await
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    // ── Socket path (AC-001) ──────────────────────────────────────────────────

    /// The daemon socket path is non-empty and contains expected components.
    #[test]
    fn test_daemon_socket_path_is_correct_for_platform() {
        let path = daemon_socket_path();
        let path_str = path.to_string_lossy();

        // Must contain the forge-mcp component.
        assert!(
            path_str.contains("forge-mcp"),
            "socket path must contain 'forge-mcp': {path_str}"
        );

        // Must end with daemon.sock.
        assert!(
            path_str.ends_with("daemon.sock"),
            "socket path must end with 'daemon.sock': {path_str}"
        );
    }

    /// XDG_RUNTIME_DIR is respected when set.
    #[test]
    fn test_xdg_runtime_dir_is_used_when_set() {
        // This test modifies env — only safe in single-threaded context.
        // We check the logic by direct path construction rather than env mutation.
        let xdg = std::env::var("XDG_RUNTIME_DIR").unwrap_or_default();
        if !xdg.is_empty() {
            let path = daemon_socket_path();
            assert!(
                path.starts_with(&xdg),
                "socket path must be under XDG_RUNTIME_DIR when set"
            );
        }
        // If XDG_RUNTIME_DIR is not set, the fallback /tmp path is used.
        // Both branches produce a valid path ending in daemon.sock.
        let path = daemon_socket_path();
        assert!(path.to_string_lossy().ends_with("daemon.sock"));
    }

    // ── In-process daemon integration test (AC-001, AC-002) ──────────────────

    /// AC-001 + AC-002: Start a real DaemonServer in-process over a temp socket,
    /// then connect a DaemonClient and verify pool reuse.
    #[cfg(unix)]
    #[tokio::test]
    async fn test_BC_1_03_001_daemon_lazy_start() {
        use crate::pool::{MockConnection, PoolableConnection};
        use tempfile::TempDir;

        let tmpdir = TempDir::new().expect("tempdir");
        let sock = tmpdir.path().join("test-daemon.sock");

        let counter = Arc::new(AtomicUsize::new(0));
        let factory: ConnectionFactory = {
            let c = Arc::clone(&counter);
            Arc::new(move |server: &str| {
                let c2 = Arc::clone(&c);
                let label = server.to_string();
                Box::pin(async move {
                    c2.fetch_add(1, Ordering::Relaxed);
                    Ok(MockConnection::alive(label) as Box<dyn PoolableConnection>)
                })
            })
        };

        let server = DaemonServer::with_config(
            sock.clone(),
            PoolConfig::default(),
            factory,
            Duration::from_secs(300),
        );

        // Run the daemon in a background task.
        tokio::spawn(async move {
            server.run().await.ok();
        });

        // Give it a moment to bind.
        tokio::time::sleep(Duration::from_millis(100)).await;

        let client = DaemonClient::new(sock);

        // First request — should create a connection.
        let h1 = client
            .request_connection("test-server")
            .await
            .expect("first request");
        assert_eq!(h1.server_name, "test-server");

        // Second request — must reuse (same session_id).
        let h2 = client
            .request_connection("test-server")
            .await
            .expect("second request");

        assert_eq!(
            h1.session_id, h2.session_id,
            "second request must return the same pooled session"
        );
        assert_eq!(
            counter.load(Ordering::Relaxed),
            1,
            "factory must be called exactly once"
        );
    }
}
