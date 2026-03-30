//! Protocol helpers for listing and retrieving MCP prompts with pagination.
//!
//! This module is **effectful** — it performs RPC calls through an
//! `McpConnection` and manages a per-connection prompt-list cache.
//!
//! ## Cache model
//!
//! The server can send `notifications/prompts/list_changed` at any time to
//! signal that its prompt list has changed. When that notification is
//! received, the cache should be invalidated so the next call to
//! `list_prompts_all` re-fetches from the server.
//!
//! The cache lives inside `PromptCache`, a cheaply-cloneable `Arc<Mutex<…>>`
//! wrapper that the handler registers with the `ForgeClientHandler`.
//!
//! ## Usage
//!
//! ```ignore
//! let cache = PromptCache::new();
//! // … connect, register cache with handler …
//! let prompts = list_prompts_all(&conn, &cache).await?;
//! ```

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rmcp::model::{GetPromptRequestParams, GetPromptResult, Prompt};

use crate::connection::McpConnection;
use crate::error::{CoreError, Result};

// ── PromptCache ───────────────────────────────────────────────────────────────

/// A cached copy of the server's prompt list.
///
/// Wrapped in `Arc<Mutex<…>>` so it can be shared between the protocol
/// helpers and the client handler that processes `list_changed` notifications.
#[derive(Debug, Default, Clone)]
pub struct PromptCache {
    inner: Arc<Mutex<PromptCacheInner>>,
}

#[derive(Debug, Default)]
struct PromptCacheInner {
    /// `Some(prompts)` when the cache is populated; `None` when invalid.
    prompts: Option<Vec<Prompt>>,
}

impl PromptCache {
    /// Create a new, empty (invalid) cache.
    pub fn new() -> Self {
        Self::default()
    }

    /// Invalidate the cache.
    ///
    /// Called when `notifications/prompts/list_changed` is received.
    /// The next call to `list_prompts_all` will re-fetch from the server.
    pub fn invalidate(&self) {
        let mut inner = self.inner.lock().expect("PromptCache lock poisoned");
        inner.prompts = None;
    }

    /// Read the cached prompt list, if available.
    pub fn get(&self) -> Option<Vec<Prompt>> {
        let inner = self.inner.lock().expect("PromptCache lock poisoned");
        inner.prompts.clone()
    }

    /// Populate the cache with a fresh prompt list.
    pub fn set(&self, prompts: Vec<Prompt>) {
        let mut inner = self.inner.lock().expect("PromptCache lock poisoned");
        inner.prompts = Some(prompts);
    }

    /// Returns `true` if the cache currently holds a valid (non-invalidated) list.
    pub fn is_valid(&self) -> bool {
        let inner = self.inner.lock().expect("PromptCache lock poisoned");
        inner.prompts.is_some()
    }
}

// ── list_prompts_all ──────────────────────────────────────────────────────────

/// List all prompts from the connected server, iterating through all pages.
///
/// Checks the capability guard first (returns `Err(E-PRO-003)` if the server
/// did not advertise `prompts`). Uses the cursor-based pagination pattern to
/// collect all prompts across multiple pages.
///
/// If a `cache` is provided and is valid, the cached result is returned
/// without making any RPC calls.
///
/// # Errors
/// - `Err(E-PRO-003)` if the server lacks the `prompts` capability.
/// - `Err(E-PRO-001)` on transport / protocol errors from rmcp.
pub async fn list_prompts_all<H>(
    conn: &McpConnection<H>,
    cache: Option<&PromptCache>,
) -> Result<Vec<Prompt>>
where
    H: rmcp::ClientHandler,
{
    // Capability guard — pure check, no I/O.
    if !conn.supports_prompts() {
        return Err(CoreError::CapabilityNotSupported {
            method: "prompts/list".to_string(),
            capability: "prompts".to_string(),
        });
    }

    // Return cached result if available.
    if let Some(cache) = cache {
        if let Some(cached) = cache.get() {
            return Ok(cached);
        }
    }

    // Paginate: collect all prompts across multiple pages.
    let mut all_prompts: Vec<Prompt> = Vec::new();
    let mut cursor: Option<String> = None;
    const MAX_PAGES: usize = 100;

    for _ in 0..MAX_PAGES {
        let params = cursor.map(|c| {
            rmcp::model::PaginatedRequestParams::default().with_cursor(Some(c))
        });

        let result = conn
            .peer()
            .ok_or_else(|| CoreError::Protocol("no peer available".to_string()))?
            .list_prompts(params)
            .await
            .map_err(|e| CoreError::Protocol(e.to_string()))?;

        all_prompts.extend(result.prompts);

        cursor = result.next_cursor;
        if cursor.is_none() {
            break;
        }
    }

    // Populate cache if provided.
    if let Some(cache) = cache {
        cache.set(all_prompts.clone());
    }

    Ok(all_prompts)
}

// ── get_prompt ────────────────────────────────────────────────────────────────

/// Retrieve a prompt by name, optionally passing arguments.
///
/// Checks the capability guard first (returns `Err(E-PRO-003)` if the server
/// did not advertise `prompts`). Arguments are passed to the server as-is.
///
/// # Errors
/// - `Err(E-PRO-003)` if the server lacks the `prompts` capability.
/// - `Err(E-PRO-001)` on transport / protocol errors (e.g., unknown prompt).
pub async fn get_prompt<H>(
    conn: &McpConnection<H>,
    name: impl Into<String>,
    arguments: Option<HashMap<String, serde_json::Value>>,
) -> Result<GetPromptResult>
where
    H: rmcp::ClientHandler,
{
    // Capability guard — pure check, no I/O.
    if !conn.supports_prompts() {
        return Err(CoreError::CapabilityNotSupported {
            method: "prompts/get".to_string(),
            capability: "prompts".to_string(),
        });
    }

    let params = match arguments {
        Some(args) => {
            let json_object: serde_json::Map<String, serde_json::Value> = args.into_iter().collect();
            GetPromptRequestParams::new(name).with_arguments(json_object)
        }
        None => GetPromptRequestParams::new(name),
    };

    conn.peer()
        .ok_or_else(|| CoreError::Protocol("no peer available".to_string()))?
        .get_prompt(params)
        .await
        .map_err(|e| CoreError::Protocol(e.to_string()))
}

// ── Pure unit tests ────────────────────────────────────────────────────────────

#[cfg(test)]
mod cache_tests {
    #![allow(non_snake_case)]
    use super::*;
    use rmcp::model::Prompt;

    #[test]
    fn test_prompt_cache_starts_invalid() {
        let cache = PromptCache::new();
        assert!(!cache.is_valid(), "cache should start invalid");
        assert!(cache.get().is_none(), "cache.get() should return None when invalid");
    }

    #[test]
    fn test_prompt_cache_set_and_get() {
        let cache = PromptCache::new();
        let prompts = vec![
            Prompt::new("prompt_a", Some("Prompt A"), None),
            Prompt::new("prompt_b", Some("Prompt B"), None),
        ];
        cache.set(prompts.clone());
        assert!(cache.is_valid(), "cache should be valid after set()");
        let retrieved = cache.get().expect("get() should return Some after set()");
        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].name, "prompt_a");
        assert_eq!(retrieved[1].name, "prompt_b");
    }

    /// AC-004: list_changed notification must invalidate the cache.
    #[test]
    fn test_BC_2_05_003_list_changed_invalidates_cache() {
        let cache = PromptCache::new();

        // Populate cache.
        cache.set(vec![Prompt::new("my_prompt", Some("My prompt"), None)]);
        assert!(cache.is_valid(), "cache should be valid before invalidation");

        // Simulate list_changed notification.
        cache.invalidate();

        // Cache must now be invalid.
        assert!(!cache.is_valid(), "cache should be invalid after invalidate()");
        assert!(cache.get().is_none(), "cache.get() should return None after invalidate()");
    }

    #[test]
    fn test_prompt_cache_clone_shares_state() {
        let cache1 = PromptCache::new();
        let cache2 = cache1.clone();

        // Populate via cache1.
        cache1.set(vec![Prompt::new("shared_prompt", Some("Shared"), None)]);

        // cache2 (clone) must see the same state.
        assert!(cache2.is_valid(), "cloned cache should share state");
        let prompts = cache2.get().unwrap();
        assert_eq!(prompts[0].name, "shared_prompt");

        // Invalidate via cache2.
        cache2.invalidate();

        // cache1 must also see the invalidation.
        assert!(!cache1.is_valid(), "original cache should see invalidation from clone");
    }
}
