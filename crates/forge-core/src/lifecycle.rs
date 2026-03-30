//! Connection lifecycle management — keepalive, clean shutdown, reconnection.
//!
//! This module implements STORY-009 (AC-001–AC-004, BC-1.02.003).
//!
//! ## Design overview
//!
//! ```text
//! ConnectionManager
//!   ├─ ConnectionEntry (per connection)
//!   │   ├─ McpConnection  (rmcp handle)
//!   │   ├─ ConnectionState (pure state machine)
//!   │   └─ LifecycleTask   (tokio task: keepalive loop)
//!   └─ ConnectionLifecycleConfig  (timeouts, intervals)
//! ```
//!
//! ### Keepalive (AC-001)
//!
//! A background task sends `ping` every `keepalive_interval`. After
//! `max_missed_pings` consecutive failures the connection is marked
//! `Error("keepalive timeout")`.
//!
//! ### Graceful shutdown (AC-002)
//!
//! `ConnectionManager::remove()` / `McpConnection::close_graceful()` transitions
//! the connection to `Disconnecting`, tells rmcp to close (which sends shutdown
//! notification), and waits up to `graceful_shutdown_timeout` for the service to
//! exit before forcibly cancelling.
//!
//! ### State machine (AC-003)
//!
//! Pure transitions live in `connection.rs`; this module drives them.
//!
//! ### Reconnect (AC-004)
//!
//! `ConnectionManager::reconnect()` calls `start_reconnecting()` on the state
//! machine (Error → Connecting) then delegates to the provided factory closure
//! to open a fresh transport and promotes the result to Connected.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tracing::{debug, warn};

use crate::connection::ConnectionState;
use crate::error::{CoreError, Result};

// ── Configuration ─────────────────────────────────────────────────────────────

/// Configuration for the lifecycle manager.
///
/// All fields have sensible defaults matching the story spec.
#[derive(Debug, Clone)]
pub struct ConnectionLifecycleConfig {
    /// How often to send a keepalive ping (default: 30 s).
    pub keepalive_interval: Duration,
    /// Number of consecutive missed pings before marking connection dead.
    pub max_missed_pings: u32,
    /// Graceful-shutdown timeout: if the server doesn't exit within this window
    /// the transport is forcibly cancelled (default: 5 s per spec EC-001).
    pub graceful_shutdown_timeout: Duration,
}

impl Default for ConnectionLifecycleConfig {
    fn default() -> Self {
        Self {
            keepalive_interval: Duration::from_secs(30),
            max_missed_pings: 3,
            graceful_shutdown_timeout: Duration::from_secs(5),
        }
    }
}

// ── LifecycleState ────────────────────────────────────────────────────────────

/// Live state for a single managed connection.
///
/// All fields are `Arc`-shared so the background keepalive task can update them.
#[derive(Debug)]
struct LifecycleEntry {
    /// The current connection state.
    state: ConnectionState,
    /// Number of consecutive missed pings (reset on success).
    missed_pings: u32,
    /// Handle to the background keepalive task, if running.
    keepalive_handle: Option<JoinHandle<()>>,
}

// ── Shared ping channel ───────────────────────────────────────────────────────

/// Minimal abstraction used by the keepalive task to issue a ping.
///
/// In production the closure calls `peer.send_request(ClientRequest::PingRequest(...))`.
/// In tests it can be a mock that counts calls and returns a configurable result.
pub type PingFn = Arc<dyn Fn() -> tokio::task::JoinHandle<bool> + Send + Sync + 'static>;

// ── ConnectionManager ─────────────────────────────────────────────────────────

/// Manages the lifecycle of one or more logical MCP connections.
///
/// Each connection is identified by a `String` key (e.g., the server label or URL).
pub struct ConnectionManager {
    config: ConnectionLifecycleConfig,
    entries: Arc<Mutex<HashMap<String, LifecycleEntry>>>,
}

impl ConnectionManager {
    /// Create a new `ConnectionManager` with the given configuration.
    pub fn new(config: ConnectionLifecycleConfig) -> Self {
        Self {
            config,
            entries: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Create a new `ConnectionManager` with default configuration.
    pub fn with_defaults() -> Self {
        Self::new(ConnectionLifecycleConfig::default())
    }

    /// Register an already-connected connection under `key`.
    ///
    /// Spawns the keepalive task using `ping_fn` to probe liveness.
    pub async fn register(&self, key: impl Into<String>, ping_fn: PingFn) {
        let key = key.into();
        let entry = LifecycleEntry {
            state: ConnectionState::Connected,
            missed_pings: 0,
            keepalive_handle: None,
        };

        let mut entries = self.entries.lock().await;
        entries.insert(key.clone(), entry);
        drop(entries); // release lock before spawning

        self.spawn_keepalive(key, ping_fn).await;
    }

    /// Transition a connection to `Disconnecting` and stop its keepalive task.
    ///
    /// Returns `Ok(())` on success.  Callers are responsible for actually
    /// calling `McpConnection::close()` on the connection handle.
    pub async fn begin_shutdown(&self, key: &str) -> Result<()> {
        let mut entries = self.entries.lock().await;
        let entry = entries.get_mut(key).ok_or_else(|| {
            CoreError::Protocol(format!("no managed connection with key {key:?}"))
        })?;

        // No-op if already disconnected (EC-002).
        if matches!(
            entry.state,
            ConnectionState::Disconnected | ConnectionState::Disconnecting
        ) {
            return Ok(());
        }

        entry.state = entry.state.clone().start_disconnecting();

        // Stop keepalive.
        if let Some(handle) = entry.keepalive_handle.take() {
            handle.abort();
        }

        Ok(())
    }

    /// Mark a connection as `Disconnected` after its transport has been closed.
    pub async fn finish_shutdown(&self, key: &str) {
        let mut entries = self.entries.lock().await;
        if let Some(entry) = entries.get_mut(key) {
            entry.state = entry.state.clone().mark_disconnected();
        }
    }

    /// Transition a connection from `Error` back to `Connecting`.
    ///
    /// The caller must subsequently open a new transport and call
    /// [`mark_reconnected`] to advance to `Connected`.
    pub async fn start_reconnect(&self, key: &str) -> Result<()> {
        let mut entries = self.entries.lock().await;
        let entry = entries.get_mut(key).ok_or_else(|| {
            CoreError::Protocol(format!("no managed connection with key {key:?}"))
        })?;

        // Only valid from Error state.
        if !matches!(entry.state, ConnectionState::Error(_)) {
            return Err(CoreError::Protocol(format!(
                "reconnect only valid from Error state; current state: {}",
                entry.state
            )));
        }

        entry.state = entry.state.clone().start_reconnecting();
        entry.missed_pings = 0;
        Ok(())
    }

    /// Advance a connection from `Connecting` to `Connected` and restart keepalive.
    pub async fn mark_reconnected(&self, key: &str, ping_fn: PingFn) -> Result<()> {
        {
            let mut entries = self.entries.lock().await;
            let entry = entries.get_mut(key).ok_or_else(|| {
                CoreError::Protocol(format!("no managed connection with key {key:?}"))
            })?;

            entry.state = entry.state.clone().mark_connected();
            entry.missed_pings = 0;
        }

        self.spawn_keepalive(key.to_string(), ping_fn).await;
        Ok(())
    }

    /// Query the current state of a connection.
    pub async fn state(&self, key: &str) -> Option<ConnectionState> {
        let entries = self.entries.lock().await;
        entries.get(key).map(|e| e.state.clone())
    }

    /// Remove a connection entry entirely.
    pub async fn remove(&self, key: &str) {
        let mut entries = self.entries.lock().await;
        if let Some(entry) = entries.remove(key)
            && let Some(handle) = entry.keepalive_handle
        {
            handle.abort();
        }
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    /// Spawn the background keepalive task for `key`.
    async fn spawn_keepalive(&self, key: String, ping_fn: PingFn) {
        let entries_arc = Arc::clone(&self.entries);
        let interval = self.config.keepalive_interval;
        let max_missed = self.config.max_missed_pings;

        // Clone key for use inside the task; keep original for the store step.
        let key_task = key.clone();

        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            ticker.tick().await; // skip the first immediate tick

            loop {
                ticker.tick().await;

                // Run ping in a separate task so we can time it out.
                let ping_handle = (ping_fn)();
                let success = matches!(timeout(interval, ping_handle).await, Ok(Ok(true)));

                let mut entries = entries_arc.lock().await;
                let entry = match entries.get_mut(&key_task) {
                    Some(e) => e,
                    None => break, // connection was removed
                };

                if success {
                    debug!("keepalive ping OK for {:?}", key_task);
                    entry.missed_pings = 0;
                } else {
                    entry.missed_pings += 1;
                    warn!(
                        "keepalive ping missed for {:?} ({}/{})",
                        key_task, entry.missed_pings, max_missed
                    );

                    if entry.missed_pings >= max_missed {
                        warn!(
                            "marking connection {:?} as dead after {} missed pings",
                            key_task, max_missed
                        );
                        entry.state = entry
                            .state
                            .clone()
                            .mark_error(format!("keepalive timeout after {max_missed} missed pings"));
                        break;
                    }
                }
            }
        });

        // Store the handle.
        let mut entries = self.entries.lock().await;
        if let Some(entry) = entries.get_mut(&key) {
            // Abort any previous keepalive first.
            if let Some(old) = entry.keepalive_handle.replace(handle) {
                old.abort();
            }
        }
    }
}

// ── Graceful shutdown helper ──────────────────────────────────────────────────

/// Gracefully close an `McpConnection` with a timeout.
///
/// Calls `close()` on `service_close_fn` (which sends the shutdown notification),
/// waiting up to `graceful_shutdown_timeout`.  If the timeout is exceeded the
/// future is dropped (rmcp's drop guard cancels the background task).
pub async fn close_with_timeout<F, Fut>(
    close_fn: F,
    graceful_shutdown_timeout: Duration,
) -> Result<()>
where
    F: FnOnce() -> Fut + Send,
    Fut: std::future::Future<Output = Result<()>> + Send,
{
    match timeout(graceful_shutdown_timeout, close_fn()).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_elapsed) => {
            // Timeout — transport is dropped; rmcp's DropGuard cancels the task.
            warn!(
                "graceful shutdown timed out after {:.1}s — forcing close",
                graceful_shutdown_timeout.as_secs_f64()
            );
            Ok(()) // forced close is acceptable (EC-001)
        }
    }
}

// ── Pure state-machine tests (AC-003, AC-004) ─────────────────────────────────

#[cfg(test)]
mod pure_tests {
    #![allow(non_snake_case)]
    use crate::connection::ConnectionState;

    /// AC-003: Valid state machine transitions complete without panic.
    #[test]
    fn test_BC_1_02_003_state_machine_valid_transitions() {
        // Disconnected → Connecting → Connected → Disconnecting → Disconnected
        let s = ConnectionState::Disconnected.start_connecting();
        assert_eq!(s, ConnectionState::Connecting);
        let s = s.mark_connected();
        assert_eq!(s, ConnectionState::Connected);
        let s = s.start_disconnecting();
        assert_eq!(s, ConnectionState::Disconnecting);
        let s = s.mark_disconnected();
        assert_eq!(s, ConnectionState::Disconnected);
    }

    /// AC-004: Error → Connecting (reconnect), previous error is cleared.
    #[test]
    fn test_BC_1_02_003_reconnect_from_error() {
        let s = ConnectionState::Connected.mark_error("network failure");
        assert!(matches!(s, ConnectionState::Error(_)));
        // Reconnect clears the error.
        let s = s.start_reconnecting();
        assert_eq!(s, ConnectionState::Connecting);
    }

    /// No-op: close called on already-disconnected connection (EC-002).
    #[test]
    fn test_close_on_disconnected_is_noop() {
        let s = ConnectionState::Disconnected;
        // start_disconnecting on Disconnected should be a no-op.
        let s2 = s.start_disconnecting();
        assert_eq!(s2, ConnectionState::Disconnected);
    }

    /// mark_error from any state.
    #[test]
    fn test_error_from_connecting() {
        let s = ConnectionState::Connecting.mark_error("failed");
        assert_eq!(s, ConnectionState::Error("failed".to_string()));
    }
}

// ── Integration tests ─────────────────────────────────────────────────────────

#[cfg(test)]
mod integration_tests {
    #![allow(non_snake_case)]
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use tokio::time::sleep;

    /// Helper: build a ping_fn that always succeeds, counting calls.
    fn always_succeed_ping(counter: Arc<AtomicU32>) -> PingFn {
        Arc::new(move || {
            let c = Arc::clone(&counter);
            tokio::spawn(async move {
                c.fetch_add(1, Ordering::Relaxed);
                true
            })
        })
    }

    /// Helper: build a ping_fn that always fails.
    fn always_fail_ping(counter: Arc<AtomicU32>) -> PingFn {
        Arc::new(move || {
            let c = Arc::clone(&counter);
            tokio::spawn(async move {
                c.fetch_add(1, Ordering::Relaxed);
                false
            })
        })
    }

    /// AC-001: keepalive pings keep the connection alive; state stays Connected.
    #[tokio::test]
    async fn test_BC_1_02_003_keepalive_maintains_connection() {
        let config = ConnectionLifecycleConfig {
            keepalive_interval: Duration::from_millis(50),
            max_missed_pings: 3,
            graceful_shutdown_timeout: Duration::from_secs(1),
        };
        let mgr = ConnectionManager::new(config);
        let call_count = Arc::new(AtomicU32::new(0));

        mgr.register("srv1", always_succeed_ping(Arc::clone(&call_count)))
            .await;

        // Wait long enough for at least 3 pings.
        sleep(Duration::from_millis(300)).await;

        let state = mgr.state("srv1").await.expect("state should exist");
        assert_eq!(state, ConnectionState::Connected, "state must remain Connected");
        assert!(
            call_count.load(Ordering::Relaxed) >= 2,
            "at least 2 keepalive pings should have fired"
        );

        mgr.remove("srv1").await;
    }

    /// Dead server detected: after max_missed_pings the state becomes Error.
    #[tokio::test]
    async fn test_BC_1_02_003_dead_server_detected_after_timeout() {
        let config = ConnectionLifecycleConfig {
            keepalive_interval: Duration::from_millis(30),
            max_missed_pings: 3,
            graceful_shutdown_timeout: Duration::from_secs(1),
        };
        let mgr = ConnectionManager::new(config);
        let fail_count = Arc::new(AtomicU32::new(0));

        mgr.register("dead-srv", always_fail_ping(Arc::clone(&fail_count)))
            .await;

        // Wait for max_missed_pings (3) × interval (30ms) plus margin.
        sleep(Duration::from_millis(300)).await;

        let state = mgr.state("dead-srv").await.expect("state should exist");
        assert!(
            matches!(state, ConnectionState::Error(_)),
            "connection must be Error after {max_missed} missed pings; got: {state:?}",
            max_missed = 3,
            state = state,
        );

        mgr.remove("dead-srv").await;
    }

    /// AC-002: begin_shutdown transitions to Disconnecting; finish_shutdown to Disconnected.
    #[tokio::test]
    async fn test_BC_1_02_003_graceful_shutdown() {
        let config = ConnectionLifecycleConfig {
            keepalive_interval: Duration::from_millis(1000), // long — shouldn't fire during test
            max_missed_pings: 3,
            graceful_shutdown_timeout: Duration::from_millis(200),
        };
        let mgr = ConnectionManager::new(config);
        let call_count = Arc::new(AtomicU32::new(0));

        mgr.register("srv2", always_succeed_ping(Arc::clone(&call_count)))
            .await;

        // Verify Connected initially.
        let state = mgr.state("srv2").await.unwrap();
        assert_eq!(state, ConnectionState::Connected);

        // Initiate graceful shutdown.
        mgr.begin_shutdown("srv2").await.expect("begin_shutdown should succeed");

        let state = mgr.state("srv2").await.unwrap();
        assert_eq!(state, ConnectionState::Disconnecting);

        // Simulate transport closed.
        mgr.finish_shutdown("srv2").await;

        let state = mgr.state("srv2").await.unwrap();
        assert_eq!(state, ConnectionState::Disconnected);
    }

    /// EC-002: shutdown on already-disconnected is a no-op.
    #[tokio::test]
    async fn test_close_on_disconnected_connection_is_noop() {
        let mgr = ConnectionManager::with_defaults();
        // Manually insert a Disconnected entry.
        {
            let mut entries = mgr.entries.lock().await;
            entries.insert(
                "disc-srv".to_string(),
                LifecycleEntry {
                    state: ConnectionState::Disconnected,
                    missed_pings: 0,
                    keepalive_handle: None,
                },
            );
        }

        // Should succeed without error.
        mgr.begin_shutdown("disc-srv")
            .await
            .expect("begin_shutdown on Disconnected must be no-op");

        let state = mgr.state("disc-srv").await.unwrap();
        // Still Disconnected — no transition.
        assert_eq!(state, ConnectionState::Disconnected);
    }

    /// AC-004: reconnect from Error state clears the error and advances to Connecting.
    #[tokio::test]
    async fn test_BC_1_02_003_reconnect_from_error() {
        let mgr = ConnectionManager::with_defaults();

        // Insert an Error entry.
        {
            let mut entries = mgr.entries.lock().await;
            entries.insert(
                "err-srv".to_string(),
                LifecycleEntry {
                    state: ConnectionState::Error("previous failure".to_string()),
                    missed_pings: 2,
                    keepalive_handle: None,
                },
            );
        }

        // Reconnect transitions Error → Connecting.
        mgr.start_reconnect("err-srv")
            .await
            .expect("start_reconnect should succeed from Error state");

        let state = mgr.state("err-srv").await.unwrap();
        assert_eq!(state, ConnectionState::Connecting, "state must be Connecting after reconnect");

        // Now simulate successful reconnect.
        let call_count = Arc::new(AtomicU32::new(0));
        mgr.mark_reconnected("err-srv", always_succeed_ping(Arc::clone(&call_count)))
            .await
            .expect("mark_reconnected should succeed");

        let state = mgr.state("err-srv").await.unwrap();
        assert_eq!(state, ConnectionState::Connected, "state must be Connected after mark_reconnected");

        mgr.remove("err-srv").await;
    }

    /// Reconnect from non-Error state should return an error.
    #[tokio::test]
    async fn test_reconnect_from_connected_is_error() {
        let mgr = ConnectionManager::with_defaults();

        {
            let mut entries = mgr.entries.lock().await;
            entries.insert(
                "conn-srv".to_string(),
                LifecycleEntry {
                    state: ConnectionState::Connected,
                    missed_pings: 0,
                    keepalive_handle: None,
                },
            );
        }

        let result = mgr.start_reconnect("conn-srv").await;
        assert!(
            result.is_err(),
            "start_reconnect from Connected must return an error"
        );
    }

    /// `close_with_timeout` completes cleanly when the close_fn succeeds.
    #[tokio::test]
    async fn test_close_with_timeout_clean_shutdown() {
        let result = close_with_timeout(
            || async { Ok(()) },
            Duration::from_secs(1),
        )
        .await;

        assert!(result.is_ok(), "clean shutdown should return Ok");
    }

    /// `close_with_timeout` returns Ok when the close_fn times out (forced close, EC-001).
    #[tokio::test]
    async fn test_close_with_timeout_forced_on_timeout() {
        let result = close_with_timeout(
            || async {
                // Simulate a server that never responds.
                sleep(Duration::from_secs(60)).await;
                Ok(())
            },
            Duration::from_millis(50),
        )
        .await;

        // Forced close is acceptable — no error returned.
        assert!(result.is_ok(), "forced close should return Ok");
    }
}
