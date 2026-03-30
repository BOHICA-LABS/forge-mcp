//! Session pool — maps server name → pooled connection entry.
//!
//! ## Design
//!
//! The `SessionPool` maintains a `HashMap<String, PoolEntry>` guarded by a
//! `tokio::sync::Mutex`. Each `PoolEntry` tracks:
//! - The connection state (`Connected` | `Error` | …)
//! - Last-used timestamp (for idle timeout, AC-003)
//! - A `ConnectionFactory` closure that can (re)create connections
//!
//! Since this story targets the daemon pooling layer — not actual MCP transport
//! (which requires a running MCP server) — the pool works with abstract
//! "connection handles" represented by the `PoolableConnection` trait. In
//! production the CLI layer wraps a real `McpConnection` behind this trait;
//! in tests a mock is used.
//!
//! ### Multiplexing (AC-004 / VP-015)
//!
//! Multiple callers requesting the same server name receive a `SessionHandle`
//! that holds an `Arc` reference to the shared `PoolEntry`. No two callers
//! ever force a second physical connection to the same server while one is live.
//!
//! ### Named Sessions (STORY-011 / BC-1.03.002)
//!
//! A secondary index `HashMap<String, String>` maps user-provided session names
//! to server-name keys in the primary pool. Named sessions survive across CLI
//! invocations as long as the daemon process remains alive. The `NamedSession`
//! struct captures the name, server, creation time, and last activity time for
//! the `daemon sessions` list command (AC-004).

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::Mutex;
use tracing::{debug, info, warn};

use crate::error::DaemonError;
use crate::session::SessionId;

// ── Named session ─────────────────────────────────────────────────────────────

/// Metadata for a user-named session (STORY-011, BC-1.03.002).
///
/// A named session is an overlay on the pool: it gives a human-readable alias
/// to an existing pool entry (identified by `server_name`). Multiple names can
/// point to the same server, though they will all share the single pooled
/// connection.
#[derive(Debug, Clone)]
pub struct NamedSession {
    /// The user-provided name (e.g. `"my-debug"`).
    pub name: String,
    /// The MCP server this session targets.
    pub server_name: String,
    /// When the named session was registered.
    pub created_at: Instant,
    /// When the named session was last accessed.
    pub last_activity: Instant,
}

impl NamedSession {
    fn new(name: impl Into<String>, server_name: impl Into<String>) -> Self {
        let now = Instant::now();
        Self {
            name: name.into(),
            server_name: server_name.into(),
            created_at: now,
            last_activity: now,
        }
    }

    fn touch(&mut self) {
        self.last_activity = Instant::now();
    }
}

// ── Pool configuration ────────────────────────────────────────────────────────

/// Configuration for the `SessionPool`.
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Connections idle longer than this are evicted (AC-003, default 5 min).
    pub idle_timeout: Duration,
    /// Maximum number of connections in the pool (EC-003).
    pub max_connections: usize,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            idle_timeout: Duration::from_secs(300), // 5 minutes
            max_connections: 32,
        }
    }
}

// ── Abstract connection ───────────────────────────────────────────────────────

/// Minimal interface the pool requires from a connection.
///
/// This is the seam between the pool and the real `McpConnection`. In
/// production use `RealConnection`; in tests use `MockConnection`.
pub trait PoolableConnection: Send + Sync + 'static {
    /// Returns `true` if the connection is still alive.
    fn is_alive(&self) -> bool;
    /// Server name / label for logging.
    fn label(&self) -> &str;
}

/// A factory closure that creates a new connection for `server`.
pub type ConnectionFactory =
    Arc<dyn Fn(&str) -> BoxFuture<Result<Box<dyn PoolableConnection>, DaemonError>> + Send + Sync>;

/// Boxed future alias to avoid repeating the type bounds.
pub type BoxFuture<T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send>>;

// ── Pool entry ────────────────────────────────────────────────────────────────

struct PoolEntry {
    connection: Box<dyn PoolableConnection>,
    last_used: Instant,
    session_id: SessionId,
    server_name: String,
}

impl PoolEntry {
    fn new(server_name: impl Into<String>, connection: Box<dyn PoolableConnection>) -> Self {
        Self {
            session_id: SessionId::new(),
            server_name: server_name.into(),
            connection,
            last_used: Instant::now(),
        }
    }

    fn touch(&mut self) {
        self.last_used = Instant::now();
    }

    fn is_idle(&self, timeout: Duration) -> bool {
        self.last_used.elapsed() > timeout
    }
}

// ── SessionHandle ─────────────────────────────────────────────────────────────

/// A handle to a pooled session returned to callers.
///
/// Wraps session metadata. The caller can use the `session_id` to correlate
/// requests back to the pool entry.
#[derive(Debug, Clone)]
pub struct SessionHandle {
    /// The unique session ID for this connection slot.
    pub session_id: SessionId,
    /// The server name this handle points to.
    pub server_name: String,
}

// ── SessionPool ───────────────────────────────────────────────────────────────

/// Thread-safe pool of server connections.
///
/// Multiple tokio tasks may call `get_or_create` concurrently; the mutex
/// ensures only one physical connection is created per server (AC-004).
///
/// The pool also maintains a secondary index of named sessions (STORY-011).
/// Named sessions map a user-provided name → server_name key in `entries`.
pub struct SessionPool {
    config: PoolConfig,
    entries: Arc<Mutex<HashMap<String, PoolEntry>>>,
    /// Secondary index: session name → server name.
    named: Arc<Mutex<HashMap<String, NamedSession>>>,
    factory: ConnectionFactory,
}

impl SessionPool {
    /// Create a new pool with the given config and connection factory.
    pub fn new(config: PoolConfig, factory: ConnectionFactory) -> Self {
        Self {
            config,
            entries: Arc::new(Mutex::new(HashMap::new())),
            named: Arc::new(Mutex::new(HashMap::new())),
            factory,
        }
    }

    /// Get an existing live connection for `server`, or create a new one.
    ///
    /// - If a live connection already exists → touch it and return handle (AC-002).
    /// - If the connection is dead → evict it and create a new one (EC-002).
    /// - If no connection exists → create a new one.
    ///
    /// Holds the mutex for the duration of the factory call only when a new
    /// connection must be created, and then only briefly to insert.
    /// Callers block on the same mutex — guaranteeing no duplicate connections
    /// to the same server (AC-004 / VP-015).
    pub async fn get_or_create(&self, server: &str) -> Result<SessionHandle, DaemonError> {
        let mut entries = self.entries.lock().await;

        // 1. Check if a live connection already exists.
        if let Some(entry) = entries.get_mut(server) {
            if entry.connection.is_alive() {
                debug!("pool hit for {server:?}");
                entry.touch();
                return Ok(SessionHandle {
                    session_id: entry.session_id.clone(),
                    server_name: entry.server_name.clone(),
                });
            } else {
                // Dead connection — evict it.
                warn!("evicting dead connection for {server:?}");
                entries.remove(server);
            }
        }

        // 2. Enforce max pool size via LRU eviction (EC-003).
        if entries.len() >= self.config.max_connections {
            self.evict_lru(&mut entries);
        }

        // 3. Create a new connection — we hold the mutex here to prevent
        //    duplicate creation (AC-004).
        info!("creating new connection for {server:?}");
        let conn = (self.factory)(server).await?;
        let entry = PoolEntry::new(server, conn);
        let handle = SessionHandle {
            session_id: entry.session_id.clone(),
            server_name: entry.server_name.clone(),
        };
        entries.insert(server.to_string(), entry);

        Ok(handle)
    }

    /// Explicitly mark a server's connection as dead (e.g., after detecting error).
    ///
    /// The next `get_or_create` call for this server will create a fresh connection.
    pub async fn evict(&self, server: &str) {
        let mut entries = self.entries.lock().await;
        if entries.remove(server).is_some() {
            info!("manually evicted connection for {server:?}");
        }
    }

    /// Evict all connections that have exceeded the idle timeout (AC-003).
    ///
    /// This is intended to be called periodically by the daemon's maintenance loop.
    pub async fn evict_idle(&self) {
        let mut entries = self.entries.lock().await;
        let timeout = self.config.idle_timeout;
        let before = entries.len();
        entries.retain(|server, entry| {
            if entry.is_idle(timeout) {
                info!("evicting idle connection for {server:?}");
                false
            } else {
                true
            }
        });
        let evicted = before - entries.len();
        if evicted > 0 {
            debug!("evicted {evicted} idle connection(s)");
        }
    }

    /// Returns the number of live entries in the pool.
    pub async fn len(&self) -> usize {
        self.entries.lock().await.len()
    }

    /// Returns `true` if the pool is empty.
    pub async fn is_empty(&self) -> bool {
        self.entries.lock().await.is_empty()
    }

    /// Returns the session handle for a server if it exists in the pool (for testing).
    pub async fn get_handle(&self, server: &str) -> Option<SessionHandle> {
        let entries = self.entries.lock().await;
        entries.get(server).map(|e| SessionHandle {
            session_id: e.session_id.clone(),
            server_name: e.server_name.clone(),
        })
    }

    // ── Named session API (STORY-011 / BC-1.03.002) ───────────────────────────

    /// Create a named session backed by the given server.
    ///
    /// - Calls `get_or_create` to ensure the underlying pool entry exists.
    /// - If `name` already exists, **overwrites** with a warning (EC-002).
    /// - Returns the `SessionHandle` for the backing pool entry.
    pub async fn get_or_create_named(
        &self,
        name: &str,
        server: &str,
    ) -> Result<SessionHandle, DaemonError> {
        // Ensure the pool entry exists first.
        let handle = self.get_or_create(server).await?;

        let mut named = self.named.lock().await;
        if named.contains_key(name) {
            warn!("named session {name:?} already exists — overwriting");
        }
        named.insert(name.to_string(), NamedSession::new(name, server));

        Ok(handle)
    }

    /// Retrieve a pooled session handle by name.
    ///
    /// Touches the `NamedSession` last-activity timestamp and the underlying
    /// pool entry. Returns `Err(SessionNotFound)` if the name is unknown
    /// (AC-003 / E-CON-003).
    pub async fn get_by_name(&self, name: &str) -> Result<SessionHandle, DaemonError> {
        // Look up the name in the secondary index.
        let server_name = {
            let mut named = self.named.lock().await;
            match named.get_mut(name) {
                Some(ns) => {
                    ns.touch();
                    ns.server_name.clone()
                }
                None => {
                    return Err(DaemonError::SessionNotFound {
                        name: name.to_string(),
                    });
                }
            }
        };

        // Retrieve (or recreate if dead) the underlying pool entry.
        self.get_or_create(&server_name).await
    }

    /// Remove a named session.
    ///
    /// The backing pool entry is **not** evicted — other names or anonymous
    /// pool lookups may still use it. Returns `true` if the name existed.
    pub async fn remove_named(&self, name: &str) -> bool {
        let mut named = self.named.lock().await;
        if named.remove(name).is_some() {
            info!("removed named session {name:?}");
            true
        } else {
            false
        }
    }

    /// List all active named sessions.
    ///
    /// Returns cloned `NamedSession` metadata sorted by creation time (AC-004).
    pub async fn list_named(&self) -> Vec<NamedSession> {
        let named = self.named.lock().await;
        let mut sessions: Vec<NamedSession> = named.values().cloned().collect();
        sessions.sort_by_key(|s| s.created_at);
        sessions
    }

    // ── Internal helpers ──────────────────────────────────────────────────────

    /// Evict the least-recently-used entry. Called with the lock held.
    fn evict_lru(&self, entries: &mut HashMap<String, PoolEntry>) {
        if let Some(oldest_key) = entries
            .iter()
            .min_by_key(|(_, e)| e.last_used)
            .map(|(k, _)| k.clone())
        {
            entries.remove(&oldest_key);
            warn!("LRU eviction: removed {oldest_key:?} to make room");
        }
    }
}

// ── Mock helpers (always compiled, doc-hidden) ────────────────────────────────
//
// These are in the main module (not behind #[cfg(test)]) so that
// `daemon.rs` tests can import them via `crate::pool::MockConnection`.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/// A mock connection whose liveness can be controlled via `AtomicBool`.
///
/// Used in pool tests and daemon integration tests.
#[doc(hidden)]
pub struct MockConnection {
    pub alive: Arc<AtomicBool>,
    pub name: String,
}

impl MockConnection {
    /// Create an alive mock connection with the given label.
    pub fn alive(label: impl Into<String>) -> Box<dyn PoolableConnection> {
        Box::new(Self {
            alive: Arc::new(AtomicBool::new(true)),
            name: label.into(),
        })
    }

    /// Create a dead mock connection (simulates a lost connection).
    pub fn dead(label: impl Into<String>) -> Box<dyn PoolableConnection> {
        Box::new(Self {
            alive: Arc::new(AtomicBool::new(false)),
            name: label.into(),
        })
    }
}

impl PoolableConnection for MockConnection {
    fn is_alive(&self) -> bool {
        self.alive.load(Ordering::Relaxed)
    }
    fn label(&self) -> &str {
        &self.name
    }
}

/// Build a factory that always returns an alive connection and counts calls.
#[doc(hidden)]
pub fn counting_factory(counter: Arc<AtomicUsize>) -> ConnectionFactory {
    Arc::new(move |server: &str| {
        let c = Arc::clone(&counter);
        let label = server.to_string();
        Box::pin(async move {
            c.fetch_add(1, Ordering::Relaxed);
            Ok(MockConnection::alive(label) as Box<dyn PoolableConnection>)
        })
    })
}

/// Type alias for the queue used by [`queued_factory`] to reduce complexity.
#[doc(hidden)]
pub type ConnectionQueue = Arc<Mutex<Vec<Result<Box<dyn PoolableConnection>, DaemonError>>>>;

/// Build a factory backed by a LIFO queue of pre-built connections.
///
/// Each call pops the next connection from the back. `vec![c1, c2]` → first
/// call returns `c2`, second call returns `c1`.
#[doc(hidden)]
pub fn queued_factory(queue: ConnectionQueue) -> ConnectionFactory {
    Arc::new(move |_server: &str| {
        let q = Arc::clone(&queue);
        Box::pin(async move {
            let mut guard = q.lock().await;
            guard.pop().unwrap_or_else(|| {
                Err(DaemonError::ConnectionFailed(
                    "mock queue exhausted".to_string(),
                ))
            })
        })
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use super::*;

    // ── AC-002: pool reuse ────────────────────────────────────────────────────

    /// AC-002: second request returns the SAME session ID (pool hit).
    #[tokio::test]
    async fn test_BC_1_03_001_session_pool_reuse() {
        let counter = Arc::new(AtomicUsize::new(0));
        let pool = SessionPool::new(PoolConfig::default(), counting_factory(Arc::clone(&counter)));

        let h1 = pool.get_or_create("my-server").await.expect("first request");
        let h2 = pool.get_or_create("my-server").await.expect("second request");

        assert_eq!(
            h1.session_id, h2.session_id,
            "second request must return the same pooled session"
        );
        assert_eq!(
            counter.load(Ordering::Relaxed),
            1,
            "factory must be called exactly once for pool reuse"
        );
    }

    // ── EC-002: dead connection triggers reconnect ────────────────────────────

    /// EC-002: dead connection in pool is evicted; next request creates a new one.
    #[tokio::test]
    async fn test_dead_connection_triggers_reconnect() {
        // Queue (LIFO): pop() returns last element first.
        // Index 1 is popped first (dead), index 0 popped second (alive).
        let queue: Vec<Result<Box<dyn PoolableConnection>, DaemonError>> = vec![
            Ok(MockConnection::alive("new")), // second call
            Ok(MockConnection::dead("dead")), // first call
        ];
        let q = Arc::new(Mutex::new(queue));
        let pool = SessionPool::new(PoolConfig::default(), queued_factory(Arc::clone(&q)));

        let h1 = pool.get_or_create("srv").await.expect("first request");
        let h2 = pool
            .get_or_create("srv")
            .await
            .expect("second request (dead→reconnect)");

        assert_ne!(
            h1.session_id, h2.session_id,
            "after dead eviction, a new session must be created"
        );
        assert_eq!(pool.len().await, 1);
    }

    // ── AC-003: idle timeout eviction ─────────────────────────────────────────

    /// AC-003: idle connections are evicted by `evict_idle`.
    #[tokio::test]
    async fn test_BC_1_03_001_idle_timeout_closes_connection() {
        let counter = Arc::new(AtomicUsize::new(0));
        let config = PoolConfig {
            idle_timeout: Duration::from_millis(10), // very short for test
            max_connections: 32,
        };
        let pool = SessionPool::new(config, counting_factory(Arc::clone(&counter)));

        pool.get_or_create("idle-srv").await.expect("create");
        assert_eq!(pool.len().await, 1);

        tokio::time::sleep(Duration::from_millis(50)).await;

        pool.evict_idle().await;
        assert_eq!(pool.len().await, 0, "idle connection must be evicted");
    }

    // ── AC-004: concurrent requests multiplexed ───────────────────────────────

    /// AC-004: concurrent requests for the same server all get the same session ID.
    #[tokio::test]
    async fn test_BC_1_03_001_concurrent_requests_multiplexed() {
        let counter = Arc::new(AtomicUsize::new(0));
        let pool = Arc::new(SessionPool::new(
            PoolConfig::default(),
            counting_factory(Arc::clone(&counter)),
        ));

        let mut handles = Vec::new();
        for _ in 0..10 {
            let pool_clone = Arc::clone(&pool);
            handles.push(tokio::spawn(async move {
                pool_clone
                    .get_or_create("concurrent-srv")
                    .await
                    .expect("concurrent request")
                    .session_id
            }));
        }

        let session_ids: Vec<SessionId> = futures::future::join_all(handles)
            .await
            .into_iter()
            .map(|r| r.expect("task must not panic"))
            .collect();

        let first = &session_ids[0];
        for id in &session_ids {
            assert_eq!(
                id, first,
                "all concurrent requests must share the same pooled session"
            );
        }
        assert_eq!(
            counter.load(Ordering::Relaxed),
            1,
            "factory must be called exactly once despite concurrent requests"
        );
    }

    // ── LRU eviction (EC-003) ────────────────────────────────────────────────

    #[tokio::test]
    async fn test_lru_eviction_at_max_capacity() {
        let counter = Arc::new(AtomicUsize::new(0));
        let config = PoolConfig {
            idle_timeout: Duration::from_secs(300),
            max_connections: 2,
        };
        let pool = SessionPool::new(config, counting_factory(Arc::clone(&counter)));

        pool.get_or_create("srv-a").await.expect("a");
        pool.get_or_create("srv-b").await.expect("b");
        assert_eq!(pool.len().await, 2);

        pool.get_or_create("srv-c").await.expect("c");
        assert_eq!(pool.len().await, 2, "pool must not exceed max_connections");
    }

    // ── Manual evict ─────────────────────────────────────────────────────────

    #[tokio::test]
    async fn test_manual_evict_removes_entry() {
        let counter = Arc::new(AtomicUsize::new(0));
        let pool = SessionPool::new(PoolConfig::default(), counting_factory(Arc::clone(&counter)));

        pool.get_or_create("srv").await.expect("create");
        assert_eq!(pool.len().await, 1);
        pool.evict("srv").await;
        assert_eq!(pool.len().await, 0);
    }

    // ── Named session tests (STORY-011 / BC-1.03.002) ────────────────────────

    /// AC-001: Create a named session → retrieve it by name → same session ID.
    #[tokio::test]
    async fn test_BC_1_03_002_named_session_creation() {
        let counter = Arc::new(AtomicUsize::new(0));
        let pool = SessionPool::new(PoolConfig::default(), counting_factory(Arc::clone(&counter)));

        // Create a named session.
        let handle = pool
            .get_or_create_named("my-debug", "myserver")
            .await
            .expect("named session creation must succeed");

        assert_eq!(handle.server_name, "myserver");

        // Retrieve by name — must return same session ID.
        let handle2 = pool
            .get_by_name("my-debug")
            .await
            .expect("named session retrieval must succeed");

        assert_eq!(
            handle.session_id, handle2.session_id,
            "get_by_name must return the same pooled session"
        );
        // Factory called exactly once.
        assert_eq!(counter.load(Ordering::Relaxed), 1);
    }

    /// AC-002: Unnamed sessions still work with auto-generated IDs.
    #[tokio::test]
    async fn test_BC_1_03_002_unnamed_sessions_use_auto_id() {
        let counter = Arc::new(AtomicUsize::new(0));
        let pool = SessionPool::new(PoolConfig::default(), counting_factory(Arc::clone(&counter)));

        // Unnamed path — get_or_create by server name.
        let h1 = pool.get_or_create("anon-server").await.expect("first");
        let h2 = pool.get_or_create("anon-server").await.expect("second");

        assert_eq!(
            h1.session_id, h2.session_id,
            "unnamed sessions must reuse the same pool entry"
        );
        assert_eq!(counter.load(Ordering::Relaxed), 1);

        // Named index must be empty.
        assert_eq!(pool.list_named().await.len(), 0);
    }

    /// AC-002: Two different names produce two independent session entries.
    #[tokio::test]
    async fn test_BC_1_03_002_named_session_resume() {
        let counter = Arc::new(AtomicUsize::new(0));
        let pool = SessionPool::new(PoolConfig::default(), counting_factory(Arc::clone(&counter)));

        let h_a1 = pool
            .get_or_create_named("session-a", "server-a")
            .await
            .expect("create session-a");
        let h_b1 = pool
            .get_or_create_named("session-b", "server-b")
            .await
            .expect("create session-b");

        // They point to different servers → different session IDs.
        assert_ne!(
            h_a1.session_id, h_b1.session_id,
            "two different names on different servers must yield different sessions"
        );

        // Re-fetch by name — must get same IDs back.
        let h_a2 = pool.get_by_name("session-a").await.expect("resume session-a");
        let h_b2 = pool.get_by_name("session-b").await.expect("resume session-b");

        assert_eq!(h_a1.session_id, h_a2.session_id, "session-a must be stable across get_by_name");
        assert_eq!(h_b1.session_id, h_b2.session_id, "session-b must be stable across get_by_name");
        // Factory called exactly twice (one per server).
        assert_eq!(counter.load(Ordering::Relaxed), 2);
    }

    /// AC-003: Looking up a non-existent named session returns SessionNotFound.
    #[tokio::test]
    async fn test_BC_1_03_002_session_not_found() {
        let counter = Arc::new(AtomicUsize::new(0));
        let pool = SessionPool::new(PoolConfig::default(), counting_factory(Arc::clone(&counter)));

        let result = pool.get_by_name("does-not-exist").await;
        assert!(
            result.is_err(),
            "looking up non-existent named session must return an error"
        );

        let err = result.unwrap_err();
        assert!(
            err.to_string().contains("does-not-exist"),
            "error message must include the session name: {err}"
        );
        // Matches SessionNotFound variant.
        assert!(
            matches!(err, DaemonError::SessionNotFound { ref name } if name == "does-not-exist"),
            "must be SessionNotFound variant"
        );
    }

    /// Close a named session → no longer retrievable.
    #[tokio::test]
    async fn test_close_named_session_not_retrievable() {
        let counter = Arc::new(AtomicUsize::new(0));
        let pool = SessionPool::new(PoolConfig::default(), counting_factory(Arc::clone(&counter)));

        pool.get_or_create_named("to-close", "some-server")
            .await
            .expect("create");

        let removed = pool.remove_named("to-close").await;
        assert!(removed, "remove_named must return true for existing session");

        // Subsequent lookup must fail.
        let result = pool.get_by_name("to-close").await;
        assert!(
            result.is_err(),
            "closed named session must not be retrievable"
        );
        assert!(matches!(
            result.unwrap_err(),
            DaemonError::SessionNotFound { .. }
        ));
    }

    /// AC-004: list_named returns all active named sessions.
    #[tokio::test]
    async fn test_BC_1_03_002_session_list_json() {
        let counter = Arc::new(AtomicUsize::new(0));
        let pool = SessionPool::new(PoolConfig::default(), counting_factory(Arc::clone(&counter)));

        // Start with empty list.
        assert_eq!(pool.list_named().await.len(), 0);

        pool.get_or_create_named("alpha", "srv-alpha").await.expect("alpha");
        pool.get_or_create_named("beta", "srv-beta").await.expect("beta");
        pool.get_or_create_named("gamma", "srv-gamma").await.expect("gamma");

        let sessions = pool.list_named().await;
        assert_eq!(sessions.len(), 3, "list must show all active named sessions");

        let names: Vec<&str> = sessions.iter().map(|s| s.name.as_str()).collect();
        assert!(names.contains(&"alpha"), "list must include alpha");
        assert!(names.contains(&"beta"), "list must include beta");
        assert!(names.contains(&"gamma"), "list must include gamma");

        // Remove one — list shrinks.
        pool.remove_named("beta").await;
        let sessions = pool.list_named().await;
        assert_eq!(sessions.len(), 2, "list must shrink after removal");
        let names: Vec<&str> = sessions.iter().map(|s| s.name.as_str()).collect();
        assert!(!names.contains(&"beta"), "removed session must not appear");
    }

    /// EC-002: Creating a named session with an existing name overwrites it (with warning).
    #[tokio::test]
    async fn test_named_session_overwrite_on_duplicate_name() {
        let counter = Arc::new(AtomicUsize::new(0));
        let pool = SessionPool::new(PoolConfig::default(), counting_factory(Arc::clone(&counter)));

        // Create "my-session" pointing at server-a.
        pool.get_or_create_named("my-session", "server-a")
            .await
            .expect("first create");

        // Overwrite with server-b.
        pool.get_or_create_named("my-session", "server-b")
            .await
            .expect("second create overwrites");

        // Now list should show only 1 session.
        let sessions = pool.list_named().await;
        assert_eq!(sessions.len(), 1, "overwrite must not duplicate the name");
        assert_eq!(sessions[0].server_name, "server-b", "overwritten session must point to new server");
    }
}
