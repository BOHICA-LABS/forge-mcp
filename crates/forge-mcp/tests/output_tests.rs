//! Integration tests for STORY-024: Structured JSON Output & Agent-Optimized Tokens
//!
//! Tests are named after their behavioral-contract identifiers:
//!   BC-5.11.002 — JSON on stdout, diagnostics on stderr
//!   BC-5.12.001 — Token budget and minimal default output

use std::env;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use tokio::process::Command;
use tokio::time::timeout;

// ── Helpers ────────────────────────────────────────────────────────────────

/// Locate the `forge-mcp` binary produced by Cargo.
fn forge_mcp_bin() -> PathBuf {
    // Set by Cargo when running integration tests for a binary crate.
    if let Ok(p) = env::var("CARGO_BIN_EXE_forge-mcp") {
        return PathBuf::from(p);
    }
    // Fallback: search relative to the test binary location.
    let bin_name = if cfg!(windows) {
        "forge-mcp.exe"
    } else {
        "forge-mcp"
    };
    let exe = env::current_exe().expect("current_exe");
    let dir = exe.parent().expect("exe parent");
    let candidate = dir.join(bin_name);
    if candidate.exists() {
        return candidate;
    }
    dir.parent()
        .map(|p| p.join(bin_name))
        .filter(|p| p.exists())
        .unwrap_or(candidate)
}

/// Locate the `forge-test-server` binary.
fn test_server_bin() -> PathBuf {
    if let Ok(p) = env::var("CARGO_BIN_EXE_forge-test-server") {
        return PathBuf::from(p);
    }
    let bin_name = if cfg!(windows) {
        "forge-test-server.exe"
    } else {
        "forge-test-server"
    };
    let exe = env::current_exe().expect("current_exe");
    let dir = exe.parent().expect("exe parent");
    let candidate = dir.join(bin_name);
    if candidate.exists() {
        return candidate;
    }
    dir.parent()
        .map(|p| p.join(bin_name))
        .filter(|p| p.exists())
        .unwrap_or(candidate)
}

/// Run `forge-mcp` with the given args; capture stdout + stderr separately.
/// Returns `(stdout_str, stderr_str, exit_status)`.
async fn run_forge_mcp(args: &[&str]) -> (String, String, std::process::ExitStatus) {
    let bin = forge_mcp_bin();
    let output = timeout(
        Duration::from_secs(15),
        Command::new(&bin)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output(),
    )
    .await
    .expect("forge-mcp timed out")
    .expect("forge-mcp spawn failed");

    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (stdout, stderr, output.status)
}

/// Build a `stdio://<path>` URI pointing to the test server binary.
fn stdio_uri() -> String {
    let bin = test_server_bin();
    format!("stdio://{}", bin.display())
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-001: All command results on stdout as JSON; diagnostics on stderr.
// ─────────────────────────────────────────────────────────────────────────────

/// AC-001 — `forge-mcp list <server>` emits valid JSON on stdout.
/// No JSON must appear on stderr (only diagnostics / log lines allowed there).
#[tokio::test]
async fn test_bc_5_11_002_json_stdout_diagnostics_stderr() {
    let server = stdio_uri();
    let (stdout, _stderr, status) = run_forge_mcp(&["list", &server]).await;

    // Exit 0 on success
    assert!(
        status.success(),
        "expected exit 0, got: {status:?}\nstdout: {stdout}\nstderr: {_stderr}"
    );

    // stdout must be parseable JSON
    let parsed: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|e| panic!("stdout is not valid JSON: {e}\nstdout: {stdout}"));

    // stdout must be an object (not an array or scalar)
    assert!(
        parsed.is_object(),
        "stdout JSON must be an object, got: {parsed}"
    );

    // No JSON on stderr — stderr may contain log lines but not structured JSON objects
    // (a very simple heuristic: stderr should not start with '{' after trimming)
    let stderr_trimmed = _stderr.trim();
    if !stderr_trimmed.is_empty() {
        assert!(
            !stderr_trimmed.starts_with('{'),
            "stderr should not contain JSON; it should be diagnostic text only.\nstderr: {stderr_trimmed}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-002: Output schemas.
// ─────────────────────────────────────────────────────────────────────────────

/// AC-002 — Output schemas for all four commands.
///
/// list  → `{ "servers": [...] }` when no server given,
///         `{ "tools": [...] }` when a server URI is given.
/// call  → `{ "result": { "content": [...], "isError": bool } }`
/// info  → `{ "server": {...}, "capabilities": {...} }`
/// grep  → `{ "matches": [...] }`
#[tokio::test]
async fn test_bc_5_11_002_output_schemas() {
    let server = stdio_uri();

    // ── list <server> → { "tools": [...] }
    {
        let (stdout, _stderr, status) = run_forge_mcp(&["list", &server]).await;
        assert!(
            status.success(),
            "list failed\nstdout: {stdout}\nstderr: {_stderr}"
        );
        let v: serde_json::Value = serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("list stdout not JSON: {e}\n{stdout}"));
        assert!(
            v.get("tools").map(|x| x.is_array()).unwrap_or(false),
            "list <server> must return {{\"tools\": [...]}}, got: {v}"
        );
    }

    // ── list (no server) → { "servers": [...] }
    {
        let (stdout, _stderr, status) = run_forge_mcp(&["list"]).await;
        assert!(
            status.success(),
            "list (no server) failed\nstdout: {stdout}\nstderr: {_stderr}"
        );
        let v: serde_json::Value = serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("list stdout not JSON: {e}\n{stdout}"));
        assert!(
            v.get("servers").map(|x| x.is_array()).unwrap_or(false),
            "list (no server) must return {{\"servers\": [...]}}, got: {v}"
        );
    }

    // ── call <server> <tool> → { "result": { "content": [...], "isError": bool } }
    {
        let (stdout, _stderr, status) = run_forge_mcp(&["call", &server, "mock_tool_0"]).await;
        assert!(
            status.success(),
            "call failed\nstdout: {stdout}\nstderr: {_stderr}"
        );
        let v: serde_json::Value = serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("call stdout not JSON: {e}\n{stdout}"));
        let result = v.get("result").expect("call output must have 'result' key");
        assert!(
            result.get("content").map(|x| x.is_array()).unwrap_or(false),
            "call result must contain 'content' array, got: {v}"
        );
        assert!(
            result.get("isError").is_some(),
            "call result must contain 'isError' field, got: {v}"
        );
    }

    // ── info <server> → { "server": {...}, "capabilities": {...} }
    {
        let (stdout, _stderr, status) = run_forge_mcp(&["info", &server]).await;
        assert!(
            status.success(),
            "info failed\nstdout: {stdout}\nstderr: {_stderr}"
        );
        let v: serde_json::Value = serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("info stdout not JSON: {e}\n{stdout}"));
        assert!(
            v.get("server").map(|x| x.is_object()).unwrap_or(false),
            "info output must have 'server' object, got: {v}"
        );
        assert!(
            v.get("capabilities")
                .map(|x| x.is_object())
                .unwrap_or(false),
            "info output must have 'capabilities' object, got: {v}"
        );
    }

    // ── grep <pattern> → { "matches": [...] }
    {
        let (stdout, _stderr, status) = run_forge_mcp(&["grep", "mock"]).await;
        assert!(
            status.success(),
            "grep failed\nstdout: {stdout}\nstderr: {_stderr}"
        );
        let v: serde_json::Value = serde_json::from_str(&stdout)
            .unwrap_or_else(|e| panic!("grep stdout not JSON: {e}\n{stdout}"));
        assert!(
            v.get("matches").map(|x| x.is_array()).unwrap_or(false),
            "grep output must have 'matches' array, got: {v}"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-003: list + call combined output ≤ 500 tokens.
// Token approximation: output_bytes / 4 (GPT-4 byte heuristic).
// ─────────────────────────────────────────────────────────────────────────────

/// AC-003 — Combined stdout bytes for `list <server>` + `call <server> <tool>`
/// must fit within 500 approximate tokens (bytes / 4 ≤ 500).
#[tokio::test]
async fn test_bc_5_12_001_token_count_within_budget() {
    let server = stdio_uri();

    let (list_out, _, list_status) = run_forge_mcp(&["list", &server]).await;
    assert!(list_status.success(), "list failed: {list_out}");

    let (call_out, _, call_status) = run_forge_mcp(&["call", &server, "mock_tool_0"]).await;
    assert!(call_status.success(), "call failed: {call_out}");

    let combined_bytes = list_out.len() + call_out.len();
    let approx_tokens = combined_bytes / 4;

    assert!(
        approx_tokens <= 500,
        "Combined list+call output exceeds 500 token budget: \
         {combined_bytes} bytes ≈ {approx_tokens} tokens.\n\
         list output ({} bytes): {list_out}\n\
         call output ({} bytes): {call_out}",
        list_out.len(),
        call_out.len()
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-004: Default output is minimal; --verbose enables extended metadata.
// ─────────────────────────────────────────────────────────────────────────────

/// AC-004 — Default output is minimal (no metadata fields like `_meta`,
/// `_verbose`, `timing`, etc.). `--verbose` output is larger.
#[tokio::test]
async fn test_bc_5_12_001_minimal_default_output() {
    let server = stdio_uri();

    // Default output
    let (default_out, _, status) = run_forge_mcp(&["list", &server]).await;
    assert!(status.success(), "list (default) failed: {default_out}");

    // Verbose output
    let (verbose_out, _, _) = run_forge_mcp(&["--verbose", "list", &server]).await;

    // Default must be strictly smaller than verbose
    assert!(
        default_out.len() < verbose_out.len(),
        "default output ({} bytes) should be smaller than --verbose output ({} bytes).\n\
         default: {default_out}\n\
         verbose: {verbose_out}",
        default_out.len(),
        verbose_out.len()
    );

    // Default output must NOT contain metadata-only keys
    let default_val: serde_json::Value = serde_json::from_str(&default_out)
        .unwrap_or_else(|e| panic!("default output not JSON: {e}"));
    let has_meta = default_val.get("_meta").is_some()
        || default_val.get("timing").is_some()
        || default_val.get("metadata").is_some();
    assert!(
        !has_meta,
        "default output must not contain metadata fields, got: {default_val}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-005: --pretty flag enables indented JSON; default is compact.
// ─────────────────────────────────────────────────────────────────────────────

/// AC-005 — `--pretty` produces indented (multi-line) JSON.
/// Default output is compact (single-line, no internal newlines).
#[tokio::test]
async fn test_bc_5_11_002_pretty_print_flag() {
    let server = stdio_uri();

    // Default: compact (no newlines inside the JSON object)
    let (compact_out, _, status) = run_forge_mcp(&["list", &server]).await;
    assert!(status.success(), "list (compact) failed: {compact_out}");
    let compact_trimmed = compact_out.trim();
    assert!(
        !compact_trimmed.contains('\n'),
        "default (compact) output must be single-line JSON, got:\n{compact_trimmed}"
    );

    // --pretty: multi-line indented JSON
    let (pretty_out, _, pretty_status) = run_forge_mcp(&["--pretty", "list", &server]).await;
    assert!(
        pretty_status.success(),
        "list --pretty failed: {pretty_out}"
    );
    let pretty_trimmed = pretty_out.trim();
    assert!(
        pretty_trimmed.contains('\n'),
        "--pretty output must be multi-line JSON, got:\n{pretty_trimmed}"
    );

    // Both must parse to the same JSON value (identical data, different formatting)
    let compact_val: serde_json::Value = serde_json::from_str(compact_trimmed)
        .unwrap_or_else(|e| panic!("compact output not valid JSON: {e}"));
    let pretty_val: serde_json::Value = serde_json::from_str(pretty_trimmed)
        .unwrap_or_else(|e| panic!("pretty output not valid JSON: {e}"));
    assert_eq!(
        compact_val, pretty_val,
        "compact and --pretty outputs must represent the same JSON data"
    );
}
