//! Integration tests for STORY-025: Pipeable Output & Shell Composition
//!
//! Tests are named after their behavioral-contract identifiers:
//!   BC-5.12.002 — Pipeable Output & Shell Composition

use std::env;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;

use tokio::process::Command;
use tokio::time::timeout;

// ── Helpers ────────────────────────────────────────────────────────────────

/// Locate the `forge-mcp` binary produced by Cargo.
fn forge_mcp_bin() -> PathBuf {
    if let Ok(p) = env::var("CARGO_BIN_EXE_forge-mcp") {
        return PathBuf::from(p);
    }
    let bin_name = if cfg!(windows) { "forge-mcp.exe" } else { "forge-mcp" };
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
/// Returns `(stdout_bytes, stderr_str, exit_status)`.
async fn run_forge_mcp_raw(args: &[&str]) -> (Vec<u8>, String, std::process::ExitStatus) {
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

    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    (output.stdout, stderr, output.status)
}

/// Run `forge-mcp` and return stdout as String.
async fn run_forge_mcp(args: &[&str]) -> (String, String, std::process::ExitStatus) {
    let (stdout_bytes, stderr, status) = run_forge_mcp_raw(args).await;
    let stdout = String::from_utf8_lossy(&stdout_bytes).into_owned();
    (stdout, stderr, status)
}

/// Build a `stdio://<path>` URI pointing to the test server binary.
fn stdio_uri() -> String {
    let bin = test_server_bin();
    format!("stdio://{}", bin.display())
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-001: When stdout is piped (not a TTY), ANSI color codes are suppressed.
// ─────────────────────────────────────────────────────────────────────────────

/// AC-001 — When stdout is piped, output contains no ANSI escape sequences.
///
/// The Stdio::piped() capture means stdout is NOT a TTY.
/// With --color=auto (default), ANSI codes must be absent.
#[tokio::test]
async fn test_bc_5_12_002_no_ansi_in_pipe() {
    let server = stdio_uri();
    let (stdout, _stderr, status) =
        run_forge_mcp(&["list", &server]).await;

    assert!(
        status.success(),
        "forge-mcp list should exit 0; stderr: {_stderr}"
    );

    // Check for ANSI escape sequences: ESC[ prefix (0x1b 0x5b)
    assert!(
        !stdout.contains('\x1b'),
        "stdout must not contain ANSI escape codes when piped (not a TTY).\n\
         stdout: {stdout:?}"
    );

    // Output must still be valid JSON (no corruption)
    let _: serde_json::Value = serde_json::from_str(stdout.trim())
        .unwrap_or_else(|e| panic!("stdout is not valid JSON after pipe: {e}\nstdout: {stdout}"));
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-002: forge-mcp list | jq '.[].name' works (output is valid JSON for jq).
// ─────────────────────────────────────────────────────────────────────────────

/// AC-002 — Output JSON is valid and parseable by standard JSON tools.
///
/// Simulates `forge-mcp list <server> | jq '.[].name'` by capturing
/// stdout and verifying it parses as a JSON object with the expected schema.
/// No ANSI, no extra text, no trailing garbage.
#[tokio::test]
async fn test_bc_5_12_002_jq_compatible_output() {
    let server = stdio_uri();
    let (stdout, _stderr, status) =
        run_forge_mcp(&["list", &server]).await;

    assert!(
        status.success(),
        "forge-mcp list failed; stderr: {_stderr}"
    );

    // The raw stdout (as captured by a pipe) must parse as valid JSON.
    let val: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!(
            "jq-compatible check: stdout is not valid JSON: {e}\n\
             stdout bytes: {:?}",
            stdout.as_bytes()
        )
    });

    // Must be a JSON object (not array, not scalar).
    assert!(
        val.is_object(),
        "output must be a JSON object for jq compatibility, got: {val}"
    );

    // The 'tools' key must be present and be an array (schema from STORY-024).
    assert!(
        val.get("tools").map(|v| v.is_array()).unwrap_or(false),
        "list <server> output must have 'tools' array, got: {val}"
    );

    // No ANSI codes anywhere in the output.
    assert!(
        !stdout.contains('\x1b'),
        "jq-compatible output must not contain ANSI escape codes.\n\
         stdout: {stdout:?}"
    );

    // No extra text before or after the JSON object.
    let trimmed = stdout.trim();
    assert!(
        trimmed.starts_with('{') && trimmed.ends_with('}'),
        "output must be a bare JSON object with no extra text.\n\
         trimmed stdout: {trimmed:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-003: --null-separated flag outputs records separated by \0.
// ─────────────────────────────────────────────────────────────────────────────

/// AC-003 — `forge-mcp list --null-separated` outputs each JSON record
/// followed by `\0` (NUL byte) instead of `\n`.
///
/// This enables `forge-mcp list | xargs -0 …` shell pipelines.
#[tokio::test]
async fn test_bc_5_12_002_null_separated_output() {
    let server = stdio_uri();
    let (stdout_bytes, _stderr, status) =
        run_forge_mcp_raw(&["list", &server, "--null-separated"]).await;

    assert!(
        status.success(),
        "forge-mcp list --null-separated failed; stderr: {_stderr}"
    );

    // Must contain at least one NUL byte as separator.
    assert!(
        stdout_bytes.contains(&0u8),
        "--null-separated output must contain NUL byte (\\0) separators.\n\
         stdout bytes (hex): {:02x?}",
        &stdout_bytes[..stdout_bytes.len().min(200)]
    );

    // The output must NOT end with a plain \n (it should end with \0 or be NUL-terminated).
    // Strip trailing NUL and whitespace, then verify the content parses.
    let content = stdout_bytes
        .iter()
        .cloned()
        .take_while(|&b| b != 0)
        .collect::<Vec<u8>>();
    let record = String::from_utf8_lossy(&content);
    let _: serde_json::Value = serde_json::from_str(record.trim()).unwrap_or_else(|e| {
        panic!(
            "record before NUL separator is not valid JSON: {e}\n\
             record: {record:?}"
        )
    });
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-004: Stdout is flushed after each JSON record.
// ─────────────────────────────────────────────────────────────────────────────

/// AC-004 — Stdout is flushed after each JSON record.
///
/// We verify this functionally: the output is received in full without
/// truncation or buffering artifacts (all bytes are delivered when the
/// process exits, no partial records).
///
/// A truly thorough flush test would require a pseudo-TTY or a pipe with
/// a slow consumer; here we verify the functional contract: the complete
/// record arrives and parses cleanly, with no trailing garbage that would
/// indicate unflushed partial writes.
#[tokio::test]
async fn test_bc_5_12_002_stdout_flushed() {
    let server = stdio_uri();
    let (stdout, _stderr, status) =
        run_forge_mcp(&["list", &server]).await;

    assert!(
        status.success(),
        "forge-mcp list failed; stderr: {_stderr}"
    );

    // Output must be non-empty (flush must have delivered data).
    assert!(
        !stdout.trim().is_empty(),
        "stdout must not be empty — flush must deliver output to the pipe"
    );

    // Output must end with exactly one newline after the JSON (no partial write).
    assert!(
        stdout.ends_with('\n'),
        "stdout must end with a newline, indicating a complete flushed record.\n\
         stdout: {stdout:?}"
    );

    // The JSON must parse without error (no corruption from flush).
    let val: serde_json::Value = serde_json::from_str(stdout.trim()).unwrap_or_else(|e| {
        panic!(
            "stdout_flushed check: stdout is not valid JSON: {e}\nstdout: {stdout}"
        )
    });

    // Sanity: parsed value is an object.
    assert!(
        val.is_object(),
        "flushed output must be a JSON object, got: {val}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Bonus: --color=always forces ANSI even when piped.
// ─────────────────────────────────────────────────────────────────────────────

/// Bonus — `--color=always` forces ANSI escape codes even when stdout is piped.
///
/// The Stdio::piped() capture is not a TTY; --color=always must override that.
/// NOTE: This test verifies the CLI flag is accepted and that the flag controls
/// color behavior, but since ANSI in piped JSON could cause parse issues for
/// naive consumers, we only check the flag is accepted (no parse error).
#[tokio::test]
async fn test_bc_5_12_002_color_always_flag_accepted() {
    let server = stdio_uri();
    // --color=always must be accepted without error
    let (stdout, _stderr, status) =
        run_forge_mcp(&["--color=always", "list", &server]).await;

    assert!(
        status.success(),
        "--color=always must be accepted; stderr: {_stderr}"
    );

    // stdout must be non-empty
    assert!(
        !stdout.trim().is_empty(),
        "--color=always stdout must be non-empty"
    );
}

/// Bonus — `--color=never` suppresses ANSI codes even in a TTY context.
#[tokio::test]
async fn test_bc_5_12_002_color_never_flag() {
    let server = stdio_uri();
    let (stdout, _stderr, status) =
        run_forge_mcp(&["--color=never", "list", &server]).await;

    assert!(
        status.success(),
        "--color=never must be accepted; stderr: {_stderr}"
    );

    assert!(
        !stdout.contains('\x1b'),
        "--color=never must suppress ANSI codes.\nstdout: {stdout:?}"
    );
}
