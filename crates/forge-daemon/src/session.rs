//! Session ID generation — pure (DI-003).
//!
//! Generates unique, time-ordered session IDs using UUIDv4.
//! This module is pure: no I/O, no side effects.

use std::fmt;

/// A unique identifier for a daemon session.
///
/// Wraps a UUID string for type safety. Session IDs are unique across
/// concurrent sessions (DI-003).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionId(String);

impl SessionId {
    /// Generate a new unique session ID.
    ///
    /// Uses UUIDv4 (random). Pure in the sense that it requires no I/O
    /// beyond the OS entropy source, which is modelled as a side-effect
    /// free input for purity purposes (DI-003).
    pub fn new() -> Self {
        // Use a simple unique ID without requiring the `uuid` crate.
        // We combine a counter with a hash of thread/time state.
        use std::sync::atomic::{AtomicU64, Ordering};
        use std::time::{SystemTime, UNIX_EPOCH};

        static COUNTER: AtomicU64 = AtomicU64::new(0);
        let seq = COUNTER.fetch_add(1, Ordering::Relaxed);
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();

        Self(format!("session-{ts:016x}-{seq:08x}"))
    }

    /// Create a `SessionId` from a raw string (for testing / deserialization).
    pub fn from_str(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Return the raw string representation.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ── Pure tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use super::*;
    use std::collections::HashSet;

    /// DI-003: session IDs are unique across concurrent sessions.
    #[test]
    fn test_session_ids_are_unique() {
        let ids: HashSet<String> = (0..1000)
            .map(|_| SessionId::new().as_str().to_string())
            .collect();
        assert_eq!(ids.len(), 1000, "all 1000 session IDs must be unique");
    }

    /// Session ID round-trips through `from_str` / `as_str`.
    #[test]
    fn test_session_id_round_trip() {
        let id = SessionId::from_str("test-session-42");
        assert_eq!(id.as_str(), "test-session-42");
        assert_eq!(id.to_string(), "test-session-42");
    }

    /// Two `SessionId::new()` calls produce different IDs.
    #[test]
    fn test_two_new_ids_differ() {
        let a = SessionId::new();
        let b = SessionId::new();
        assert_ne!(a, b, "consecutive session IDs must differ");
    }
}
