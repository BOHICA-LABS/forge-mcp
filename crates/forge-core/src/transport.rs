//! Transport connection functions for forge-core.
//!
//! This module is **effectful** — it spawns child processes and performs I/O.
//! All pure logic lives in `connection.rs`.
//!
//! Supported transports:
//! - `connect_stdio()` — JSON-RPC over child-process stdin/stdout (STORY-007)
//! - `connect_http()` — Streamable HTTP transport (STORY-008)

use std::{collections::HashMap, sync::Arc, time::Duration};

use http::{HeaderName, HeaderValue};
use rmcp::ServiceExt;
use rmcp::model::{ClientCapabilities, ClientInfo, Implementation};
use rmcp::transport::TokioChildProcess;
use rmcp::transport::streamable_http_client::{
    StreamableHttpClientTransport, StreamableHttpClientTransportConfig,
};
use tokio::process::Command;
use tracing::warn;

use crate::connection::{McpConnection, TransportKind};
use crate::error::{CoreError, Result};
use crate::types::NegotiatedCapabilities;

/// Default timeout for the MCP `initialize` handshake.
pub const DEFAULT_CONNECT_TIMEOUT_SECS: u64 = 30;

// ── Forge MCP client identity & capabilities ─────────────────────────────────

/// Build the `ClientInfo` that Forge MCP advertises in every `initialize` request.
///
/// Per AC-004 / BC-2.04.002, Forge MCP advertises:
/// - `sampling: {}` — we can handle `sampling/createMessage` from servers
/// - `elicitation: {}` — we can handle `elicitation/create` from servers
/// - `roots: { listChanged: true }` — we expose filesystem roots and notify on changes
///
/// rmcp's `ClientCapabilities::builder()` pattern is used exclusively (AD-002, DI-004).
fn forge_client_info() -> ClientInfo {
    // Build capabilities via rmcp's builder (AC-005 compliance: no hand-rolled JSON).
    let caps = ClientCapabilities::builder()
        .enable_sampling()
        .enable_elicitation()
        .enable_roots()
        .enable_roots_list_changed()
        .build();

    ClientInfo::new(caps, Implementation::new("forge-mcp", env!("CARGO_PKG_VERSION")))
}

/// Extract `NegotiatedCapabilities` from a freshly-established `RunningService`.
///
/// rmcp stores the `InitializeResult` returned by the server in `Peer::peer_info()`.
/// We read it here to build our Forge-specific wrapper. If `peer_info()` is absent
/// (should never happen after a successful `serve()`) we fall back to empty defaults
/// so the connection is still usable, just with no advertised capabilities.
fn extract_capabilities_from_service(
    service: &rmcp::service::RunningService<rmcp::RoleClient, ClientInfo>,
    advertised_client_caps: &ClientCapabilities,
) -> NegotiatedCapabilities {
    let (server_caps, version) = service
        .peer()
        .peer_info()
        .map(|info| {
            (
                info.capabilities.clone(),
                info.protocol_version.to_string(),
            )
        })
        .unwrap_or_default();

    NegotiatedCapabilities::new(server_caps, advertised_client_caps.clone(), version)
}

// ── Stdio transport ──────────────────────────────────────────────────────────

/// Connect to an MCP server via stdio transport.
///
/// Spawns `command` with `args`, sets environment variables from `env`
/// (merged on top of the calling process's environment), then performs
/// the MCP initialize handshake.
///
/// The client advertises Forge MCP's capabilities (sampling, elicitation,
/// roots/listChanged) per AC-004 / BC-2.04.002.
///
/// # Errors
/// - [`CoreError::ServerNotFound`] — command not found / spawn failed
/// - [`CoreError::Timeout`] — initialize did not complete within `DEFAULT_CONNECT_TIMEOUT_SECS`
/// - [`CoreError::Protocol`] — server sent invalid JSON-RPC
pub async fn connect_stdio(
    command: &str,
    args: &[String],
    env: &HashMap<String, String>,
) -> Result<McpConnection<ClientInfo>> {
    connect_stdio_with_timeout(command, args, env, DEFAULT_CONNECT_TIMEOUT_SECS).await
}

/// Like [`connect_stdio`] but with an explicit timeout in seconds.
pub async fn connect_stdio_with_timeout(
    command: &str,
    args: &[String],
    env: &HashMap<String, String>,
    timeout_secs: u64,
) -> Result<McpConnection<ClientInfo>> {
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
            CoreError::ServerNotFound {
                message: format!("{command}: {msg}"),
            }
        } else {
            CoreError::Io(e)
        }
    })?;

    // ── 3. Build our client identity with Forge's capabilities (AC-004/AC-005)
    let client_info = forge_client_info();
    let advertised_caps = client_info.capabilities.clone();

    // ── 4. Perform the MCP initialize handshake with a timeout ───────────────
    let timeout = Duration::from_secs(timeout_secs);
    let serve_fut = client_info.serve(transport);

    let running = tokio::time::timeout(timeout, serve_fut)
        .await
        .map_err(|_| CoreError::ConnectionTimeout {
            seconds: timeout_secs,
        })?
        .map_err(|e| CoreError::Protocol(e.to_string()))?;

    // ── 5. Extract negotiated capabilities from the completed handshake ───────
    let caps = extract_capabilities_from_service(&running, &advertised_caps);

    // ── 6. Wrap in forge's McpConnection ────────────────────────────────────
    Ok(McpConnection::new(running, command, TransportKind::Stdio, caps))
}

// ── HTTP transport ───────────────────────────────────────────────────────────

/// Establish an MCP connection over the Streamable HTTP transport.
///
/// # Errors
///
/// | Condition | Error |
/// |-----------|-------|
/// | `http://` URL (non-TLS) | warning emitted; connection proceeds |
/// | HTTP 401 from server | `CoreError::AuthenticationFailed` |
/// | HTTP 503 / unreachable | `CoreError::ServerUnavailable` |
/// | DNS failure | `CoreError::DnsResolutionFailed` |
/// | TLS error | `CoreError::TlsError` |
/// | Handshake timeout | `CoreError::Timeout` |
/// | Invalid URL | `CoreError::Protocol` |
/// | Session loss (HTTP 404) | `CoreError::SessionLostFatal` |
pub async fn connect_http(
    url: &str,
    headers: &HashMap<String, String>,
) -> Result<McpConnection<ClientInfo>> {
    // ── 1. URL validation / scheme warning ─────────────────────────────────
    let effective_url = normalise_url(url)?;

    // ── 2. Build custom headers map ────────────────────────────────────────
    let mut custom_headers: HashMap<HeaderName, HeaderValue> = HashMap::new();
    for (name, value) in headers {
        let header_name = HeaderName::from_bytes(name.as_bytes())
            .map_err(|e| CoreError::Protocol(format!("invalid header name {name:?}: {e}")))?;
        let header_value = HeaderValue::from_str(value)
            .map_err(|e| CoreError::Protocol(format!("invalid header value for {name:?}: {e}")))?;
        custom_headers.insert(header_name, header_value);
    }

    // ── 3. Build transport config ───────────────────────────────────────────
    let config = StreamableHttpClientTransportConfig::with_uri(Arc::from(effective_url.as_str()))
        .custom_headers(custom_headers)
        .reinit_on_expired_session(true);

    let transport = StreamableHttpClientTransport::from_config(config);

    // ── 4. Build our client identity with Forge's capabilities (AC-004/AC-005)
    let client_info = forge_client_info();
    let advertised_caps = client_info.capabilities.clone();

    // ── 5. Perform MCP initialize handshake ────────────────────────────────
    let service = client_info.serve(transport).await.map_err(|e| {
        classify_init_error(e, url)
    })?;

    // ── 6. Extract negotiated capabilities ──────────────────────────────────
    let caps = extract_capabilities_from_service(&service, &advertised_caps);

    Ok(McpConnection::new(service, url, TransportKind::Http, caps))
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Normalise the URL and emit `E-CON-010` for insecure `http://` scheme.
fn normalise_url(url: &str) -> Result<String> {
    let trimmed = url.trim();
    let has_scheme = trimmed.contains("://");

    let normalised = if has_scheme {
        trimmed.to_string()
    } else {
        let with_scheme = format!("http://{trimmed}");
        emit_insecure_warning(&with_scheme);
        with_scheme
    };

    if normalised.starts_with("http://") {
        emit_insecure_warning(&normalised);
    }

    let host_part = normalised
        .strip_prefix("https://")
        .or_else(|| normalised.strip_prefix("http://"))
        .unwrap_or(&normalised);

    if host_part.trim_matches('/').is_empty() {
        return Err(CoreError::Protocol(format!("invalid URL: {url:?}")));
    }

    Ok(normalised)
}

/// Emit the `E-CON-010` warning to stderr.
fn emit_insecure_warning(url: &str) {
    let msg = format!("E-CON-010: insecure HTTP scheme for {url} — prefer HTTPS");
    warn!("{}", msg);
    eprintln!("WARNING: {msg}");
}

/// Map an rmcp `ClientInitializeError` to a `CoreError`.
fn classify_init_error(e: rmcp::service::ClientInitializeError, url: &str) -> CoreError {
    let msg = e.to_string();

    if msg.contains("401") || msg.contains("Unauthorized") || msg.contains("unauthorized") {
        return CoreError::AuthenticationFailed { url: url.to_string() };
    }
    if msg.contains("503") || msg.contains("Service Unavailable") {
        return CoreError::ServerUnavailable {
            status: "503".to_string(),
            url: url.to_string(),
        };
    }
    if msg.contains("connection refused")
        || msg.contains("Connection refused")
        || msg.contains("error sending request")
        || msg.contains("os error 61")
        || msg.contains("os error 111")
        || msg.contains("tcp connect error")
    {
        return CoreError::ServerUnavailable {
            status: "connection refused".to_string(),
            url: url.to_string(),
        };
    }
    if msg.contains("dns") || msg.contains("DNS") || msg.contains("resolve")
        || msg.contains("No such host")
    {
        return CoreError::DnsResolutionFailed {
            url: url.to_string(),
            cause: msg,
        };
    }
    if msg.contains("tls") || msg.contains("TLS") || msg.contains("certificate")
        || msg.contains("Certificate")
    {
        return CoreError::TlsError {
            url: url.to_string(),
            cause: msg,
        };
    }
    if msg.contains("timeout") || msg.contains("Timeout") || msg.contains("timed out") {
        return CoreError::Timeout {
            seconds: 0,
            url: url.to_string(),
        };
    }
    if msg.contains("404") || msg.contains("Session expired") {
        return CoreError::SessionLostFatal {
            url: url.to_string(),
            cause: msg,
        };
    }

    CoreError::Rmcp(format!("{url}: {msg}"))
}

// ── Env var expansion (stdio) ─────────────────────────────────────────────────

/// Expand `${VAR}` and `$VAR` patterns in `value` using the calling process's
/// environment. Unset variables are left as-is.
fn expand_env_value(value: &str) -> String {
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
                i += 1;
                let start = i;
                while i < chars.len() && chars[i] != '}' {
                    i += 1;
                }
                let var_name: String = chars[start..i].iter().collect();
                if i < chars.len() {
                    i += 1;
                }
                match std::env::var(&var_name) {
                    Ok(v) => result.push_str(&v),
                    Err(_) => result.push_str(&format!("${{{var_name}}}")),
                }
            } else if chars[i].is_alphanumeric() || chars[i] == '_' {
                let start = i;
                while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let var_name: String = chars[start..i].iter().collect();
                match std::env::var(&var_name) {
                    Ok(v) => result.push_str(&v),
                    Err(_) => result.push_str(&format!("${var_name}")),
                }
            } else {
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

// ── Unit tests for env-var expansion ────────────────────────────────────────

#[cfg(test)]
mod expansion_tests {
    use super::*;

    #[test]
    fn test_expand_no_vars() {
        assert_eq!(expand_env_value("hello world"), "hello world");
    }

    #[test]
    fn test_expand_braced_var() {
        unsafe { std::env::set_var("_FORGE_TEST_VAR", "expanded_value") };
        assert_eq!(
            expand_env_value("prefix_${_FORGE_TEST_VAR}_suffix"),
            "prefix_expanded_value_suffix"
        );
    }

    #[test]
    fn test_expand_unbraced_var() {
        unsafe { std::env::set_var("_FORGE_TEST_VAR2", "value2") };
        assert_eq!(expand_env_value("$_FORGE_TEST_VAR2/rest"), "value2/rest");
    }

    #[test]
    fn test_expand_missing_var_leaves_placeholder() {
        let result = expand_env_value("${_FORGE_UNSET_XYZ_12345}");
        assert_eq!(result, "${_FORGE_UNSET_XYZ_12345}");
    }

    #[test]
    fn test_expand_bare_dollar() {
        assert_eq!(expand_env_value("cost $5.00"), "cost $5.00");
    }
}
