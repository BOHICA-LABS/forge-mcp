//! Integration tests for STORY-016: Tool List & Invocation with Pagination
//!
//! Covers all 6 acceptance criteria using the mock MCP test server.
//!
//! AC-001 — paginated tool listing (multi-page)
//! AC-002 — pagination cursor loop detected and stops ≤ 100 pages
//! AC-003 — tool invocation success
//! AC-004 — tool error (isError:true) vs protocol error distinction
//! AC-005 — schema validation rejects invalid args
//! AC-006 — list_changed notification handling

#![allow(non_snake_case)]

use std::collections::HashMap;
use std::io::Write;

use forge_core::{
    CoreError, ToolListInvalidator, ToolResult, call_tool, connect_stdio, list_tools,
};

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
        .parent() // crates/
        .and_then(|p| p.parent()) // workspace root
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

/// Write a mock-server JSON config to a temp file and return the `NamedTempFile`.
///
/// The caller MUST keep the returned `NamedTempFile` alive for the test duration.
fn write_server_config(json: &str) -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().expect("tempfile");
    f.write_all(json.as_bytes()).expect("write config");
    f
}

// ── AC-001: Paginated tool listing ────────────────────────────────────────────

/// AC-001: `list_tools()` returns all tools across multiple pages.
///
/// The mock server is configured with 5 tools and page_size=2, so three
/// pages are needed (2 + 2 + 1).  All 5 tools must appear in the result.
#[tokio::test]
async fn test_BC_2_05_001_list_tools_paginated() {
    let bin = test_server_bin();
    let config_json = r#"{
        "tools": [
            { "name": "tool_0", "description": "Tool 0", "extra_params": [] },
            { "name": "tool_1", "description": "Tool 1", "extra_params": [] },
            { "name": "tool_2", "description": "Tool 2", "extra_params": [] },
            { "name": "tool_3", "description": "Tool 3", "extra_params": [] },
            { "name": "tool_4", "description": "Tool 4", "extra_params": [] }
        ],
        "resources": [],
        "prompts": [],
        "capabilities": {
            "tools": true,
            "resources": false,
            "prompts": false,
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

    let conn = connect_stdio(&bin, &args, &HashMap::new())
        .await
        .expect("connect");

    let tools = list_tools(&conn).await.expect("list_tools should succeed");

    // All 5 tools must be returned despite pagination.
    assert_eq!(tools.len(), 5, "expected 5 tools across all pages");

    // Each tool name must appear exactly once.
    for i in 0..5 {
        let expected = format!("tool_{i}");
        assert!(
            tools.iter().any(|t| t.name == expected.as_str()),
            "tool_{i} missing from result"
        );
    }

    conn.shutdown().await.ok();
}

/// AC-001 (edge case EC-001): Server with 0 tools returns empty Vec.
#[tokio::test]
async fn test_BC_2_05_001_list_tools_empty_server() {
    let bin = test_server_bin();
    let config_json = r#"{
        "tools": [],
        "resources": [],
        "prompts": [],
        "capabilities": {
            "tools": true,
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

    let conn = connect_stdio(&bin, &args, &HashMap::new())
        .await
        .expect("connect");

    let tools = list_tools(&conn)
        .await
        .expect("list_tools empty should succeed");
    assert_eq!(
        tools.len(),
        0,
        "expected empty Vec for server with no tools"
    );

    conn.shutdown().await.ok();
}

// ── AC-002: Pagination cursor loop detection ──────────────────────────────────

/// AC-002: When the server returns a looping cursor, `list_tools` stops after
/// detecting the loop and returns `Err(E-PRO-004)`.
///
/// The mock server's `loop_cursor: true` mode always returns the same cursor,
/// simulating an infinite-loop server.
#[tokio::test]
async fn test_BC_2_05_001_pagination_cursor_loop_detected() {
    let bin = test_server_bin();
    let config_json = r#"{
        "tools": [
            { "name": "loop_tool", "description": "Loop", "extra_params": [] }
        ],
        "resources": [],
        "prompts": [],
        "capabilities": {
            "tools": true,
            "resources": false,
            "prompts": false,
            "logging": false,
            "completions": false
        },
        "pagination": { "page_size": 1, "loop_cursor": true },
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

    let conn = connect_stdio(&bin, &args, &HashMap::new())
        .await
        .expect("connect");

    let result = list_tools(&conn).await;

    match result {
        Err(CoreError::PaginationCursorLoop { pages }) => {
            // Loop must be detected quickly (cursor dedup on second identical cursor).
            assert!(
                pages <= forge_core::MAX_PAGES,
                "loop should be detected by page {pages} (≤ MAX_PAGES)"
            );
        }
        other => panic!("expected PaginationCursorLoop error, got: {other:?}"),
    }

    conn.shutdown().await.ok();
}

// ── AC-003: Tool invocation success ──────────────────────────────────────────

/// AC-003: `call_tool` with valid arguments returns `Ok(ToolResult { is_error: false })`.
#[tokio::test]
async fn test_BC_2_05_001_tool_invocation_success() {
    let bin = test_server_bin();
    let config_json = r#"{
        "tools": [
            { "name": "echo_tool", "description": "Echoes input", "extra_params": [] }
        ],
        "resources": [],
        "prompts": [],
        "capabilities": {
            "tools": true,
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

    let conn = connect_stdio(&bin, &args, &HashMap::new())
        .await
        .expect("connect");

    let result = call_tool(
        &conn,
        "echo_tool",
        serde_json::json!({ "input": "hello world" }),
        None, // no schema validation for this test
    )
    .await
    .expect("call_tool should succeed");

    assert!(!result.is_error, "successful call must have is_error=false");
    assert!(!result.content.is_empty(), "result must have content");

    conn.shutdown().await.ok();
}

// ── AC-004: Tool error vs protocol error distinction ─────────────────────────

/// AC-004: When the server returns `isError: true`, `call_tool` returns
/// `Ok(ToolResult { is_error: true })` — NOT an `Err`.
///
/// The mock server's `error_tool_result: true` flag causes it to return
/// `CallToolResult::error(...)` for every tool invocation.
#[tokio::test]
async fn test_BC_2_05_001_tool_error_vs_protocol_error() {
    let bin = test_server_bin();
    let config_json = r#"{
        "tools": [
            { "name": "failing_tool", "description": "Always fails", "extra_params": [] }
        ],
        "resources": [],
        "prompts": [],
        "capabilities": {
            "tools": true,
            "resources": false,
            "prompts": false,
            "logging": false,
            "completions": false
        },
        "pagination": { "page_size": 0, "loop_cursor": false },
        "error_injection": {
            "delay_ms": 0,
            "crash_on_tool_call": false,
            "error_tool_result": true,
            "emit_malformed": false
        }
    }"#;
    let config_file = write_server_config(config_json);
    let args = vec![
        "--config".to_string(),
        config_file.path().to_string_lossy().to_string(),
    ];

    let conn = connect_stdio(&bin, &args, &HashMap::new())
        .await
        .expect("connect");

    // call_tool MUST return Ok even when the tool reports an error.
    let result = call_tool(
        &conn,
        "failing_tool",
        serde_json::json!({ "input": "test" }),
        None,
    )
    .await;

    match result {
        Ok(ToolResult {
            is_error: true,
            content,
        }) => {
            // Correct: tool error, not a protocol error.
            assert!(
                !content.is_empty(),
                "error result should have content explaining the error"
            );
        }
        Ok(ToolResult {
            is_error: false, ..
        }) => {
            panic!("expected is_error=true but got is_error=false");
        }
        Err(e) => {
            panic!("expected Ok(ToolResult {{ is_error: true }}), got Err: {e:?}");
        }
    }

    conn.shutdown().await.ok();
}

/// AC-004 (protocol error): Calling a non-existent tool returns `Err` (protocol-level failure).
#[tokio::test]
async fn test_BC_2_05_001_protocol_error_on_unknown_tool() {
    let bin = test_server_bin();
    let config_json = r#"{
        "tools": [
            { "name": "real_tool", "description": "Exists", "extra_params": [] }
        ],
        "resources": [],
        "prompts": [],
        "capabilities": {
            "tools": true,
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

    let conn = connect_stdio(&bin, &args, &HashMap::new())
        .await
        .expect("connect");

    // The mock server returns a JSON-RPC error for unknown tools.
    let result = call_tool(&conn, "nonexistent_tool", serde_json::json!({}), None).await;

    assert!(
        result.is_err(),
        "calling a non-existent tool should return a protocol Err, got: {result:?}"
    );

    conn.shutdown().await.ok();
}

// ── AC-005: Schema validation ─────────────────────────────────────────────────

/// AC-005: `call_tool` with a tool_def rejects arguments that violate the schema.
///
/// We construct a `Tool` with a required `input` string field and pass an
/// empty object — this must be rejected before the request is sent.
#[tokio::test]
async fn test_BC_2_05_001_argument_schema_validation() {
    let bin = test_server_bin();
    let config_json = r#"{
        "tools": [
            { "name": "strict_tool", "description": "Needs input", "extra_params": [] }
        ],
        "resources": [],
        "prompts": [],
        "capabilities": {
            "tools": true,
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

    let conn = connect_stdio(&bin, &args, &HashMap::new())
        .await
        .expect("connect");

    // Build a tool definition with a required `input` field.
    let schema: std::sync::Arc<serde_json::Map<String, serde_json::Value>> = std::sync::Arc::new(
        serde_json::from_value(serde_json::json!({
            "type": "object",
            "properties": {
                "input": { "type": "string" }
            },
            "required": ["input"]
        }))
        .expect("valid schema"),
    );

    let tool_def = rmcp::model::Tool::new("strict_tool", "Needs input", schema);

    // Missing required field — must return E-PRO-005 without hitting the server.
    let result = call_tool(
        &conn,
        "strict_tool",
        serde_json::json!({}), // missing "input"
        Some(&tool_def),
    )
    .await;

    match result {
        Err(CoreError::SchemaValidationFailed { tool, reason }) => {
            assert_eq!(tool, "strict_tool");
            assert!(
                !reason.is_empty(),
                "reason should explain the validation failure"
            );
        }
        other => panic!("expected SchemaValidationFailed, got: {other:?}"),
    }

    // Correct arguments — must succeed.
    let ok_result = call_tool(
        &conn,
        "strict_tool",
        serde_json::json!({ "input": "valid value" }),
        Some(&tool_def),
    )
    .await
    .expect("valid args should succeed");

    assert!(!ok_result.is_error);

    conn.shutdown().await.ok();
}

// ── AC-006: list_changed notification ────────────────────────────────────────

/// AC-006: `ToolListInvalidator::invalidate()` signals the watcher and the
/// generation counter increments.
///
/// Note: the MCP `notifications/tools/list_changed` notification fires from a
/// running server, but wiring it into rmcp's client handler requires a custom
/// `ClientHandler` implementation.  This test validates the observable contract
/// of the watcher/invalidator pair directly (unit-style integration test),
/// confirming that any code which calls `invalidate()` in response to the
/// notification will work correctly.
#[tokio::test]
async fn test_BC_2_05_001_list_changed_notification() {
    let (inv, mut watcher) = ToolListInvalidator::new();

    // Initially — no invalidation.
    assert!(!watcher.is_stale());
    assert_eq!(watcher.generation(), 0);
    assert_eq!(inv.generation(), 0);

    // Simulate server sending notifications/tools/list_changed → invalidate.
    inv.invalidate();

    // Watcher detects stale state.
    assert!(watcher.is_stale(), "watcher must detect invalidation");
    assert_eq!(watcher.generation(), 1);

    // Mark fresh (e.g., after re-fetching tool list).
    watcher.mark_fresh();
    assert!(!watcher.is_stale());

    // Second invalidation.
    inv.invalidate();
    assert!(watcher.is_stale());
    assert_eq!(watcher.generation(), 2);

    // Multiple watcher clones share the same signal.
    let mut watcher2 = watcher.clone();
    assert!(watcher2.is_stale());
    inv.invalidate();
    assert_eq!(watcher2.generation(), 3);
}

// ── AC-003 + capability guard ─────────────────────────────────────────────────

/// Capability guard: `list_tools` returns `E-PRO-003` when server has no
/// tools capability.
#[tokio::test]
async fn test_BC_2_05_001_list_tools_requires_tools_capability() {
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

    let conn = connect_stdio(&bin, &args, &HashMap::new())
        .await
        .expect("connect");

    let result = list_tools(&conn).await;
    match result {
        Err(CoreError::CapabilityNotSupported { method, capability }) => {
            assert_eq!(method, "tools/list");
            assert_eq!(capability, "tools");
        }
        other => panic!("expected CapabilityNotSupported, got: {other:?}"),
    }

    conn.shutdown().await.ok();
}

/// Capability guard: `call_tool` returns `E-PRO-003` when server has no
/// tools capability.
#[tokio::test]
async fn test_BC_2_05_001_call_tool_requires_tools_capability() {
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

    let conn = connect_stdio(&bin, &args, &HashMap::new())
        .await
        .expect("connect");

    let result = call_tool(&conn, "any_tool", serde_json::json!({}), None).await;
    match result {
        Err(CoreError::CapabilityNotSupported { method, capability }) => {
            assert_eq!(method, "tools/call");
            assert_eq!(capability, "tools");
        }
        other => panic!("expected CapabilityNotSupported, got: {other:?}"),
    }

    conn.shutdown().await.ok();
}
