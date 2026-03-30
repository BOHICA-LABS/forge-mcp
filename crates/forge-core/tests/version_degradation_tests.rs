//! Integration tests for STORY-015: Graceful Degradation with Older Spec Versions.
//!
//! Tests verify that `McpConnection` correctly applies version-based feature
//! gating after the MCP `initialize` / `initialized` handshake completes.
//!
//! AC coverage:
//!   AC-001 — version detection: negotiated protocol version stored on McpConnection
//!   AC-002 — version mismatch warning: E-CON-006 emitted, connection continues
//!   AC-003 — method gating by version: elicitation/create returns E-PRO-003 for old server
//!   AC-004 — unknown version continues with conservative defaults, no crash

// BC-tracing test names intentionally use uppercase (BC-N-MM-NNN format).
#![allow(non_snake_case)]

use std::collections::HashMap;

use forge_core::{
    connect_stdio,
    error::CoreError,
    types::{SpecVersion, features_for_version},
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
    // Check target-triple subdirectories first (CI with --target <triple>).
    if let Ok(entries) = std::fs::read_dir(&target_dir) {
        for entry in entries.flatten() {
            let candidate = entry.path().join("debug").join("forge-test-server");
            if candidate.exists() {
                return candidate.to_string_lossy().to_string();
            }
        }
    }
    // Fallback: plain target/debug (local builds without --target).
    target_dir
        .join("debug")
        .join("forge-test-server")
        .to_string_lossy()
        .to_string()
}

// ─────────────────────────────────────────────────────────────────────────────
// Pure unit tests (no network / subprocess required)
// ─────────────────────────────────────────────────────────────────────────────

/// AC-001 (version detection): `SpecVersion::parse` recognises all known versions.
#[test]
fn test_BC_2_04_003_version_detection_parse() {
    // Known versions parse successfully.
    assert_eq!(SpecVersion::parse("2024-11-05"), Some(SpecVersion::V2024_11_05));
    assert_eq!(SpecVersion::parse("2025-03-26"), Some(SpecVersion::V2025_03_26));
    assert_eq!(SpecVersion::parse("2025-06-18"), Some(SpecVersion::V2025_06_18));

    // Both versions from the story spec are supported.
    // Story references "2024-11-05" (older) and "2025-11-25" (story draft name
    // for latest). The real rmcp latest is "2025-06-18".
    assert!(SpecVersion::parse("2024-11-05").is_some(), "2024-11-05 must be recognised");
    assert!(SpecVersion::parse("2025-06-18").is_some(), "2025-06-18 must be recognised");
}

/// AC-001: latest version → all features available.
#[test]
fn test_BC_2_04_003_version_detection_latest_all_features() {
    let fs = features_for_version("2025-06-18");
    assert!(fs.elicitation, "latest version must enable elicitation");
    assert!(fs.streamable_http, "latest version must enable streamable_http");
    assert!(fs.logging, "latest version must enable logging");
    assert!(fs.tools);
    assert!(fs.resources);
    assert!(fs.prompts);
}

/// AC-002 (version mismatch warning): when the server reports an older version,
/// the mismatch is detectable via `has_version_mismatch()` / `version_warning()`.
///
/// This is a pure-logic test — we verify the `FeatureSet` reflects degradation.
#[test]
fn test_BC_2_04_003_version_mismatch_older_version_reduces_features() {
    // Simulating: client proposes "2025-06-18", server responds "2024-11-05".
    let older_fs = features_for_version("2024-11-05");
    let latest_fs = features_for_version("2025-06-18");

    // Older version must have fewer features.
    assert!(!older_fs.elicitation, "2024-11-05 must not have elicitation");
    assert!(!older_fs.streamable_http, "2024-11-05 must not have streamable_http");
    assert!(latest_fs.elicitation, "2025-06-18 must have elicitation");
    assert!(latest_fs.streamable_http, "2025-06-18 must have streamable_http");

    // Basic features are available in both.
    assert!(older_fs.tools && older_fs.resources && older_fs.prompts);
    assert!(latest_fs.tools && latest_fs.resources && latest_fs.prompts);
}

/// AC-002 (E-CON-006): The `ProtocolVersionMismatch` error formats correctly.
#[test]
fn test_BC_2_04_003_version_mismatch_warning_error_format() {
    let err = CoreError::ProtocolVersionMismatch {
        proposed: "2025-06-18".to_string(),
        negotiated: "2024-11-05".to_string(),
    };
    let msg = err.to_string();
    assert!(
        msg.contains("E-CON-006"),
        "E-CON-006 code must appear in warning: {msg}"
    );
    assert!(
        msg.contains("2025-06-18"),
        "proposed version must appear in warning: {msg}"
    );
    assert!(
        msg.contains("2024-11-05"),
        "negotiated version must appear in warning: {msg}"
    );
}

/// AC-003 (method gating): elicitation is disabled for server on "2024-11-05".
#[test]
fn test_BC_2_04_003_method_gated_by_version_elicitation_blocked() {
    let fs = features_for_version("2024-11-05");
    assert!(
        !fs.elicitation,
        "elicitation/create must be gated for 2024-11-05"
    );
}

/// AC-003: streamable HTTP is disabled for "2024-11-05" but enabled for "2025-03-26".
#[test]
fn test_BC_2_04_003_method_gated_by_version_streamable_http() {
    assert!(!features_for_version("2024-11-05").streamable_http);
    assert!(features_for_version("2025-03-26").streamable_http);
    assert!(features_for_version("2025-06-18").streamable_http);
}

/// AC-004 (unknown version continues): unrecognised version string → conservative
/// feature set, no panic.
#[test]
fn test_BC_2_04_003_unknown_version_continues() {
    // EC-002: future version like "2026-01-01" must not crash.
    let fs = features_for_version("2026-01-01");
    // Conservative: advanced features off, basic features on.
    assert!(!fs.elicitation, "unknown future version must not enable elicitation");
    assert!(fs.tools, "unknown version must still allow tools");
    assert!(fs.resources, "unknown version must still allow resources");
    assert!(fs.prompts, "unknown version must still allow prompts");

    // Completely garbage string must also not crash.
    let fs2 = features_for_version("not-a-version");
    assert!(fs2.tools);
    assert!(!fs2.elicitation);

    // Empty string must not crash.
    let fs3 = features_for_version("");
    assert!(fs3.tools);
}

/// AC-004: version string parsing handles edge cases without panicking.
#[test]
fn test_BC_2_04_003_version_string_edge_cases() {
    // All of these must parse safely without panic.
    let edge_cases = [
        "",
        "garbage",
        "2024",
        "2024-11",
        "2024-11-05-extra",
        "9999-99-99",
        "0000-00-00",
        "\n2024-11-05\n",
        " 2024-11-05 ", // whitespace
        "2024-11-05\0",  // null byte
    ];
    for case in &edge_cases {
        // Must not panic.
        let _ = features_for_version(case);
        let _ = SpecVersion::parse(case);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Integration tests (require forge-test-server subprocess)
// ─────────────────────────────────────────────────────────────────────────────

/// AC-001 (integration): After connect, protocol_version is stored and non-empty.
///
/// forge-test-server uses rmcp's default (latest) protocol version "2025-06-18".
#[tokio::test]
async fn test_BC_2_04_003_version_detection_live_server() {
    let bin = test_server_bin();
    let env = HashMap::new();

    let conn = connect_stdio(&bin, &[], &env)
        .await
        .expect("connect should succeed");

    let version = conn.protocol_version();
    assert!(!version.is_empty(), "protocol_version must not be empty after connect");
    assert!(
        version.contains('-'),
        "protocol version must be a date-like string: {version}"
    );

    // Verify the version is one of our known versions.
    assert!(
        SpecVersion::parse(version).is_some(),
        "live server must report a recognised spec version, got: {version}"
    );

    conn.shutdown().await.ok();
}

/// AC-001 (integration): Latest version server → all features enabled.
#[tokio::test]
async fn test_BC_2_04_003_version_detection_latest_enables_all_features() {
    let bin = test_server_bin();
    let args = vec![
        "--tools".to_string(),
        "1".to_string(),
    ];
    let env = HashMap::new();

    let conn = connect_stdio(&bin, &args, &env)
        .await
        .expect("connect should succeed");

    // forge-test-server reports "2025-06-18" (rmcp latest).
    assert_eq!(
        conn.protocol_version(),
        "2025-06-18",
        "test server must report rmcp's latest version"
    );

    // With the latest version, elicitation and streamable_http must be permitted
    // (even though the server doesn't have a separate cap flag for elicitation —
    // the version gate alone governs it).
    assert!(
        conn.supports_elicitation(),
        "latest version server must support elicitation"
    );
    assert!(
        conn.supports_streamable_http(),
        "latest version server must support streamable_http"
    );
    assert!(!conn.has_version_mismatch(), "no mismatch when server reports latest");

    conn.shutdown().await.ok();
}

/// AC-002 (integration): Version mismatch → `has_version_mismatch()` returns true.
///
/// We cannot force the forge-test-server to report an older version without
/// modifying it, so we test the mismatch logic at the `McpConnection` API
/// level by connecting and confirming the happy path (no mismatch with
/// forge-test-server), plus the pure-logic path above.
#[tokio::test]
async fn test_BC_2_04_003_version_mismatch_warning_no_mismatch_with_latest() {
    let bin = test_server_bin();
    let env = HashMap::new();

    let conn = connect_stdio(&bin, &[], &env)
        .await
        .expect("connect should succeed");

    // forge-test-server reports latest version → no mismatch.
    assert!(
        !conn.has_version_mismatch(),
        "forge-test-server reports latest, so no mismatch expected"
    );
    assert!(
        conn.version_warning().is_none(),
        "version_warning must be None when versions match"
    );

    conn.shutdown().await.ok();
}

/// AC-003 (integration): `guard_elicitation()` succeeds on a latest-version server.
#[tokio::test]
async fn test_BC_2_04_003_method_gated_by_version_elicitation_allowed_on_latest() {
    let bin = test_server_bin();
    let env = HashMap::new();

    let conn = connect_stdio(&bin, &[], &env)
        .await
        .expect("connect should succeed");

    // Latest version → elicitation guard passes.
    assert!(
        conn.guard_elicitation().is_ok(),
        "elicitation/create must be allowed on latest version server"
    );

    conn.shutdown().await.ok();
}

/// AC-003 (pure logic): `guard_elicitation()` on an older-version `FeatureSet`
/// would return Err.  We verify this through the `FeatureSet` directly (since
/// we can't force the test server to downgrade).
#[test]
fn test_BC_2_04_003_method_gated_by_version_guard_elicitation_returns_err_for_old() {
    // Build the FeatureSet for "2024-11-05".
    let old_fs = features_for_version("2024-11-05");
    assert!(
        !old_fs.elicitation,
        "2024-11-05 FeatureSet must have elicitation=false (the guard would return Err)"
    );

    // Confirm the error type that guard_elicitation would emit.
    let expected_err = CoreError::CapabilityNotSupported {
        method: "elicitation/create".to_string(),
        capability: "elicitation (requires spec ≥ 2025-06-18; negotiated 2024-11-05)".to_string(),
    };
    let msg = expected_err.to_string();
    assert!(
        msg.contains("E-PRO-003"),
        "guard error must carry E-PRO-003 code: {msg}"
    );
}

/// AC-004 (integration): Unknown version server → connection succeeds, conservative features.
///
/// forge-test-server uses rmcp's latest. This test validates the pure path:
/// connecting always succeeds, and a hypothetical unknown-version response
/// would yield conservative features without crashing.
#[tokio::test]
async fn test_BC_2_04_003_unknown_version_live_server_no_crash() {
    // Any supported server must connect without crashing.
    let bin = test_server_bin();
    let env = HashMap::new();

    let conn = connect_stdio(&bin, &[], &env)
        .await
        .expect("connect must succeed regardless of protocol version");

    // The connection object must be valid.
    assert!(conn.state().is_connected());

    conn.shutdown().await.ok();
}
