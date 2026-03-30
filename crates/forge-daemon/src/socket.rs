//! Socket path resolution and conflict detection for the forge-daemon.
//!
//! ## Design
//!
//! This module owns two concerns:
//!
//! 1. **Pure path resolution** (`daemon_socket_path`): Returns the platform-
//!    appropriate socket path from environment variables or OS defaults.
//!    No I/O.  No side effects.
//!
//! 2. **Effectful conflict detection** (`resolve_socket_conflict`): Checks if
//!    a socket path is in use, determines whether the occupant is alive or
//!    stale, and returns a `SocketConflictResolution` describing what to do.
//!
//! ## Error codes
//!
//! | Code     | Meaning                                  |
//! |----------|------------------------------------------|
//! | E-DAE-002 | Socket conflict detected (live daemon)  |
//! | E-DAE-003 | Stale socket removed                    |
//! | E-DAE-004 | Cannot acquire lock (permission denied) |

use std::path::{Path, PathBuf};

use tracing::{info, warn};

use crate::error::DaemonError;

// ── Public types ───────────────────────────────────────────────────────────────

/// Resolution strategy chosen after inspecting the socket file.
#[derive(Debug)]
pub enum SocketConflictResolution {
    /// Another daemon is alive and listening — connect to the existing one.
    UseExisting,
    /// The socket was stale (dead PID / no listener) — it was removed and a
    /// fresh daemon can now bind.
    ReplacedStale,
    /// Conflict could not be resolved.
    Failed(DaemonError),
}

// ── Pure: socket path resolution ──────────────────────────────────────────────

/// Returns the daemon socket path according to the following priority:
///
/// 1. `FORGE_DAEMON_SOCKET` environment variable (all platforms).
/// 2. `$XDG_RUNTIME_DIR/forge-mcp/daemon.sock` (Linux with XDG).
/// 3. `$TMPDIR/forge-mcp-<uid>/daemon.sock` (macOS / Linux fallback).
/// 4. `\\.\pipe\forge-mcp` (Windows).
///
/// This function is **pure**: it reads environment variables but performs no
/// I/O and has no side effects.
pub fn daemon_socket_path() -> PathBuf {
    // 1. Explicit override (all platforms).
    if let Ok(explicit) = std::env::var("FORGE_DAEMON_SOCKET")
        && !explicit.is_empty()
    {
        return PathBuf::from(explicit);
    }

    #[cfg(unix)]
    {
        // 2. XDG_RUNTIME_DIR (Linux with logind).
        if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR")
            && !xdg.is_empty()
        {
            let mut p = PathBuf::from(xdg);
            p.push("forge-mcp");
            p.push("daemon.sock");
            return p;
        }

        // 3. TMPDIR or /tmp fallback (macOS / Linux without XDG).
        let tmp = std::env::var("TMPDIR").unwrap_or_else(|_| "/tmp".to_string());
        let uid = platform_uid();
        let mut p = PathBuf::from(tmp);
        p.push(format!("forge-mcp-{uid}"));
        p.push("daemon.sock");
        p
    }

    #[cfg(windows)]
    {
        // 4. Windows named pipe.
        PathBuf::from(r"\\.\pipe\forge-mcp")
    }
}

/// Returns the accompanying PID lock-file path for a given socket path.
///
/// The lock file sits alongside the socket:
/// `<socket-dir>/daemon.pid`
///
/// Pure: no I/O.
pub fn pid_lock_path(socket_path: &Path) -> PathBuf {
    let mut p = socket_path
        .parent()
        .unwrap_or(Path::new("/tmp"))
        .to_path_buf();
    p.push("daemon.pid");
    p
}

// ── Effectful: conflict detection ─────────────────────────────────────────────

/// Inspect `socket_path` and determine how to handle any conflict.
///
/// Call this **before** attempting to bind a new socket.
///
/// Behaviour:
/// - **No socket file** → returns `Ok(None)` (clean start — caller may bind).
/// - **Socket file exists, daemon alive** → returns
///   `Ok(Some(SocketConflictResolution::UseExisting))` (E-DAE-002).
/// - **Socket file exists, daemon dead** → removes the stale socket (and PID
///   file) and returns `Ok(Some(SocketConflictResolution::ReplacedStale))`
///   (E-DAE-003).
/// - **Permission denied** → returns
///   `Ok(Some(SocketConflictResolution::Failed(DaemonError::LockAcquireFailed …)))` (E-DAE-004).
///
/// # Errors
///
/// Returns `Err` only for unexpected I/O failures unrelated to the
/// expected conflict scenarios.
#[cfg(unix)]
pub async fn resolve_socket_conflict(
    socket_path: &Path,
) -> Result<Option<SocketConflictResolution>, DaemonError> {
    use std::io::ErrorKind;

    // Fast path: nothing there yet.
    if !socket_path.exists() {
        return Ok(None);
    }

    // Socket file exists — probe whether a daemon is listening.
    match tokio::net::UnixStream::connect(socket_path).await {
        Ok(_stream) => {
            // Successfully connected → live daemon (E-DAE-002).
            info!(
                "E-DAE-002: Daemon already running at {}",
                socket_path.display()
            );
            return Ok(Some(SocketConflictResolution::UseExisting));
        }
        Err(e) if e.kind() == ErrorKind::ConnectionRefused
            || e.kind() == ErrorKind::NotFound =>
        {
            // Socket file present but nothing listening → stale (E-DAE-003).
        }
        Err(e) if e.kind() == ErrorKind::PermissionDenied => {
            // Can't connect due to permissions (E-DAE-004).
            return Ok(Some(SocketConflictResolution::Failed(
                DaemonError::LockAcquireFailed {
                    path: socket_path.display().to_string(),
                    cause: e.to_string(),
                },
            )));
        }
        Err(e) => {
            // Treat other errors (e.g. ENOTSOCK if path is a directory) as
            // stale — we'll try to remove and let the bind attempt surface a
            // clearer error if removal fails.
            warn!(
                "unexpected connect error on {:?}: {e} — treating as stale",
                socket_path
            );
        }
    }

    // Attempt to remove the stale socket (E-DAE-003).
    match tokio::fs::remove_file(socket_path).await {
        Ok(()) => {
            info!(
                "E-DAE-003: Removed stale daemon socket at {}",
                socket_path.display()
            );
        }
        Err(e) if e.kind() == ErrorKind::PermissionDenied => {
            return Ok(Some(SocketConflictResolution::Failed(
                DaemonError::LockAcquireFailed {
                    path: socket_path.display().to_string(),
                    cause: format!("cannot remove stale socket: {e}"),
                },
            )));
        }
        Err(e) if e.kind() == ErrorKind::NotFound => {
            // Raced with another process that already cleaned it up — fine.
        }
        Err(e) => {
            return Err(DaemonError::Io(e));
        }
    }

    // Also clean up the PID lock file if it exists (best-effort).
    let pid_path = pid_lock_path(socket_path);
    let _ = tokio::fs::remove_file(&pid_path).await;

    Ok(Some(SocketConflictResolution::ReplacedStale))
}

/// Stub for non-Unix platforms.
#[cfg(not(unix))]
pub async fn resolve_socket_conflict(
    _socket_path: &Path,
) -> Result<Option<SocketConflictResolution>, DaemonError> {
    Ok(None)
}

// ── PID lock file helpers ──────────────────────────────────────────────────────

/// Write the current process PID to `lock_path`.
///
/// Effectful: creates/overwrites the file.
pub async fn write_pid_lock(lock_path: &Path) -> Result<(), DaemonError> {
    let pid = std::process::id().to_string();
    if let Some(parent) = lock_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(lock_path, pid.as_bytes()).await?;
    Ok(())
}

/// Read PID from `lock_path` and check if the process is alive.
///
/// Returns:
/// - `Ok(Some(pid))` if the file exists and the process is alive.
/// - `Ok(None)` if the file does not exist or the process is dead.
/// - `Err` on parse or I/O errors unrelated to the file being absent.
pub async fn check_pid_lock(lock_path: &Path) -> Result<Option<u32>, DaemonError> {
    let contents = match tokio::fs::read_to_string(lock_path).await {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(DaemonError::Io(e)),
    };

    let pid: u32 = contents
        .trim()
        .parse()
        .map_err(|_| DaemonError::Ipc(format!("invalid PID in lock file: {:?}", contents.trim())))?;

    if is_process_alive(pid) {
        Ok(Some(pid))
    } else {
        Ok(None)
    }
}

/// Remove the PID lock file at `lock_path` (best-effort, no error on missing).
pub async fn remove_pid_lock(lock_path: &Path) -> Result<(), DaemonError> {
    match tokio::fs::remove_file(lock_path).await {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(DaemonError::Io(e)),
    }
}

// ── Platform helpers (pure) ───────────────────────────────────────────────────

/// Returns the effective UID on Unix, or the process ID on other platforms.
#[cfg(unix)]
fn platform_uid() -> u32 {
    // SAFETY: getuid(2) is always safe and has no failure mode.
    #[cfg(target_os = "linux")]
    {
        unsafe extern "C" {
            fn getuid() -> u32;
        }
        // SAFETY: pure system call.
        unsafe { getuid() }
    }
    #[cfg(not(target_os = "linux"))]
    {
        // macOS: use process ID as UID proxy for the socket directory name.
        // Real UID would require nix/libc crate; PID is unique enough for
        // the socket-dir name.
        std::process::id()
    }
}

/// Returns `true` if the OS reports that process `pid` is alive.
///
/// On Unix: sends signal 0 (no-op probe).
/// On other platforms: always returns `false` (conservative).
fn is_process_alive(pid: u32) -> bool {
    #[cfg(unix)]
    {
        // kill(pid, 0) returns 0 if the process exists, ESRCH if not.
        // SAFETY: kill with sig=0 is a standard POSIX probe.
        let ret = unsafe {
            libc_kill(pid as i32, 0)
        };
        ret == 0
    }
    #[cfg(not(unix))]
    {
        let _ = pid;
        false
    }
}

#[cfg(unix)]
unsafe fn libc_kill(pid: i32, sig: i32) -> i32 {
    unsafe extern "C" {
        fn kill(pid: i32, sig: i32) -> i32;
    }
    // SAFETY: caller already marks this function unsafe; kill(2) is a
    // standard POSIX probe when sig=0.
    unsafe { kill(pid, sig) }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use super::*;
    use std::env;

    // ── AC-004: socket path configuration ────────────────────────────────────

    /// Default path ends with daemon.sock and contains forge-mcp.
    #[test]
    fn test_BC_1_03_003_socket_path_configuration_default() {
        // Temporarily unset FORGE_DAEMON_SOCKET to test the default.
        let _guard = EnvGuard::unset("FORGE_DAEMON_SOCKET");

        let path = daemon_socket_path();
        let s = path.to_string_lossy();

        assert!(
            s.contains("forge-mcp"),
            "default socket path must contain 'forge-mcp': {s}"
        );
        // On Unix the default ends with daemon.sock; on Windows it's a pipe.
        #[cfg(unix)]
        assert!(
            s.ends_with("daemon.sock"),
            "Unix socket path must end with 'daemon.sock': {s}"
        );
    }

    /// FORGE_DAEMON_SOCKET env var is respected.
    #[test]
    fn test_BC_1_03_003_socket_path_configuration_env_override() {
        let _guard = EnvGuard::set("FORGE_DAEMON_SOCKET", "/tmp/my-custom.sock");

        let path = daemon_socket_path();
        assert_eq!(path, PathBuf::from("/tmp/my-custom.sock"));
    }

    /// XDG_RUNTIME_DIR is used on Unix when FORGE_DAEMON_SOCKET is absent.
    #[cfg(unix)]
    #[test]
    fn test_xdg_runtime_dir_used_when_set() {
        let _forge_guard = EnvGuard::unset("FORGE_DAEMON_SOCKET");
        let _xdg_guard = EnvGuard::set("XDG_RUNTIME_DIR", "/run/user/1000");

        let path = daemon_socket_path();
        assert!(
            path.starts_with("/run/user/1000"),
            "path must start with XDG_RUNTIME_DIR: {:?}",
            path
        );
        assert!(path.ends_with("daemon.sock"));
    }

    // ── pid_lock_path ─────────────────────────────────────────────────────────

    #[test]
    fn test_pid_lock_path_adjacent_to_socket() {
        let socket = PathBuf::from("/tmp/forge-mcp-123/daemon.sock");
        let lock = pid_lock_path(&socket);
        assert_eq!(lock, PathBuf::from("/tmp/forge-mcp-123/daemon.pid"));
    }

    // ── AC-001: stale socket detection ───────────────────────────────────────

    /// No socket file → resolve_socket_conflict returns None (clean start).
    #[cfg(unix)]
    #[tokio::test]
    async fn test_BC_1_03_003_no_socket_clean_start() {
        use tempfile::TempDir;
        let dir = TempDir::new().expect("tempdir");
        let sock = dir.path().join("daemon.sock");

        // Socket does not exist.
        assert!(!sock.exists());

        let result = resolve_socket_conflict(&sock).await.expect("no error");
        assert!(
            result.is_none(),
            "no socket → should return None (clean start)"
        );
    }

    /// Stale socket (file exists, nothing listening) → ReplacedStale.
    #[cfg(unix)]
    #[tokio::test]
    async fn test_BC_1_03_003_stale_socket_removed() {
        use tempfile::TempDir;
        let dir = TempDir::new().expect("tempdir");
        let sock = dir.path().join("daemon.sock");

        // Write a dummy socket file (just a regular file is enough — nothing
        // is listening on it, so connect will fail with ConnectionRefused).
        tokio::fs::write(&sock, b"stale").await.expect("write stale");

        let result = resolve_socket_conflict(&sock)
            .await
            .expect("no error")
            .expect("should be Some");

        match result {
            SocketConflictResolution::ReplacedStale => {}
            other => panic!("expected ReplacedStale, got {other:?}"),
        }

        // File must have been removed.
        assert!(!sock.exists(), "stale socket file must be removed");
    }

    /// Live daemon (real UnixListener) → UseExisting.
    #[cfg(unix)]
    #[tokio::test]
    async fn test_BC_1_03_003_live_daemon_detected() {
        use tempfile::TempDir;
        use tokio::net::UnixListener;

        let dir = TempDir::new().expect("tempdir");
        let sock = dir.path().join("daemon.sock");

        // Bind a real listener so connect() succeeds.
        let _listener = UnixListener::bind(&sock).expect("bind");

        let result = resolve_socket_conflict(&sock)
            .await
            .expect("no error")
            .expect("should be Some");

        match result {
            SocketConflictResolution::UseExisting => {}
            other => panic!("expected UseExisting, got {other:?}"),
        }

        // Socket must still be there — we did NOT remove it.
        assert!(sock.exists(), "live socket must NOT be removed");
    }

    // ── PID lock file helpers ─────────────────────────────────────────────────

    /// write_pid_lock + check_pid_lock round-trip for a living process.
    #[cfg(unix)]
    #[tokio::test]
    async fn test_pid_lock_write_and_check_alive() {
        use tempfile::TempDir;
        let dir = TempDir::new().expect("tempdir");
        let lock = dir.path().join("daemon.pid");

        write_pid_lock(&lock).await.expect("write");
        let pid = check_pid_lock(&lock).await.expect("check");

        // Our own PID — process must be alive.
        assert_eq!(pid, Some(std::process::id()), "own PID must be alive");
    }

    /// check_pid_lock returns None when the lock file contains a dead PID.
    #[cfg(unix)]
    #[tokio::test]
    async fn test_pid_lock_dead_pid_returns_none() {
        use tempfile::TempDir;
        let dir = TempDir::new().expect("tempdir");
        let lock = dir.path().join("daemon.pid");

        // PID 0 is never a valid user-space process — should be dead.
        // Actually PID 0 is the scheduler; use a very high number that
        // almost certainly won't exist.
        tokio::fs::write(&lock, b"9999999").await.expect("write");
        let pid = check_pid_lock(&lock).await.expect("check");
        assert!(pid.is_none(), "dead PID must return None");
    }

    /// check_pid_lock returns None when file is absent.
    #[tokio::test]
    async fn test_pid_lock_missing_file_returns_none() {
        let path = PathBuf::from("/tmp/forge-mcp-test-missing-888.pid");
        // Make sure it's gone.
        let _ = tokio::fs::remove_file(&path).await;

        let result = check_pid_lock(&path).await.expect("no error");
        assert!(result.is_none());
    }

    // ── remove_pid_lock idempotent ────────────────────────────────────────────

    #[tokio::test]
    async fn test_remove_pid_lock_idempotent() {
        let path = PathBuf::from("/tmp/forge-mcp-test-remove-idempotent-888.pid");
        // Remove even if absent — must not error.
        remove_pid_lock(&path).await.expect("idempotent remove");
    }

    // ── Helper: environment variable guard ───────────────────────────────────

    /// RAII guard that restores an environment variable on drop.
    struct EnvGuard {
        key: &'static str,
        old: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let old = env::var(key).ok();
            // SAFETY: tests using EnvGuard must run single-threaded or with
            // #[tokio::test] (each test gets its own runtime; env mutation is
            // safe within a single test process as long as tests don't share env).
            unsafe { env::set_var(key, value) };
            Self { key, old }
        }

        fn unset(key: &'static str) -> Self {
            let old = env::var(key).ok();
            // SAFETY: see above.
            unsafe { env::remove_var(key) };
            Self { key, old }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.old {
                // SAFETY: restoring env on drop; safe within test process.
                Some(v) => unsafe { env::set_var(self.key, v) },
                None => unsafe { env::remove_var(self.key) },
            }
        }
    }
}
