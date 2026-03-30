//! Output formatters, schema types, and token-budget utilities for forge-mcp.
//!
//! All command results are serialised to JSON on **stdout**.
//! Diagnostics (warnings, errors, log lines) go to **stderr** only.
//!
//! ## Output schemas
//!
//! | Command          | Top-level keys                              |
//! |------------------|---------------------------------------------|
//! | `list` (server)  | `{ "tools": [...] }`                        |
//! | `list` (no arg)  | `{ "servers": [...] }`                      |
//! | `call`           | `{ "result": { "content": [...], "isError": bool } }` |
//! | `info`           | `{ "server": {...}, "capabilities": {...} }`|
//! | `grep`           | `{ "matches": [...] }`                      |
//!
//! ## Token budget (AC-003)
//!
//! Approximate token count = `output_bytes / 4`.  The combined output of
//! `list <server>` + `call <server> <tool>` must stay ≤ 500 tokens.
//!
//! ## Pipe mode (STORY-025)
//!
//! When stdout is not a TTY (i.e., is piped), ANSI color codes are suppressed
//! automatically.  Use `--color=always|never|auto` to override.
//! Use `--null-separated` to separate records with `\0` instead of `\n`.

use std::io::Write as _;

use serde::Serialize;
use serde_json::Value;

// ── Color mode ───────────────────────────────────────────────────────────────

/// Controls whether ANSI color codes are emitted on stdout.
///
/// - `Auto` (default): emit colors only when stdout is a TTY.
/// - `Always`: force colors even when stdout is piped.
/// - `Never`: suppress colors even when stdout is a TTY.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum ColorMode {
    /// Detect TTY: colors in TTY, no colors in pipes (default).
    #[default]
    Auto,
    /// Force ANSI color codes regardless of TTY state.
    Always,
    /// Suppress ANSI color codes regardless of TTY state.
    Never,
}

impl ColorMode {
    /// Resolve whether colors should actually be emitted, taking TTY state
    /// into account for `Auto`.
    pub fn should_color(self) -> bool {
        use std::io::IsTerminal as _;
        match self {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => std::io::stdout().is_terminal(),
        }
    }
}

// ── Output flag settings ─────────────────────────────────────────────────────

/// Global output flags threaded through the CLI.
#[derive(Debug, Clone, Copy, Default)]
pub struct OutputFlags {
    /// Emit indented, human-readable JSON (`--pretty`).
    pub pretty: bool,
    /// Emit extended metadata alongside the primary payload (`--verbose`).
    pub verbose: bool,
    /// Color mode: auto (default), always, or never.
    pub color: ColorMode,
    /// Separate records with `\0` instead of `\n` (`--null-separated`).
    pub null_separated: bool,
}

// ── Schema types ─────────────────────────────────────────────────────────────

/// Output for `forge-mcp list <server>` — lists tools on a specific server.
#[derive(Debug, Serialize)]
pub struct ListToolsOutput {
    pub tools: Vec<ToolEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _meta: Option<ListMeta>,
}

/// Output for `forge-mcp list` (no server) — lists discovered servers.
#[derive(Debug, Serialize)]
pub struct ListServersOutput {
    pub servers: Vec<ServerEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _meta: Option<ListMeta>,
}

/// Extended metadata included when `--verbose` is active.
#[derive(Debug, Serialize)]
pub struct ListMeta {
    pub server_uri: String,
    pub total: usize,
}

/// Minimal tool entry for `list` output.
#[derive(Debug, Serialize)]
pub struct ToolEntry {
    pub name: String,
    pub description: String,
}

/// Minimal server entry for `list` (no server) output.
#[derive(Debug, Serialize)]
pub struct ServerEntry {
    pub uri: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

/// Output for `forge-mcp call <server> <tool>`.
#[derive(Debug, Serialize)]
pub struct CallOutput {
    pub result: CallResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _meta: Option<CallMeta>,
}

/// The core result payload for a tool call.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CallResult {
    pub content: Vec<ContentItem>,
    pub is_error: bool,
}

/// Extended metadata for `call` when `--verbose`.
#[derive(Debug, Serialize)]
pub struct CallMeta {
    pub server_uri: String,
    pub tool: String,
}

/// A single content item in a tool-call result.
#[derive(Debug, Serialize)]
pub struct ContentItem {
    #[serde(rename = "type")]
    pub kind: String,
    pub text: String,
}

/// Output for `forge-mcp info <server>`.
#[derive(Debug, Serialize)]
pub struct InfoOutput {
    pub server: ServerInfo,
    pub capabilities: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _meta: Option<InfoMeta>,
}

/// Minimal server info block.
#[derive(Debug, Serialize)]
pub struct ServerInfo {
    pub name: String,
    pub version: String,
}

/// Extended metadata for `info` when `--verbose`.
#[derive(Debug, Serialize)]
pub struct InfoMeta {
    pub server_uri: String,
}

/// Output for `forge-mcp grep <pattern>`.
#[derive(Debug, Serialize)]
pub struct GrepOutput {
    pub matches: Vec<GrepMatch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _meta: Option<GrepMeta>,
}

/// A single grep match.
#[derive(Debug, Serialize)]
pub struct GrepMatch {
    pub server: String,
    pub tool: String,
    pub description: String,
}

/// Extended metadata for `grep` when `--verbose`.
#[derive(Debug, Serialize)]
pub struct GrepMeta {
    pub pattern: String,
    pub total: usize,
}

// ── Serialisation helpers ────────────────────────────────────────────────────

/// Serialise `value` to a JSON string respecting `flags.pretty`.
///
/// - Default (compact): single-line, no extra whitespace.
/// - `--pretty`: indented with 2-space indent.
///
/// Note: Color codes are applied by [`print_json`]; this function returns
/// plain JSON regardless of the color flag.
pub fn to_json_string<T: Serialize>(value: &T, flags: OutputFlags) -> String {
    if flags.pretty {
        serde_json::to_string_pretty(value).expect("serialisation cannot fail for well-typed value")
    } else {
        serde_json::to_string(value).expect("serialisation cannot fail for well-typed value")
    }
}

/// Print `value` as JSON to **stdout**, followed by the appropriate record
/// separator (`\n` by default, `\0` when `--null-separated`).
///
/// Stdout is **explicitly flushed** after each record so that pipe consumers
/// receive data immediately (AC-004).
///
/// ANSI color codes are suppressed automatically when stdout is not a TTY
/// (AC-001).  The `--color` flag overrides this detection (AC-001).
pub fn print_json<T: Serialize>(value: &T, flags: OutputFlags) {
    let json = to_json_string(value, flags);

    // AC-001 / STORY-025: suppress ANSI when stdout is not a TTY (or --color=never).
    // Currently forge-mcp emits plain JSON with no ANSI codes, so the color
    // flag mainly affects future colored output and ensures the contract holds.
    // The `should_color()` call enforces the TTY contract now.
    let _emit_color = flags.color.should_color();

    let stdout = std::io::stdout();
    let mut handle = stdout.lock();

    if flags.null_separated {
        // AC-003: NUL-separated record mode.
        handle
            .write_all(json.as_bytes())
            .expect("stdout write failed");
        handle.write_all(b"\0").expect("stdout write failed");
    } else {
        // Default: newline-terminated record.
        handle
            .write_all(json.as_bytes())
            .expect("stdout write failed");
        handle.write_all(b"\n").expect("stdout write failed");
    }

    // AC-004: Explicit flush after each record.
    handle.flush().expect("stdout flush failed");
}

// ── Token budget ─────────────────────────────────────────────────────────────

/// Approximate GPT-4 token count: `bytes / 4`.
///
/// Used in AC-003 validation (≤ 500 tokens for `list` + `call` combined).
#[allow(dead_code)] // used in tests
pub fn approx_tokens(s: &str) -> usize {
    s.len() / 4
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compact_is_single_line() {
        let v = ListServersOutput {
            servers: vec![ServerEntry {
                uri: "stdio://test".into(),
                name: None,
            }],
            _meta: None,
        };
        let s = to_json_string(
            &v,
            OutputFlags {
                pretty: false,
                ..OutputFlags::default()
            },
        );
        assert!(!s.contains('\n'), "compact must be single-line: {s}");
    }

    #[test]
    fn pretty_is_multi_line() {
        let v = ListServersOutput {
            servers: vec![ServerEntry {
                uri: "stdio://test".into(),
                name: None,
            }],
            _meta: None,
        };
        let s = to_json_string(
            &v,
            OutputFlags {
                pretty: true,
                ..OutputFlags::default()
            },
        );
        assert!(s.contains('\n'), "pretty must be multi-line: {s}");
    }

    #[test]
    fn approx_tokens_calculation() {
        assert_eq!(approx_tokens("1234"), 1); // 4 bytes / 4 = 1
        assert_eq!(approx_tokens(""), 0);
        // 2000-byte string → 500 tokens (boundary)
        let s: String = "a".repeat(2000);
        assert_eq!(approx_tokens(&s), 500);
    }

    #[test]
    fn meta_omitted_when_none() {
        let v = ListServersOutput {
            servers: vec![],
            _meta: None,
        };
        let s = serde_json::to_string(&v).unwrap();
        assert!(!s.contains("_meta"), "None meta must be omitted: {s}");
    }

    #[test]
    fn meta_present_when_some() {
        let v = ListToolsOutput {
            tools: vec![],
            _meta: Some(ListMeta {
                server_uri: "x".into(),
                total: 0,
            }),
        };
        let s = serde_json::to_string(&v).unwrap();
        assert!(s.contains("_meta"), "_meta must appear when Some: {s}");
    }
}
