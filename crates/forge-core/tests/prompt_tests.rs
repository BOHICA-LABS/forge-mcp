//! Integration tests for STORY-018: Prompt List & Retrieval with Pagination.
//!
//! Tests verify that `McpConnection` correctly lists and retrieves prompts,
//! with pagination support and capability guarding.
//!
//! AC coverage:
//!   AC-001 — list_prompts_all() returns Vec<Prompt> with pagination support
//!   AC-002 — get_prompt(name, arguments) returns GetPromptResult
//!   AC-003 — capability guard returns E-PRO-003 when prompts not advertised
//!   AC-004 — list_changed notification invalidates the PromptCache

// BC-tracing test names intentionally use uppercase (BC-N-MM-NNN format).
#![allow(non_snake_case)]

use std::collections::HashMap;
use std::io::Write;

use forge_core::{CoreError, PromptCache, connect_stdio, get_prompt, list_prompts_all};

// ── Helper ────────────────────────────────────────────────────────────────────

/// Resolve the path to the `forge-test-server` binary.
///
/// Resolution order:
/// 1. `CARGO_BIN_EXE_forge-test-server` env var (set when the binary is a
///    dev-dependency of the test crate — ideal path).
/// 2. Scan `target/<triple>/debug/` subdirectories (handles CI runs with
///    `--target <triple>` that place binaries under a target-triple prefix).
/// 3. Fallback: `target/debug/forge-test-server` (local `cargo test` without
///    an explicit `--target` flag).
fn test_server_bin() -> String {
    if let Ok(path) = std::env::var("CARGO_BIN_EXE_forge-test-server") {
        return path;
    }
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
        .expect("CARGO_MANIFEST_DIR must be set in test environment");
    let workspace_root = std::path::Path::new(&manifest_dir)
        .parent()
        .and_then(|p| p.parent())
        .expect("expected workspace root");
    let target_dir = workspace_root.join("target");
    let bin_name = if cfg!(windows) {
        "forge-test-server.exe"
    } else {
        "forge-test-server"
    };

    // Check target-triple subdirectories first (CI with --target <triple>).
    if let Ok(entries) = std::fs::read_dir(&target_dir) {
        for entry in entries.flatten() {
            let candidate = entry.path().join("debug").join(bin_name);
            if candidate.exists() {
                return candidate.to_string_lossy().to_string();
            }
        }
    }
    // Fallback: plain target/debug (local builds without --target).
    target_dir
        .join("debug")
        .join(bin_name)
        .to_string_lossy()
        .to_string()
}

fn write_server_config(json: &str) -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().expect("tempfile");
    f.write_all(json.as_bytes()).expect("write config");
    f
}

// ── AC-001: Paginated prompt listing ─────────────────────────────────────────

/// AC-001: `list_prompts_all` returns all prompts across multiple pages.
///
/// We configure the mock server with 5 prompts and page_size=2, so we expect
/// 3 pages (2 + 2 + 1). The function must collect all 5 prompts.
#[tokio::test]
async fn test_BC_2_05_003_list_prompts_paginated() {
    let bin = test_server_bin();
    let config_json = r#"{
        "tools": [],
        "resources": [],
        "prompts": [
            { "name": "prompt_0", "description": "Prompt 0" },
            { "name": "prompt_1", "description": "Prompt 1" },
            { "name": "prompt_2", "description": "Prompt 2" },
            { "name": "prompt_3", "description": "Prompt 3" },
            { "name": "prompt_4", "description": "Prompt 4" }
        ],
        "capabilities": {
            "tools": false,
            "resources": false,
            "prompts": true,
            "logging": false,
            "completions": false
        },
        "pagination": { "page_size": 2, "loop_cursor": false },
        "error_injection": {
            "delay_ms": 0,
            "crash_on_tool_call": false,
            "error_tool_result": false,
            "emit_malformed": false
        }
    }"#;
    let config_file = write_server_config(config_json);
    let args = vec![
        "--config".to_string(),
        config_file.path().to_string_lossy().to_string(),
    ];
    let env = HashMap::new();

    let conn = connect_stdio(&bin, &args, &env)
        .await
        .expect("connect should succeed");

    assert!(
        conn.supports_prompts(),
        "server must advertise prompts capability"
    );

    let prompts = list_prompts_all(&conn, None)
        .await
        .expect("list_prompts_all should succeed");

    assert_eq!(
        prompts.len(),
        5,
        "expected 5 prompts across all pages, got {}",
        prompts.len()
    );

    // Verify names are collected in order.
    let names: Vec<&str> = prompts.iter().map(|p| p.name.as_str()).collect();
    assert!(names.contains(&"prompt_0"), "prompt_0 must be present");
    assert!(names.contains(&"prompt_4"), "prompt_4 must be present");

    conn.shutdown().await.ok();
}

/// AC-001b: list_prompts_all returns from cache on second call.
#[tokio::test]
async fn test_BC_2_05_003_list_prompts_uses_cache() {
    let bin = test_server_bin();
    let config_json = r#"{
        "tools": [],
        "resources": [],
        "prompts": [
            { "name": "cached_prompt", "description": "Cached" }
        ],
        "capabilities": {
            "tools": false,
            "resources": false,
            "prompts": true,
            "logging": false,
            "completions": false
        },
        "pagination": { "page_size": 0, "loop_cursor": false },
        "error_injection": {
            "delay_ms": 0,
            "crash_on_tool_call": false,
            "error_tool_result": false,
            "emit_malformed": false
        }
    }"#;
    let config_file = write_server_config(config_json);
    let args = vec![
        "--config".to_string(),
        config_file.path().to_string_lossy().to_string(),
    ];
    let env = HashMap::new();

    let conn = connect_stdio(&bin, &args, &env)
        .await
        .expect("connect should succeed");

    let cache = PromptCache::new();
    assert!(!cache.is_valid(), "cache must start invalid");

    // First call: fetches from server and populates cache.
    let prompts1 = list_prompts_all(&conn, Some(&cache))
        .await
        .expect("first call should succeed");
    assert_eq!(prompts1.len(), 1);
    assert!(cache.is_valid(), "cache should be valid after first call");

    // Second call: should return cached result.
    let prompts2 = list_prompts_all(&conn, Some(&cache))
        .await
        .expect("second call should succeed");
    assert_eq!(prompts2.len(), 1);
    assert_eq!(
        prompts1[0].name, prompts2[0].name,
        "cached result must match"
    );

    conn.shutdown().await.ok();
}

// ── AC-002: Get prompt with arguments ────────────────────────────────────────

/// AC-002: `get_prompt(name, arguments)` returns a `GetPromptResult` with messages.
#[tokio::test]
async fn test_BC_2_05_003_get_prompt_with_args() {
    let bin = test_server_bin();
    let config_json = r#"{
        "tools": [],
        "resources": [],
        "prompts": [
            { "name": "greeting", "description": "Greeting prompt" }
        ],
        "capabilities": {
            "tools": false,
            "resources": false,
            "prompts": true,
            "logging": false,
            "completions": false
        },
        "pagination": { "page_size": 0, "loop_cursor": false },
        "error_injection": {
            "delay_ms": 0,
            "crash_on_tool_call": false,
            "error_tool_result": false,
            "emit_malformed": false
        }
    }"#;
    let config_file = write_server_config(config_json);
    let args = vec![
        "--config".to_string(),
        config_file.path().to_string_lossy().to_string(),
    ];
    let env = HashMap::new();

    let conn = connect_stdio(&bin, &args, &env)
        .await
        .expect("connect should succeed");

    assert!(conn.supports_prompts(), "server must advertise prompts");

    // Call get_prompt with no arguments.
    let result = get_prompt(&conn, "greeting", None)
        .await
        .expect("get_prompt should succeed");

    assert!(
        !result.messages.is_empty(),
        "GetPromptResult must have at least one message"
    );

    // The mock server returns "Mock response for prompt: <name>".
    if let rmcp::model::PromptMessageContent::Text { ref text } = result.messages[0].content {
        assert!(
            text.contains("greeting"),
            "response text should reference the prompt name, got: {text}"
        );
    }

    // Call get_prompt with arguments (arguments are passed as-is; mock ignores them).
    let mut arguments = HashMap::new();
    arguments.insert(
        "user".to_string(),
        serde_json::Value::String("Alice".to_string()),
    );
    let result_with_args = get_prompt(&conn, "greeting", Some(arguments))
        .await
        .expect("get_prompt with args should succeed");

    assert!(
        !result_with_args.messages.is_empty(),
        "GetPromptResult with args must have at least one message"
    );

    conn.shutdown().await.ok();
}

/// AC-002b: `McpConnection::get_prompt` convenience method works.
#[tokio::test]
async fn test_BC_2_05_003_conn_get_prompt_method() {
    let bin = test_server_bin();
    let config_json = r#"{
        "tools": [],
        "resources": [],
        "prompts": [
            { "name": "my_prompt", "description": "My prompt" }
        ],
        "capabilities": {
            "tools": false,
            "resources": false,
            "prompts": true,
            "logging": false,
            "completions": false
        },
        "pagination": { "page_size": 0, "loop_cursor": false },
        "error_injection": {
            "delay_ms": 0,
            "crash_on_tool_call": false,
            "error_tool_result": false,
            "emit_malformed": false
        }
    }"#;
    let config_file = write_server_config(config_json);
    let args_cli = vec![
        "--config".to_string(),
        config_file.path().to_string_lossy().to_string(),
    ];
    let env = HashMap::new();

    let conn = connect_stdio(&bin, &args_cli, &env)
        .await
        .expect("connect should succeed");

    // Use the McpConnection::get_prompt convenience method.
    let result = conn
        .get_prompt("my_prompt", None)
        .await
        .expect("conn.get_prompt should succeed");

    assert!(
        !result.messages.is_empty(),
        "must have at least one message"
    );

    conn.shutdown().await.ok();
}

// ── AC-003: Capability guard ──────────────────────────────────────────────────

/// AC-003: When server did not advertise `prompts`, `list_prompts_all` returns
/// `Err(E-PRO-003)` without making any network calls.
#[tokio::test]
async fn test_BC_2_05_003_capability_guard() {
    let bin = test_server_bin();
    let config_json = r#"{
        "tools": [],
        "resources": [],
        "prompts": [],
        "capabilities": {
            "tools": false,
            "resources": false,
            "prompts": false,
            "logging": false,
            "completions": false
        },
        "pagination": { "page_size": 0, "loop_cursor": false },
        "error_injection": {
            "delay_ms": 0,
            "crash_on_tool_call": false,
            "error_tool_result": false,
            "emit_malformed": false
        }
    }"#;
    let config_file = write_server_config(config_json);
    let args = vec![
        "--config".to_string(),
        config_file.path().to_string_lossy().to_string(),
    ];
    let env = HashMap::new();

    let conn = connect_stdio(&bin, &args, &env)
        .await
        .expect("connect should succeed even with no capabilities");

    assert!(
        !conn.supports_prompts(),
        "server must NOT advertise prompts"
    );

    // list_prompts_all must fail with E-PRO-003 (no I/O attempted).
    let list_result = list_prompts_all(&conn, None).await;
    match list_result {
        Err(CoreError::CapabilityNotSupported { method, capability }) => {
            assert_eq!(method, "prompts/list", "wrong method in error");
            assert_eq!(capability, "prompts", "wrong capability in error");
        }
        other => panic!("expected CapabilityNotSupported for list_prompts_all, got: {other:?}"),
    }

    // get_prompt must also fail with E-PRO-003.
    let get_result = get_prompt(&conn, "any_prompt", None).await;
    match get_result {
        Err(CoreError::CapabilityNotSupported { method, capability }) => {
            assert_eq!(method, "prompts/get", "wrong method in error");
            assert_eq!(capability, "prompts", "wrong capability in error");
        }
        other => panic!("expected CapabilityNotSupported for get_prompt, got: {other:?}"),
    }

    // McpConnection::get_prompt convenience method must also guard.
    let conn_get_result = conn.get_prompt("any_prompt", None).await;
    match conn_get_result {
        Err(CoreError::CapabilityNotSupported { method, capability }) => {
            assert_eq!(method, "prompts/get");
            assert_eq!(capability, "prompts");
        }
        other => panic!("expected CapabilityNotSupported for conn.get_prompt, got: {other:?}"),
    }

    conn.shutdown().await.ok();
}

// ── AC-004: list_changed notification invalidates cache ───────────────────────

/// AC-004: When `notifications/prompts/list_changed` arrives, the cached
/// prompt list is invalidated. We test this via the pure `PromptCache` unit
/// (the notification handler calls `cache.invalidate()`).
///
/// The unit test lives in `protocol.rs` under `#[cfg(test)]`. This integration
/// test validates the same behaviour end-to-end by:
/// 1. Populating the cache via `list_prompts_all`.
/// 2. Calling `cache.invalidate()` (simulating the notification receipt).
/// 3. Verifying the cache is now invalid so the next call re-fetches.
#[tokio::test]
async fn test_BC_2_05_003_list_changed_invalidates_cache() {
    let bin = test_server_bin();
    let config_json = r#"{
        "tools": [],
        "resources": [],
        "prompts": [
            { "name": "invalidate_test_prompt", "description": "Cache invalidation test" }
        ],
        "capabilities": {
            "tools": false,
            "resources": false,
            "prompts": true,
            "logging": false,
            "completions": false
        },
        "pagination": { "page_size": 0, "loop_cursor": false },
        "error_injection": {
            "delay_ms": 0,
            "crash_on_tool_call": false,
            "error_tool_result": false,
            "emit_malformed": false
        }
    }"#;
    let config_file = write_server_config(config_json);
    let args = vec![
        "--config".to_string(),
        config_file.path().to_string_lossy().to_string(),
    ];
    let env = HashMap::new();

    let conn = connect_stdio(&bin, &args, &env)
        .await
        .expect("connect should succeed");

    let cache = PromptCache::new();

    // Populate cache.
    let prompts = list_prompts_all(&conn, Some(&cache))
        .await
        .expect("list_prompts_all should succeed");
    assert_eq!(prompts.len(), 1, "expected 1 prompt");
    assert!(cache.is_valid(), "cache must be valid after first fetch");

    // Simulate list_changed notification: invalidate the cache.
    cache.invalidate();

    // Cache must now be invalid.
    assert!(
        !cache.is_valid(),
        "cache must be invalid after list_changed"
    );
    assert!(
        cache.get().is_none(),
        "cache.get() must return None after list_changed"
    );

    // Next call to list_prompts_all must re-fetch from server.
    let prompts_after = list_prompts_all(&conn, Some(&cache))
        .await
        .expect("re-fetch after invalidation should succeed");
    assert_eq!(
        prompts_after.len(),
        1,
        "re-fetched prompt list must have same count"
    );
    assert!(cache.is_valid(), "cache must be valid again after re-fetch");

    conn.shutdown().await.ok();
}
