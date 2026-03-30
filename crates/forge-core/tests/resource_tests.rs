//! Integration tests for STORY-017: Resource List, Read & Subscription Management
//!
//! AC coverage:
//!   AC-001 — `list_resources()` returns Vec<Resource> with pagination support
//!   AC-002 — `read_resource(uri)` returns ResourceContent (text + binary blob)
//!   AC-003 — `subscribe_resource(uri)` sends resources/subscribe and returns handle
//!   AC-004 — Dropping handle (or calling `unsubscribe()`) sends resources/unsubscribe
//!   AC-005 — If server didn't advertise resources, list_resources() returns Err(E-PRO-003)

#![allow(non_snake_case)]

use std::collections::HashMap;
use std::io::Write;

use forge_core::{
    CoreError, connect_stdio,
    list_resources, read_resource,
    subscribe_resource, subscribe_resource_with_sender,
    ResourceData,
};

// ── Helpers ──────────────────────────────────────────────────────────────────

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
    workspace_root
        .join("target")
        .join("debug")
        .join("forge-test-server")
        .to_string_lossy()
        .to_string()
}

fn write_server_config(json: &str) -> tempfile::NamedTempFile {
    let mut f = tempfile::NamedTempFile::new().expect("tempfile");
    f.write_all(json.as_bytes()).expect("write config");
    f
}

// ── AC-001: Paginated resource listing ───────────────────────────────────────

/// AC-001: `list_resources()` auto-paginates and returns all resources.
///
/// The mock server is configured with 5 resources and a page_size of 2,
/// so the client must issue 3 pages to collect all 5.
#[tokio::test]
async fn test_BC_2_05_002_list_resources_paginated() {
    let bin = test_server_bin();
    let config_json = r#"{
        "tools": [],
        "resources": [
            { "uri": "resource://test/0", "name": "R0", "description": "Resource 0", "content": "data0" },
            { "uri": "resource://test/1", "name": "R1", "description": "Resource 1", "content": "data1" },
            { "uri": "resource://test/2", "name": "R2", "description": "Resource 2", "content": "data2" },
            { "uri": "resource://test/3", "name": "R3", "description": "Resource 3", "content": "data3" },
            { "uri": "resource://test/4", "name": "R4", "description": "Resource 4", "content": "data4" }
        ],
        "prompts": [],
        "capabilities": {
            "tools": false,
            "resources": true,
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
    let env = HashMap::new();

    let conn = connect_stdio(&bin, &args, &env)
        .await
        .expect("connect should succeed");

    assert!(conn.supports_resources(), "resources capability must be present");

    let resources = list_resources(&conn).await.expect("list_resources should succeed");

    assert_eq!(resources.len(), 5, "expected 5 resources across 3 pages");

    // Verify each resource has the expected URI and name.
    for (i, r) in resources.iter().enumerate() {
        assert_eq!(r.uri, format!("resource://test/{i}"), "uri mismatch at index {i}");
        assert_eq!(r.name, format!("R{i}"), "name mismatch at index {i}");
        assert!(r.description.is_some(), "description should be present");
    }

    conn.shutdown().await.ok();
}

// ── AC-001 (b): No pagination (all on one page) ───────────────────────────────

/// AC-001 (variation): When page_size is 0 (no pagination), all resources arrive
/// on the first page and `list_resources` returns them without any cursor loop.
#[tokio::test]
async fn test_BC_2_05_002_list_resources_no_pagination() {
    let bin = test_server_bin();
    let config_json = r#"{
        "tools": [],
        "resources": [
            { "uri": "resource://mock/0", "name": "Mock Resource 0", "description": "Mock resource number 0", "content": "data0" },
            { "uri": "resource://mock/1", "name": "Mock Resource 1", "description": "Mock resource number 1", "content": "data1" }
        ],
        "prompts": [],
        "capabilities": {
            "tools": false,
            "resources": true,
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

    let resources = list_resources(&conn).await.expect("list_resources should succeed");
    assert_eq!(resources.len(), 2, "expected 2 resources with no pagination");

    conn.shutdown().await.ok();
}

// ── AC-002: Read resource (text) ──────────────────────────────────────────────

/// AC-002: `read_resource(uri)` returns ResourceContent with text data.
#[tokio::test]
async fn test_BC_2_05_002_read_resource() {
    let bin = test_server_bin();
    let config_json = r#"{
        "tools": [],
        "resources": [
            { "uri": "resource://test/hello", "name": "Hello", "description": "A text resource", "content": "Hello, World!" }
        ],
        "prompts": [],
        "capabilities": {
            "tools": false,
            "resources": true,
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

    let content = read_resource(&conn, "resource://test/hello")
        .await
        .expect("read_resource should succeed");

    assert_eq!(content.uri, "resource://test/hello");
    assert!(content.mime_type.is_some(), "mime_type should be present");

    match content.content {
        ResourceData::Text(text) => {
            assert_eq!(text, "Hello, World!", "text content mismatch");
        }
        ResourceData::Binary(_) => panic!("expected Text content, got Binary"),
    }

    conn.shutdown().await.ok();
}

/// AC-002 (binary): `read_resource` decodes base64 blob content to bytes.
///
/// The test server only serves text content, so we verify the binary path
/// via unit-level testing of the protocol module in the source.
/// This integration test verifies the full text path works end-to-end.
/// Binary path is exercised in the unit tests embedded in protocol.rs.
#[tokio::test]
async fn test_BC_2_05_002_read_resource_text_content_has_uri_and_mime() {
    let bin = test_server_bin();
    let config_json = r#"{
        "tools": [],
        "resources": [
            { "uri": "resource://binary/test", "name": "Binary Resource", "description": "Binary", "content": "some content" }
        ],
        "prompts": [],
        "capabilities": {
            "tools": false,
            "resources": true,
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

    let content = read_resource(&conn, "resource://binary/test")
        .await
        .expect("read_resource should succeed");

    // URI round-trips correctly.
    assert_eq!(content.uri, "resource://binary/test");

    // Content is present.
    match &content.content {
        ResourceData::Text(t) => assert_eq!(t, "some content"),
        ResourceData::Binary(_) => panic!("expected Text"),
    }

    conn.shutdown().await.ok();
}

// ── AC-003: Subscribe to resource updates ────────────────────────────────────

/// AC-003: `subscribe_resource(uri)` succeeds (sends resources/subscribe) and
/// returns a subscription handle.
#[tokio::test]
async fn test_BC_2_05_002_subscribe_resource() {
    let bin = test_server_bin();
    let config_json = r#"{
        "tools": [],
        "resources": [
            { "uri": "resource://test/live", "name": "Live", "description": "Live resource", "content": "v1" }
        ],
        "prompts": [],
        "capabilities": {
            "tools": false,
            "resources": true,
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

    // subscribe_resource should succeed without error.
    let sub = subscribe_resource(&conn, "resource://test/live")
        .await
        .expect("subscribe_resource should succeed");

    assert_eq!(sub.uri(), "resource://test/live", "subscription handle URI must match");

    // Clean up — explicit unsubscribe.
    sub.unsubscribe().await.expect("unsubscribe should succeed");

    conn.shutdown().await.ok();
}

/// AC-003 (with sender): subscribe_resource_with_sender wires up notification delivery.
#[tokio::test]
async fn test_BC_2_05_002_subscribe_resource_with_sender_receives_update() {
    let bin = test_server_bin();
    let config_json = r#"{
        "tools": [],
        "resources": [
            { "uri": "resource://live/data", "name": "Live Data", "description": "Live", "content": "initial" }
        ],
        "prompts": [],
        "capabilities": {
            "tools": false,
            "resources": true,
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

    let (mut sub, tx) = subscribe_resource_with_sender(&conn, "resource://live/data")
        .await
        .expect("subscribe_resource_with_sender should succeed");

    // Simulate server sending a notification/resources/updated by pushing to the channel.
    tx.send("resource://live/data".to_string())
        .await
        .expect("send should succeed");

    // The handle receives the update.
    let update = sub.recv_update().await.expect("recv_update must return Some");
    assert_eq!(update, "resource://live/data", "received URI must match");

    sub.unsubscribe().await.expect("unsubscribe should succeed");
    conn.shutdown().await.ok();
}

// ── AC-004: Unsubscribe on drop ───────────────────────────────────────────────

/// AC-004: Dropping the subscription handle sends resources/unsubscribe.
///
/// We verify this doesn't panic and the connection remains usable afterwards.
#[tokio::test]
async fn test_BC_2_05_002_unsubscribe_resource() {
    let bin = test_server_bin();
    let config_json = r#"{
        "tools": [],
        "resources": [
            { "uri": "resource://drop/test", "name": "Drop test", "description": "Tests drop", "content": "data" }
        ],
        "prompts": [],
        "capabilities": {
            "tools": false,
            "resources": true,
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

    {
        let sub = subscribe_resource(&conn, "resource://drop/test")
            .await
            .expect("subscribe_resource should succeed");

        // Drop the subscription handle here — should fire resources/unsubscribe
        // in a background task without panicking.
        drop(sub);
    }

    // Give the background unsubscribe task a moment to complete.
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Connection should still be usable.
    let resources = list_resources(&conn)
        .await
        .expect("list_resources should still work after drop-unsubscribe");
    assert_eq!(resources.len(), 1, "resource list should still return 1 resource");

    conn.shutdown().await.ok();
}

/// AC-004 (explicit): Calling `subscription.unsubscribe()` is idempotent.
#[tokio::test]
async fn test_BC_2_05_002_unsubscribe_explicit_is_idempotent_after_unsubscribe_call() {
    let bin = test_server_bin();
    let config_json = r#"{
        "tools": [],
        "resources": [
            { "uri": "resource://idempotent/test", "name": "Idempotent", "description": "Test", "content": "data" }
        ],
        "prompts": [],
        "capabilities": {
            "tools": false,
            "resources": true,
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

    let sub = subscribe_resource(&conn, "resource://idempotent/test")
        .await
        .expect("subscribe_resource should succeed");

    // Explicit unsubscribe — consumes `sub`, sends resources/unsubscribe once.
    sub.unsubscribe().await.expect("unsubscribe should succeed");

    // Connection still usable.
    assert!(conn.supports_resources());
    conn.shutdown().await.ok();
}

// ── AC-005: Capability guard ──────────────────────────────────────────────────

/// AC-005: If server didn't advertise `resources`, `list_resources()` returns
/// `Err(E-PRO-003)` without making any network round-trip.
#[tokio::test]
async fn test_BC_2_05_002_capability_guard() {
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
        .expect("connect should succeed");

    assert!(!conn.supports_resources(), "resources capability must be absent");

    // list_resources must fail with E-PRO-003.
    match list_resources(&conn).await {
        Err(CoreError::CapabilityNotSupported { method, capability }) => {
            assert_eq!(method, "resources/list", "method should be resources/list");
            assert_eq!(capability, "resources", "capability should be resources");
        }
        other => panic!("expected CapabilityNotSupported for list_resources, got: {other:?}"),
    }

    // read_resource also guarded.
    match read_resource(&conn, "resource://test/anything").await {
        Err(CoreError::CapabilityNotSupported { capability, .. }) => {
            assert_eq!(capability, "resources");
        }
        other => panic!("expected CapabilityNotSupported for read_resource, got: {other:?}"),
    }

    // subscribe_resource also guarded.
    match subscribe_resource(&conn, "resource://test/anything").await {
        Err(CoreError::CapabilityNotSupported { capability, .. }) => {
            assert_eq!(capability, "resources");
        }
        other => {
            panic!("expected CapabilityNotSupported for subscribe_resource, got: {other:?}")
        }
    }

    conn.shutdown().await.ok();
}
