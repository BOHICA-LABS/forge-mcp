//! Integration tests for STORY-013 & STORY-014: Bidirectional Capability Negotiation
//! and Client Capability Advertisement.
//!
//! Tests verify that `McpConnection` correctly exposes the server's negotiated
//! capabilities after the MCP `initialize` / `initialized` handshake completes,
//! and that client capabilities are advertised based on `ClientCapabilityConfig`.
//!
//! STORY-013 AC coverage:
//!   AC-001 — connect and verify capabilities match server advertisement
//!   AC-002 — server with no tools → `supports_tools()` returns false
//!   AC-003 — server with all capabilities → all checks return true
//!   AC-004 — protocol version matches after handshake
//!   AC-005 — `list_tools()` returns E-PRO-003 when server has no tools capability
//!   AC-006 — `server_capabilities()` exposes the raw rmcp `ServerCapabilities`
//!
//! STORY-014 AC coverage:
//!   AC-001 — sampling capability advertised when enabled
//!   AC-002 — elicitation capability advertised when enabled
//!   AC-003 — roots capability advertised when enabled
//!   AC-004 — sampling NOT advertised when disabled (no LLM proxy configured)

// BC-tracing test names intentionally use uppercase (BC-N-MM-NNN format).
#![allow(non_snake_case)]

use std::collections::HashMap;

use forge_core::{CoreError, connect_stdio};

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
    assert!(
        conn.supports_resources(),
        "resources capability should be present"
    );
    assert!(
        conn.supports_prompts(),
        "prompts capability should be present"
    );
    assert!(
        conn.supports_logging(),
        "logging capability should be present"
    );

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
    assert!(
        !conn.supports_resources(),
        "resources should not be supported"
    );
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

    let resources = conn
        .list_resources()
        .await
        .expect("list_resources should succeed");
    assert_eq!(resources.resources.len(), 1, "expected 1 resource");

    let prompts = conn
        .list_prompts()
        .await
        .expect("list_prompts should succeed");
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

// ─────────────────────────────────────────────────────────────────────────────
// STORY-014 tests: Configurable client capability advertisement
// ─────────────────────────────────────────────────────────────────────────────

use forge_core::{ClientCapabilityConfig, connect_stdio_with_config};

// ── AC-001: sampling capability advertised when enabled ───────────────────────

/// AC-001: When `ClientCapabilityConfig::enable_sampling = true`, the `initialize`
/// request advertises `sampling: {}` and `supports_sampling()` returns true.
#[tokio::test]
async fn test_BC_2_04_002_sampling_capability_advertised() {
    let bin = test_server_bin();
    let env = HashMap::new();

    let config = ClientCapabilityConfig {
        enable_sampling: true,
        enable_elicitation: false,
        enable_roots: false,
        root_paths: vec![],
    };

    let conn = connect_stdio_with_config(&bin, &[], &env, config)
        .await
        .expect("connect should succeed");

    assert!(
        conn.supports_sampling(),
        "sampling capability should be advertised when enable_sampling=true"
    );

    conn.shutdown().await.ok();
}

// ── AC-002: elicitation capability advertised when enabled ────────────────────

/// AC-002: When `ClientCapabilityConfig::enable_elicitation = true`, the `initialize`
/// request advertises `elicitation: {}` and `supports_elicitation()` returns true.
#[tokio::test]
async fn test_BC_2_04_002_elicitation_capability_advertised() {
    let bin = test_server_bin();
    let env = HashMap::new();

    let config = ClientCapabilityConfig {
        enable_sampling: false,
        enable_elicitation: true,
        enable_roots: false,
        root_paths: vec![],
    };

    let conn = connect_stdio_with_config(&bin, &[], &env, config)
        .await
        .expect("connect should succeed");

    assert!(
        conn.supports_elicitation(),
        "elicitation capability should be advertised when enable_elicitation=true"
    );

    conn.shutdown().await.ok();
}

// ── AC-003: roots capability advertised when enabled ──────────────────────────

/// AC-003: When `ClientCapabilityConfig::enable_roots = true`, the `initialize`
/// request advertises `roots: { listChanged: true }` and `supports_roots()` returns true.
#[tokio::test]
async fn test_BC_2_04_002_roots_capability_advertised() {
    let bin = test_server_bin();
    let env = HashMap::new();

    let root_path = std::env::temp_dir();
    let config = ClientCapabilityConfig {
        enable_sampling: false,
        enable_elicitation: false,
        enable_roots: true,
        root_paths: vec![root_path.clone()],
    };

    let conn = connect_stdio_with_config(&bin, &[], &env, config)
        .await
        .expect("connect should succeed");

    assert!(
        conn.supports_roots(),
        "roots capability should be advertised when enable_roots=true"
    );

    conn.shutdown().await.ok();
}

// ── AC-004: sampling NOT advertised without LLM proxy config ─────────────────

/// AC-004: When `ClientCapabilityConfig::enable_sampling = false`, the `initialize`
/// request does NOT include `sampling`, and `supports_sampling()` returns false.
/// This prevents servers from sending unhandleable sampling requests.
#[tokio::test]
async fn test_BC_2_04_002_sampling_not_advertised_without_config() {
    let bin = test_server_bin();
    let env = HashMap::new();

    let config = ClientCapabilityConfig {
        enable_sampling: false,
        enable_elicitation: false,
        enable_roots: false,
        root_paths: vec![],
    };

    let conn = connect_stdio_with_config(&bin, &[], &env, config)
        .await
        .expect("connect should succeed");

    assert!(
        !conn.supports_sampling(),
        "sampling should NOT be advertised when enable_sampling=false"
    );
    assert!(
        !conn.supports_elicitation(),
        "elicitation should NOT be advertised when enable_elicitation=false"
    );
    assert!(
        !conn.supports_roots(),
        "roots should NOT be advertised when enable_roots=false"
    );

    conn.shutdown().await.ok();
}
