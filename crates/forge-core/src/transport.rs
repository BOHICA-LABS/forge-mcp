//! Transport connection functions for forge-core.
//!
//! This module is **effectful** — it spawns child processes and performs I/O.
//! All pure logic lives in `connection.rs`.
//!
//! # Design
//! - Uses `rmcp`'s `TokioChildProcess` transport exclusively (AD-002 / NFR-014).
//!   No raw JSON-RPC framing in this crate.
//! - Env-var expansion is performed at call time against the calling process's
//!   environment; values from `env` override the base environment.

use std::collections::HashMap;
use std::time::Duration;

use rmcp::ServiceExt;
use rmcp::transport::TokioChildProcess;
use tokio::process::Command;

use crate::connection::{McpConnection, TransportKind};
use crate::error::{ForgeError, Result};

/// Default timeout for the MCP `initialize` handshake.
pub const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 30;

/// Connect to an MCP server via stdio transport.
///
/// Spawns `command` with `args`, sets environment variables from `env`
/// (merged on top of the calling process's environment), then performs
/// the MCP initialize handshake.
///
/// Returns a [`McpConnection`] that owns the child-process handle.
/// When the `McpConnection` is dropped or shut down, the child process is killed.
///
/// # Errors
/// - [`ForgeError::ServerNotFound`] — command not found / spawn failed
/// - [`ForgeError::ConnectionTimeout`] — initialize did not complete within `timeout_secs`
/// - [`ForgeError::ProtocolError`] — server sent invalid JSON-RPC
pub async fn connect_stdio(
    command: &str,
    args: &[String],
    env: &HashMap<String, String>,
) -> Result<McpConnection> {
    connect_stdio_with_timeout(command, args, env, DEFAULT_CONNECT_TIMEOUT_SECS).await
}

/// Like [`connect_stdio`] but with an explicit timeout in seconds.
pub async fn connect_stdio_with_timeout(
    command: &str,
    args: &[String],
    env: &HashMap<String, String>,
    timeout_secs: u64,
) -> Result<McpConnection> {
    // ── 1. Build the command ─────────────────────────────────────────────────
    let mut cmd = Command::new(command);
    cmd.args(args);

    // Expand env vars: start from the current process environment, then apply
    // any overrides from the caller (AC-002 env var expansion).
    for (key, value) in env {
        let expanded = expand_env_value(value);
        cmd.env(key, expanded);
    }

    // ── 2. Spawn the child process via rmcp's TokioChildProcess ─────────────
    let transport = TokioChildProcess::new(cmd).map_err(|e| {
        let msg = e.to_string();
        if msg.contains("No such file or directory")
            || msg.contains("not found")
            || msg.contains("os error 2")
        {
            ForgeError::ServerNotFound {
                message: format!("{command}: {msg}"),
            }
        } else {
            ForgeError::Io(e)
        }
    })?;

    // ── 3. Perform the MCP initialize handshake with a timeout ───────────────
    let timeout = Duration::from_secs(timeout_secs);
    let serve_fut = ().serve(transport);

    let running = tokio::time::timeout(timeout, serve_fut)
        .await
        .map_err(|_| ForgeError::ConnectionTimeout {
            seconds: timeout_secs,
        })?
        .map_err(|e| ForgeError::ProtocolError {
            message: e.to_string(),
        })?;

    // ── 4. Wrap in forge's McpConnection ────────────────────────────────────
    Ok(McpConnection::from_running_service(running, TransportKind::Stdio))
}

// ── Env var expansion ────────────────────────────────────────────────────────

/// Expand `${VAR}` and `$VAR` patterns in `value` using the calling process's
/// environment.  Unset variables are left as-is (empty string substitution
/// would silently break credentials).
fn expand_env_value(value: &str) -> String {
    // Use shellexpand-lite approach: process character-by-character.
    // We support two syntaxes: `${VAR}` and `$VAR`.
    let mut result = String::with_capacity(value.len());
    let chars: Vec<char> = value.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '$' {
            i += 1;
            if i >= chars.len() {
                result.push('$');
                break;
            }

            if chars[i] == '{' {
                // ${VAR} syntax
                i += 1;
                let start = i;
                while i < chars.len() && chars[i] != '}' {
                    i += 1;
                }
                let var_name: String = chars[start..i].iter().collect();
                if i < chars.len() {
                    i += 1; // consume '}'
                }
                match std::env::var(&var_name) {
                    Ok(v) => result.push_str(&v),
                    Err(_) => {
                        // Leave unexpanded — do not silently zero out credentials.
                        result.push_str(&format!("${{{var_name}}}"));
                    }
                }
            } else if chars[i].is_alphanumeric() || chars[i] == '_' {
                // $VAR syntax
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let var_name: String = chars[start..i].iter().collect();
                match std::env::var(&var_name) {
                    Ok(v) => result.push_str(&v),
                    Err(_) => {
                        result.push_str(&format!("${var_name}"));
                    }
                }
            } else {
                // Bare '$' not followed by a valid var — pass through.
                result.push('$');
                result.push(chars[i]);
                i += 1;
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    result
}

// ── Pure unit tests for env-var expansion ────────────────────────────────────

#[cfg(test)]
mod expansion_tests {
    use super::*;

    #[test]
    fn test_expand_no_vars() {
        assert_eq!(expand_env_value("hello world"), "hello world");
    }

    #[test]
    fn test_expand_braced_var() {
        // SAFETY: single-threaded unit test with a unique key.
        unsafe { std::env::set_var("_FORGE_TEST_VAR", "expanded_value") };
        assert_eq!(
            expand_env_value("prefix_${_FORGE_TEST_VAR}_suffix"),
            "prefix_expanded_value_suffix"
        );
    }

    #[test]
    fn test_expand_unbraced_var() {
        // SAFETY: single-threaded unit test with a unique key.
        unsafe { std::env::set_var("_FORGE_TEST_VAR2", "value2") };
        assert_eq!(expand_env_value("$_FORGE_TEST_VAR2/rest"), "value2/rest");
    }

    #[test]
    fn test_expand_missing_var_leaves_placeholder() {
        // An unset variable must NOT be silently zeroed — leave it as-is.
        let result = expand_env_value("${_FORGE_UNSET_XYZ_12345}");
        assert_eq!(result, "${_FORGE_UNSET_XYZ_12345}");
    }

    #[test]
    fn test_expand_bare_dollar() {
        assert_eq!(expand_env_value("cost $5.00"), "cost $5.00");
    }
}
