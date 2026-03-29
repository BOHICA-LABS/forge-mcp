//! Integration tests for forge-test-server.
//!
//! Each test spawns the `forge-test-server` binary as a subprocess and
//! connects to it via the MCP stdio transport.

use std::env;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use rmcp::model::{
    CallToolRequestParams, GetPromptRequestParams, PaginatedRequestParams,
    ReadResourceRequestParams,
};
use rmcp::service::RoleClient;
use rmcp::transport::TokioChildProcess;
use rmcp::serve_client;
use rmcp::service::RunningService;
use tokio::process::Command;
use tokio::time::timeout;

/// Locate the forge-test-server binary built by Cargo.
fn test_server_bin() -> PathBuf {
    // CARGO_BIN_EXE_forge-test-server is set by Cargo for integration tests.
    if let Ok(p) = env::var("CARGO_BIN_EXE_forge-test-server") {
        return PathBuf::from(p);
    }
    // Fallback: look in the same directory as the current test binary.
    let exe = env::current_exe().expect("current_exe");
    let dir = exe.parent().expect("exe parent dir");
    // In deps/ layout the binary is one level up.
    let candidate = dir.join("forge-test-server");
    if candidate.exists() {
        return candidate;
    }
    dir.parent()
        .map(|p| p.join("forge-test-server"))
        .filter(|p| p.exists())
        .unwrap_or(candidate)
}

/// Spawn the server binary and return a `RunningService` (keeps connection alive).
async fn spawn_client(args: &[&str]) -> RunningService<RoleClient, ()> {
    let bin = test_server_bin();

    let mut cmd = Command::new(&bin);
    cmd.args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    let proc = TokioChildProcess::new(cmd).expect("spawn forge-test-server");

    timeout(Duration::from_secs(10), serve_client((), proc))
        .await
        .expect("timeout connecting to test server")
        .expect("serve_client failed")
}

// ── Basic protocol tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_initialize_returns_server_info() {
    let svc = spawn_client(&["--tools", "3", "--resources", "2", "--prompts", "1"]).await;
    let info = svc.peer_info().expect("server_info should be present");
    assert_eq!(info.server_info.name, "forge-test-server");
    assert!(
        info.capabilities.tools.is_some(),
        "tools capability should be present"
    );
    assert!(info.capabilities.resources.is_some(), "resources capability");
    assert!(info.capabilities.prompts.is_some(), "prompts capability");
}

#[tokio::test]
async fn test_list_tools_count() {
    let svc = spawn_client(&["--tools", "5"]).await;
    let result = svc.list_tools(None).await.expect("list_tools");
    assert_eq!(result.tools.len(), 5, "expected 5 tools");
}

#[tokio::test]
async fn test_call_tool_success() {
    let svc = spawn_client(&["--tools", "3"]).await;
    let result = svc
        .call_tool(CallToolRequestParams::new("mock_tool_0"))
        .await
        .expect("call_tool");
    assert!(
        !result.is_error.unwrap_or(false),
        "should not be an error"
    );
    let text = result
        .content
        .first()
        .and_then(|c| c.as_text())
        .map(|t| t.text.as_str())
        .unwrap_or("");
    assert!(
        text.contains("mock_tool_0"),
        "response should mention tool name"
    );
}

#[tokio::test]
async fn test_call_tool_unknown_returns_error() {
    let svc = spawn_client(&["--tools", "1"]).await;
    let result = svc
        .call_tool(CallToolRequestParams::new("nonexistent_tool"))
        .await;
    assert!(
        result.is_err(),
        "calling an unknown tool should return MCP error"
    );
}

#[tokio::test]
async fn test_list_resources() {
    let svc = spawn_client(&["--resources", "4"]).await;
    let result = svc.list_resources(None).await.expect("list_resources");
    assert_eq!(result.resources.len(), 4);
}

#[tokio::test]
async fn test_read_resource() {
    let svc = spawn_client(&["--resources", "2"]).await;
    let result = svc
        .read_resource(ReadResourceRequestParams::new("resource://mock/0"))
        .await
        .expect("read_resource");
    assert!(!result.contents.is_empty());
}

#[tokio::test]
async fn test_list_prompts() {
    let svc = spawn_client(&["--prompts", "3"]).await;
    let result = svc.list_prompts(None).await.expect("list_prompts");
    assert_eq!(result.prompts.len(), 3);
}

#[tokio::test]
async fn test_get_prompt() {
    let svc = spawn_client(&["--prompts", "2"]).await;
    let result = svc
        .get_prompt(GetPromptRequestParams::new("mock_prompt_0"))
        .await
        .expect("get_prompt");
    assert!(!result.messages.is_empty());
}

// ── Error injection tests ────────────────────────────────────────────────────

#[tokio::test]
async fn test_error_injection_tool_result() {
    let svc = spawn_client(&["--tools", "2", "--error-tool-result"]).await;
    let result = svc
        .call_tool(CallToolRequestParams::new("mock_tool_0"))
        .await
        .expect("call_tool should succeed at protocol level");
    assert!(
        result.is_error.unwrap_or(false),
        "isError should be true when --error-tool-result is set"
    );
}

// ── Pagination tests ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_pagination() {
    let svc = spawn_client(&["--tools", "6", "--page-size", "2"]).await;

    // First page
    let r1 = svc
        .list_tools(Some(PaginatedRequestParams::default()))
        .await
        .expect("page 1");
    assert_eq!(r1.tools.len(), 2, "first page should have 2 tools");
    assert!(r1.next_cursor.is_some(), "should have a next cursor");

    // Second page
    let r2 = svc
        .list_tools(Some(
            PaginatedRequestParams::default().with_cursor(r1.next_cursor.clone()),
        ))
        .await
        .expect("page 2");
    assert_eq!(r2.tools.len(), 2, "second page should have 2 tools");
    assert!(r2.next_cursor.is_some());

    // Third (last) page
    let r3 = svc
        .list_tools(Some(
            PaginatedRequestParams::default().with_cursor(r2.next_cursor.clone()),
        ))
        .await
        .expect("page 3");
    assert_eq!(r3.tools.len(), 2, "third page should have 2 tools");
    assert!(
        r3.next_cursor.is_none(),
        "last page should have no next cursor"
    );
}

#[tokio::test]
async fn test_list_all_tools_via_pagination() {
    let svc = spawn_client(&["--tools", "7", "--page-size", "3"]).await;
    let all = svc.list_all_tools().await.expect("list_all_tools");
    assert_eq!(all.len(), 7, "should collect all 7 tools via pagination");
}

#[tokio::test]
async fn test_zero_tools() {
    let svc = spawn_client(&["--tools", "0"]).await;
    let result = svc
        .list_tools(None)
        .await
        .expect("list_tools with 0 tools");
    assert!(result.tools.is_empty());
    assert!(result.next_cursor.is_none());
}

// ── Schema drift test ────────────────────────────────────────────────────────

#[tokio::test]
async fn test_schema_drift() {
    // drift_after=1 means: after the atomic counter reaches >= 1, drift is active.
    let svc = spawn_client(&["--tools", "2", "--drift-after", "1"]).await;

    // First call increments counter to 1, meeting drift_after_n_calls=1.
    let _r1 = svc.list_tools(None).await.expect("list_tools call 1");
    // Second call: drift is active.
    let r2 = svc.list_tools(None).await.expect("list_tools call 2");

    let has_drifted_param = r2.tools.iter().any(|t| {
        t.input_schema
            .get("properties")
            .and_then(|v| v.as_object())
            .map(|p| p.contains_key("drifted_param"))
            .unwrap_or(false)
    });
    assert!(
        has_drifted_param,
        "schema should include drifted_param after drift_after threshold"
    );
}

// ── Config file test ─────────────────────────────────────────────────────────

#[tokio::test]
async fn test_config_file() {
    let config = serde_json::json!({
        "tools": [
            {"name": "alpha", "description": "Alpha tool", "extra_params": []},
            {"name": "beta",  "description": "Beta tool",  "extra_params": []}
        ],
        "resources": [],
        "prompts": [],
        "capabilities": {
            "tools": true, "resources": false, "prompts": false,
            "logging": false, "completions": false
        },
        "pagination": {"page_size": 0, "loop_cursor": false},
        "error_injection": {
            "delay_ms": 0, "crash_on_tool_call": false,
            "error_tool_result": false, "emit_malformed": false
        },
        "drift_config": null
    });

    let tmp_dir = env::temp_dir();
    let path = tmp_dir.join(format!("forge-test-config-{}.json", std::process::id()));
    std::fs::write(&path, config.to_string()).expect("write config");

    let bin = test_server_bin();
    let mut cmd = Command::new(&bin);
    cmd.arg("--config")
        .arg(&path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

    let proc = TokioChildProcess::new(cmd).expect("spawn");
    let svc = serve_client((), proc).await.expect("serve_client");

    let tools = svc.list_tools(None).await.expect("list_tools");
    let _ = std::fs::remove_file(&path);

    assert_eq!(tools.tools.len(), 2);
    assert_eq!(tools.tools[0].name.as_ref(), "alpha");
    assert_eq!(tools.tools[1].name.as_ref(), "beta");
}
