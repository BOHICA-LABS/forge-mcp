//! Integration tests for STORY-013: Bidirectional Capability Negotiation.
//!
//! Tests verify that `McpConnection` correctly exposes the server's negotiated
//! capabilities after the MCP `initialize` / `initialized` handshake completes.
//!
//! AC coverage:
//!   AC-001 — connect and verify capabilities match server advertisement
//!   AC-002 — server with no tools → `supports_tools()` returns false
//!   AC-003 — server with all capabilities → all checks return true
//!   AC-004 — protocol version matches after handshake
//!   AC-005 — `list_tools()` returns E-PRO-003 when server has no tools capability
//!   AC-006 — `server_capabilities()` exposes the raw rmcp `ServerCapabilities`

// BC-tracing test names intentionally use uppercase (BC-N-MM-NNN format).
#![allow(non_snake_case)]

use std::collections::HashMap;

use forge_core::{connect_stdio, CoreError};

// ── Helper ────────────────────────────────────────────────────────────────────

/// Resolve the path to the `forge-test-server` binary.
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
    workspace_root
        .join("target")
        .join("debug")
        .join("forge-test-server")
        .to_string_lossy()
        .to_string()
}

/// Write a mock-server JSON config to a temp file and return the path.
///
/// The caller is responsible for keeping the `tempfile::NamedTempFile` alive
/// for the duration of the test — when it drops, the file is deleted.
fn write_server_config(json: &str) -> tempfile::NamedTempFile {
    use std::io::Write;
    let mut f = tempfile::NamedTempFile::new().expect("tempfile");
    f.write_all(json.as_bytes()).expect("write config");
    f
}

// ── AC-001: capabilities match server advertisement ────────────────────────────

/// AC-001: After `connect_stdio`, the `supports_*` methods agree with what the
/// mock server configured itself to advertise.
///
/// forge-test-server's default (`from_counts`) enables ALL capabilities.
#[tokio::test]
async fn test_BC_2_04_001_capabilities_match_server_advertisement() {
    let bin = test_server_bin();
    let args = vec!["--tools".to_string(), "3".to_string()];
    let env = HashMap::new();

    let conn = connect_stdio(&bin, &args, &env)
        .await
        .expect("connect should succeed");

    // The mock server's `from_counts` enables all capabilities.
    assert!(conn.supports_tools(), "tools capability should be present");
    assert!(conn.supports_resources(), "resources capability should be present");
    assert!(conn.supports_prompts(), "prompts capability should be present");
    assert!(conn.supports_logging(), "logging capability should be present");

    conn.shutdown().await.ok();
}

// ── AC-002: server with no tools capability ────────────────────────────────────

/// AC-002: When the server doesn't advertise `tools`, `supports_tools()` returns
/// false and `list_tools()` returns `Err(E-PRO-003)` without hitting the network.
#[tokio::test]
async fn test_BC_2_04_002_no_tools_capability_returns_false() {
    let bin = test_server_bin();
    // Use JSON config to disable the tools capability explicitly.
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
        .expect("connect should succeed even with empty capabilities");

    // Server did not advertise tools.
    assert!(!conn.supports_tools(), "tools should not be supported");
    assert!(!conn.supports_resources(), "resources should not be supported");
    assert!(!conn.supports_prompts(), "prompts should not be supported");
    assert!(!conn.supports_logging(), "logging should not be supported");

    // list_tools() must fail with E-PRO-003 (no I/O attempted).
    let result = conn.list_tools().await;
    match result {
        Err(CoreError::CapabilityNotSupported { method, capability }) => {
            assert_eq!(method, "tools/list");
            assert_eq!(capability, "tools");
        }
        other => panic!("expected CapabilityNotSupported, got: {other:?}"),
    }

    conn.shutdown().await.ok();
}

// ── AC-003: server with all capabilities ─────────────────────────────────────

/// AC-003: When the server advertises all capabilities, every `supports_*`
/// method returns true and list operations succeed.
#[tokio::test]
async fn test_BC_2_04_003_all_capabilities_return_true() {
    let bin = test_server_bin();
    let config_json = r#"{
        "tools": [
            { "name": "tool_alpha", "description": "Alpha", "extra_params": [] },
            { "name": "tool_beta",  "description": "Beta",  "extra_params": [] }
        ],
        "resources": [
            { "uri": "resource://test/0", "name": "R0", "description": "Resource 0", "content": "data0" }
        ],
        "prompts": [
            { "name": "prompt_x", "description": "Prompt X" }
        ],
        "capabilities": {
            "tools": true,
            "resources": true,
            "prompts": true,
            "logging": true,
            "completions": true
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

    // All capabilities must be present.
    assert!(conn.supports_tools(), "tools must be true");
    assert!(conn.supports_resources(), "resources must be true");
    assert!(conn.supports_prompts(), "prompts must be true");
    assert!(conn.supports_logging(), "logging must be true");

    // Actual list calls must succeed.
    let tools = conn.list_tools().await.expect("list_tools should succeed");
    assert_eq!(tools.tools.len(), 2, "expected 2 tools");

    let resources = conn.list_resources().await.expect("list_resources should succeed");
    assert_eq!(resources.resources.len(), 1, "expected 1 resource");

    let prompts = conn.list_prompts().await.expect("list_prompts should succeed");
    assert_eq!(prompts.prompts.len(), 1, "expected 1 prompt");

    conn.shutdown().await.ok();
}

// ── AC-004: protocol version ──────────────────────────────────────────────────

/// AC-004: `protocol_version()` returns a non-empty string after connect.
///
/// rmcp sets the protocol version from the server's `InitializeResult`. We
/// verify it is a valid MCP version string (non-empty, contains a date-like
/// pattern like `"2025-06-18"`).
#[tokio::test]
async fn test_BC_2_04_004_protocol_version_is_set() {
    let bin = test_server_bin();
    let env = HashMap::new();

    let conn = connect_stdio(&bin, &[], &env)
        .await
        .expect("connect should succeed");

    let version = conn.protocol_version();
    assert!(!version.is_empty(), "protocol_version must not be empty");
    // MCP versions are date strings in YYYY-MM-DD format.
    assert!(
        version.contains('-'),
        "protocol version should contain '-' (e.g., '2025-06-18'), got: {version}"
    );

    conn.shutdown().await.ok();
}

// ── AC-005: list_tools() guard without tools capability ───────────────────────

/// AC-005: `list_resources()` and `list_prompts()` also guard properly.
/// When the server has tools but not resources/prompts, the corresponding
/// calls must return E-PRO-003.
#[tokio::test]
async fn test_BC_2_04_005_method_guards_for_missing_capabilities() {
    let bin = test_server_bin();
    // Advertise only `tools`, nothing else.
    let config_json = r#"{
        "tools": [{ "name": "only_tool", "description": "Only tool", "extra_params": [] }],
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
    let env = HashMap::new();

    let conn = connect_stdio(&bin, &args, &env)
        .await
        .expect("connect should succeed");

    // tools: present → list_tools works.
    assert!(conn.supports_tools());
    let tools = conn.list_tools().await.expect("list_tools should succeed");
    assert_eq!(tools.tools.len(), 1);

    // resources: absent → E-PRO-003.
    assert!(!conn.supports_resources());
    match conn.list_resources().await {
        Err(CoreError::CapabilityNotSupported { capability, .. }) => {
            assert_eq!(capability, "resources");
        }
        other => panic!("expected CapabilityNotSupported for resources, got: {other:?}"),
    }

    // prompts: absent → E-PRO-003.
    assert!(!conn.supports_prompts());
    match conn.list_prompts().await {
        Err(CoreError::CapabilityNotSupported { capability, .. }) => {
            assert_eq!(capability, "prompts");
        }
        other => panic!("expected CapabilityNotSupported for prompts, got: {other:?}"),
    }

    conn.shutdown().await.ok();
}

// ── AC-006: server_capabilities() exposes raw ServerCapabilities ──────────────

/// AC-006: `server_capabilities()` returns the raw `rmcp::ServerCapabilities`
/// so downstream code can inspect sub-fields (e.g., `list_changed`, `subscribe`).
#[tokio::test]
async fn test_BC_2_04_006_server_capabilities_raw_access() {
    let bin = test_server_bin();
    let args = vec!["--tools".to_string(), "2".to_string()];
    let env = HashMap::new();

    let conn = connect_stdio(&bin, &args, &env)
        .await
        .expect("connect should succeed");

    let raw = conn.server_capabilities();

    // The raw ServerCapabilities should expose the same tools presence as supports_tools().
    assert_eq!(
        raw.tools.is_some(),
        conn.supports_tools(),
        "server_capabilities().tools.is_some() must match supports_tools()"
    );
    assert_eq!(
        raw.resources.is_some(),
        conn.supports_resources(),
        "server_capabilities().resources.is_some() must match supports_resources()"
    );
    assert_eq!(
        raw.prompts.is_some(),
        conn.supports_prompts(),
        "server_capabilities().prompts.is_some() must match supports_prompts()"
    );
    assert_eq!(
        raw.logging.is_some(),
        conn.supports_logging(),
        "server_capabilities().logging.is_some() must match supports_logging()"
    );

    conn.shutdown().await.ok();
}
