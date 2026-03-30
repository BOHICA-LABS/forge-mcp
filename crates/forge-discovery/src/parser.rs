//! Pure config file parser for MCP server definitions.
//!
//! This module is **pure** — it receives a `DiscoveredConfig` (which already
//! holds the file content as a `String`) and produces `Vec<ServerEntry>`.
//! No filesystem I/O occurs here.
//!
//! ## Supported schemas
//!
//! | Schema key  | Editors                              | Transport detection |
//! |-------------|--------------------------------------|---------------------|
//! | `mcpServers`| Claude Desktop, Cursor, Windsurf     | Implicit: `command` → Stdio, `url` → Http |
//! | `servers`   | VS Code                              | Explicit `"type"` field: `"stdio"` or `"sse"`/`"http"` |
//!
//! Auto-detection tries `mcpServers` first, then `servers`.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::Deserialize;
use serde_json::Value;

use forge_core::types::{
    EditorKind, HttpConfig, ServerEntry, StdioConfig, Transport, TransportConfig,
};

use crate::error::ConfigError;
use crate::types::EditorKind as DiscoveryEditorKind;

// ── Raw serde shapes ──────────────────────────────────────────────────────────

/// Raw shape of a server entry in the `mcpServers` schema.
/// Unknown fields are silently ignored (no `deny_unknown_fields`).
#[derive(Debug, Deserialize)]
struct RawMcpServer {
    command: Option<String>,
    #[serde(default)]
    args: Vec<String>,
    /// VS Code `"type"` field when file accidentally uses `mcpServers` — ignored here.
    #[serde(rename = "type")]
    _type: Option<String>,
    url: Option<String>,
    #[serde(rename = "serverUrl")]
    server_url: Option<String>,
    #[serde(default)]
    headers: HashMap<String, Value>,
    #[serde(default)]
    env: HashMap<String, Value>,
    #[serde(default)]
    disabled: bool,
    #[serde(rename = "alwaysAllow", default)]
    always_allow: Vec<String>,
}

/// Raw shape of a server entry in the VS Code `servers` schema.
#[derive(Debug, Deserialize)]
struct RawVsCodeServer {
    #[serde(rename = "type")]
    server_type: Option<String>,
    command: Option<String>,
    #[serde(default)]
    args: Vec<String>,
    url: Option<String>,
    #[serde(default)]
    headers: HashMap<String, Value>,
    #[serde(default)]
    env: HashMap<String, Value>,
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Convert a discovery-layer `EditorKind` to the forge-core `EditorKind`.
fn convert_editor(editor: &DiscoveryEditorKind) -> EditorKind {
    match editor {
        DiscoveryEditorKind::ClaudeDesktop => EditorKind::ClaudeDesktop,
        DiscoveryEditorKind::Cursor => EditorKind::Cursor,
        DiscoveryEditorKind::VSCode => EditorKind::VSCode,
        DiscoveryEditorKind::Windsurf => EditorKind::Windsurf,
    }
}

/// Extract a `HashMap<String, String>` from a `HashMap<String, Value>`,
/// returning `Err(ConfigError::EnvValueNotString)` on the first non-string value.
fn extract_env(
    raw_env: HashMap<String, Value>,
    path: &PathBuf,
    server_name: &str,
) -> Result<HashMap<String, String>, ConfigError> {
    let mut out = HashMap::with_capacity(raw_env.len());
    for (key, val) in raw_env {
        match val {
            Value::String(s) => {
                out.insert(key, s);
            }
            _ => {
                return Err(ConfigError::EnvValueNotString {
                    path: path.clone(),
                    server: server_name.to_owned(),
                    key,
                });
            }
        }
    }
    Ok(out)
}

/// Extract string-valued headers; non-string header values are silently skipped
/// (headers are an HTTP concern; we don't define E-CFG-NNN for headers).
fn extract_headers(raw: HashMap<String, Value>) -> HashMap<String, String> {
    raw.into_iter()
        .filter_map(|(k, v)| v.as_str().map(|s| (k, s.to_owned())))
        .collect()
}

// ── Public API ────────────────────────────────────────────────────────────────

/// Parse the content of a discovered MCP config file into a list of server entries.
///
/// # Arguments
/// * `discovered` — a `DiscoveredConfig` whose `content` field holds the raw JSON
///   string of the config file.  The `path` and `editor` fields provide source
///   attribution.
///
/// # Return value
/// * `Ok(vec)` — successfully parsed server entries (may be empty).
/// * `Err(ConfigError::JsonParseError)` — the file is not valid JSON.
/// * `Err(ConfigError::NoRecognizedSchema)` — valid JSON but neither
///   `mcpServers` nor `servers` key found.
/// * `Err(ConfigError::EnvValueNotString)` — an `env` value was not a string.
///   The error carries the first offending entry; other entries may or may not
///   have been processed.
///
/// # Purity
/// This function performs no I/O.  It is pure: same input → same output.
pub fn parse_config(
    discovered: &ParseInput,
) -> Result<Vec<ServerEntry>, ConfigError> {
    let root: Value =
        serde_json::from_str(&discovered.content).map_err(|e| {
            ConfigError::JsonParseError {
                path: discovered.path.clone(),
                source: e,
            }
        })?;

    // Auto-detect schema: try `mcpServers` first, then `servers`.
    if let Some(mcp_servers) = root.get("mcpServers") {
        parse_mcp_servers_schema(mcp_servers, discovered)
    } else if let Some(servers) = root.get("servers") {
        parse_servers_schema(servers, discovered)
    } else {
        // AC-008: valid JSON but no recognised key → E-CFG-004, empty list.
        // The caller (discovery orchestrator) may log the warning and continue.
        Err(ConfigError::NoRecognizedSchema {
            path: discovered.path.clone(),
        })
    }
}

// ── Schema parsers ────────────────────────────────────────────────────────────

fn parse_mcp_servers_schema(
    servers_value: &Value,
    input: &ParseInput,
) -> Result<Vec<ServerEntry>, ConfigError> {
    let obj = match servers_value.as_object() {
        Some(o) => o,
        None => return Ok(vec![]),
    };

    let editor_core = convert_editor(&input.editor);
    let mut entries = Vec::with_capacity(obj.len());

    for (name, raw_val) in obj {
        let raw: RawMcpServer =
            serde_json::from_value(raw_val.clone()).map_err(|e| {
                ConfigError::JsonParseError {
                    path: input.path.clone(),
                    source: e,
                }
            })?;

        // AC-006: env values stored as-is (not expanded). Non-string → E-CFG-007.
        let env = extract_env(raw.env, &input.path, name)?;

        // EC-001: `mcpServers` schema — prefer `command` (Stdio).
        // Fall back to `url` or `serverUrl` for Http.
        let (transport, config) = if let Some(cmd) = raw.command {
            (
                Transport::Stdio,
                TransportConfig::Stdio(StdioConfig {
                    command: cmd,
                    args: raw.args,
                    env,
                }),
            )
        } else {
            let url = raw
                .url
                .or(raw.server_url)
                .unwrap_or_default();
            let headers = extract_headers(raw.headers);
            (
                Transport::Http,
                TransportConfig::Http(HttpConfig { url, headers }),
            )
        };

        // AC-004: `disabled` maps to `enabled = !disabled`.
        let enabled = !raw.disabled;

        entries.push(ServerEntry {
            name: name.clone(),
            transport,
            config,
            source_editor: editor_core.clone(),
            source_path: input.path.clone(),
            enabled,
            always_allow: raw.always_allow,
        });
    }

    Ok(entries)
}

fn parse_servers_schema(
    servers_value: &Value,
    input: &ParseInput,
) -> Result<Vec<ServerEntry>, ConfigError> {
    let obj = match servers_value.as_object() {
        Some(o) => o,
        None => return Ok(vec![]),
    };

    let editor_core = convert_editor(&input.editor);
    let mut entries = Vec::with_capacity(obj.len());

    for (name, raw_val) in obj {
        let raw: RawVsCodeServer =
            serde_json::from_value(raw_val.clone()).map_err(|e| {
                ConfigError::JsonParseError {
                    path: input.path.clone(),
                    source: e,
                }
            })?;

        let env = extract_env(raw.env, &input.path, name)?;

        // AC-002: VS Code uses explicit `"type"` field.
        // `"stdio"` → Stdio, `"sse"` or `"http"` → Http (EC-001 + domain research).
        let server_type = raw
            .server_type
            .as_deref()
            .unwrap_or("stdio")
            .to_ascii_lowercase();

        let (transport, config) = match server_type.as_str() {
            "stdio" => {
                let cmd = raw.command.unwrap_or_default();
                (
                    Transport::Stdio,
                    TransportConfig::Stdio(StdioConfig {
                        command: cmd,
                        args: raw.args,
                        env,
                    }),
                )
            }
            "sse" | "http" => {
                let url = raw.url.unwrap_or_default();
                let headers = extract_headers(raw.headers);
                (
                    Transport::Http,
                    TransportConfig::Http(HttpConfig { url, headers }),
                )
            }
            // Unknown type: fall back to treating as Stdio (forward-compatible).
            _ => {
                let cmd = raw.command.unwrap_or_default();
                (
                    Transport::Stdio,
                    TransportConfig::Stdio(StdioConfig {
                        command: cmd,
                        args: raw.args,
                        env,
                    }),
                )
            }
        };

        // VS Code schema has no `disabled` / `alwaysAllow` fields.
        entries.push(ServerEntry {
            name: name.clone(),
            transport,
            config,
            source_editor: editor_core.clone(),
            source_path: input.path.clone(),
            enabled: true,
            always_allow: vec![],
        });
    }

    Ok(entries)
}

// ── ParseInput ────────────────────────────────────────────────────────────────

/// Input to `parse_config` — carries file content + source attribution.
///
/// This is a thin wrapper so `parse_config` remains pure (no `&Path` / no I/O).
/// In the discovery pipeline the caller reads the file and populates this struct.
#[derive(Debug, Clone)]
pub struct ParseInput {
    /// Raw JSON content of the config file.
    pub content: String,
    /// Absolute path to the config file (for error messages and attribution).
    pub path: PathBuf,
    /// Which editor this config file belongs to.
    pub editor: DiscoveryEditorKind,
}

impl ParseInput {
    /// Convenience constructor.
    pub fn new(
        content: impl Into<String>,
        path: impl Into<PathBuf>,
        editor: DiscoveryEditorKind,
    ) -> Self {
        Self {
            content: content.into(),
            path: path.into(),
            editor,
        }
    }
}
