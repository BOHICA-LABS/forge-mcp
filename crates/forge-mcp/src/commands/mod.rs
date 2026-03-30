//! Subcommand definitions and dispatch for forge-mcp.
//!
//! Each subcommand is declared as a clap-derive struct/enum. Handler functions
//! connect to MCP servers using forge-core and emit structured JSON via the
//! output module (STORY-024).
//!
//! **Performance constraint (NFR-001):** No subsystem crates are initialised
//! at argument-parse time. All heavy initialisation (TUI, daemon socket,
//! connection pool) happens inside the handler functions, which are only
//! called after successful parse.

use std::collections::HashMap;

use clap::Subcommand;
use serde_json::Value;

use crate::exit_codes::CliError;
use crate::output::{
    CallMeta, CallOutput, CallResult, ContentItem, GrepMatch, GrepMeta, GrepOutput, InfoMeta,
    InfoOutput, ListMeta, ListServersOutput, ListToolsOutput, OutputFlags,
    ServerEntry as OutputServerEntry, ServerInfo, ToolEntry, print_json,
};

/// All top-level subcommands supported by `forge-mcp`.
#[derive(Subcommand, Debug)]
#[non_exhaustive]
pub enum Commands {
    /// List tools, resources, and prompts exposed by an MCP server.
    List(ListArgs),

    /// Call a specific tool on an MCP server.
    Call(CallArgs),

    /// Show detailed info (capabilities, metadata) about an MCP server.
    Info(InfoArgs),

    /// Search tool names and descriptions with a regex pattern.
    Grep(GrepArgs),

    /// Run the MCP conformance test suite against a server.
    Test(TestArgs),

    /// Run a security audit against an MCP server.
    Audit(AuditArgs),

    /// Launch the interactive terminal UI.
    Tui(TuiArgs),

    /// Manage the background daemon.
    Daemon(DaemonArgs),
}

// ── Per-subcommand argument structs ──────────────────────────────────────────

/// Arguments for `forge-mcp list [server]`.
#[derive(clap::Args, Debug)]
pub struct ListArgs {
    /// Optional MCP server URI to list. If omitted, lists all discovered servers.
    pub server: Option<String>,

    /// Output results as JSON.
    #[arg(long)]
    pub json: bool,

    /// Separate each JSON record with a NUL byte (`\0`) instead of a newline.
    ///
    /// Enables `forge-mcp list | xargs -0 …` shell pipelines (AC-003).
    #[arg(long)]
    pub null_separated: bool,
}

/// Arguments for `forge-mcp call <server> <tool> [args]`.
#[derive(clap::Args, Debug)]
pub struct CallArgs {
    /// MCP server URI.
    pub server: String,

    /// Tool name to call.
    pub tool: String,

    /// Optional JSON-encoded arguments to pass to the tool.
    pub args: Option<String>,
}

/// Arguments for `forge-mcp info <server>`.
#[derive(clap::Args, Debug)]
pub struct InfoArgs {
    /// MCP server URI to inspect.
    pub server: String,

    /// Output results as JSON.
    #[arg(long)]
    pub json: bool,
}

/// Arguments for `forge-mcp grep <pattern>`.
#[derive(clap::Args, Debug)]
pub struct GrepArgs {
    /// Regex pattern to search for in tool names and descriptions.
    pub pattern: String,

    /// Optional MCP server URI to restrict the search.
    pub server: Option<String>,
}

/// Arguments for `forge-mcp test <server>`.
#[derive(clap::Args, Debug)]
pub struct TestArgs {
    /// MCP server URI to test.
    pub server: String,

    /// Output results as JSON.
    #[arg(long)]
    pub json: bool,
}

/// Arguments for `forge-mcp audit <server>`.
#[derive(clap::Args, Debug)]
pub struct AuditArgs {
    /// MCP server URI to audit.
    pub server: String,

    /// Output results as JSON.
    #[arg(long)]
    pub json: bool,
}

/// Arguments for `forge-mcp tui`.
#[derive(clap::Args, Debug)]
pub struct TuiArgs {
    // No arguments yet — future stories may add filters.
}

/// Action for the `daemon` subcommand.
#[derive(clap::Subcommand, Debug)]
pub enum DaemonAction {
    /// Start the background daemon.
    Start,
    /// Stop the background daemon.
    Stop,
    /// Restart the background daemon.
    Restart,
    /// List active daemon sessions.
    Sessions,
}

/// Arguments for `forge-mcp daemon <action>`.
#[derive(clap::Args, Debug)]
pub struct DaemonArgs {
    #[command(subcommand)]
    pub action: DaemonAction,
}

// ── URI parsing ──────────────────────────────────────────────────────────────

/// Parse a `stdio://<path>` URI into `(command, args)` for `connect_stdio`.
///
/// Returns `Err` if the URI is not a valid stdio URI.
fn parse_stdio_uri(uri: &str) -> Result<(String, Vec<String>), CliError> {
    let path = uri
        .strip_prefix("stdio://")
        .ok_or_else(|| CliError::Config(format!("unsupported URI scheme: {uri}")))?;
    // The path may include space-separated args after the binary path.
    let mut parts = path.splitn(2, ' ');
    let cmd = parts.next().unwrap_or(path).to_string();
    let args: Vec<String> = parts
        .next()
        .map(|rest| rest.split(' ').map(str::to_string).collect())
        .unwrap_or_default();
    Ok((cmd, args))
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// `forge-mcp list [server]`
///
/// - With a server URI → connect and list tools as `{ "tools": [...] }`.
/// - Without a server URI → list discovered servers as `{ "servers": [...] }`.
pub async fn handle_list(args: &ListArgs, flags: OutputFlags) -> Result<(), CliError> {
    // AC-003 (STORY-025): merge per-subcommand null_separated into flags.
    let flags = OutputFlags {
        null_separated: args.null_separated,
        ..flags
    };

    match &args.server {
        Some(uri) => {
            // Connect and list tools.
            let (cmd, env_args) = parse_stdio_uri(uri)?;
            let conn =
                forge_core::connect_stdio(&cmd, &env_args, &HashMap::new())
                    .await
                    .map_err(CliError::Connection)?;

            let tools = forge_core::list_tools(&conn)
                .await
                .map_err(CliError::Connection)?;

            let tool_entries: Vec<ToolEntry> = tools
                .iter()
                .map(|t| ToolEntry {
                    name: t.name.to_string(),
                    description: t
                        .description
                        .as_deref()
                        .unwrap_or("")
                        .to_string(),
                })
                .collect();

            let meta = if flags.verbose {
                Some(ListMeta {
                    server_uri: uri.clone(),
                    total: tool_entries.len(),
                })
            } else {
                None
            };

            print_json(
                &ListToolsOutput {
                    tools: tool_entries,
                    _meta: meta,
                },
                flags,
            );
        }
        None => {
            // List discovered servers (from local config files).
            let server_entries = discover_all_servers();

            let meta = if flags.verbose {
                Some(ListMeta {
                    server_uri: String::new(),
                    total: server_entries.len(),
                })
            } else {
                None
            };

            print_json(
                &ListServersOutput {
                    servers: server_entries,
                    _meta: meta,
                },
                flags,
            );
        }
    }
    Ok(())
}

/// Discover all locally configured MCP servers and return them as output entries.
///
/// Reads config files from Claude Desktop, Cursor, VS Code, and Windsurf.
/// Silently skips unreadable or missing files.
fn discover_all_servers() -> Vec<OutputServerEntry> {
    use forge_core::types::TransportConfig;
    use forge_discovery::{Os, ParseInput, discover_configs, parse_config};

    let discovered = match discover_configs(Os::current(), None, None) {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!("discovery failed: {e}");
            return vec![];
        }
    };

    let mut entries: Vec<OutputServerEntry> = Vec::new();

    for cfg in &discovered {
        if !cfg.exists {
            continue;
        }
        if cfg.access_error.is_some() {
            tracing::warn!("skipping {}: {}", cfg.path.display(), cfg.access_error.as_deref().unwrap_or(""));
            continue;
        }
        let content = match std::fs::read_to_string(&cfg.path) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("cannot read {}: {e}", cfg.path.display());
                continue;
            }
        };
        let input = ParseInput::new(content, &cfg.path, cfg.editor.clone());
        match parse_config(&input) {
            Ok(servers) => {
                for s in servers {
                    let uri = match &s.config {
                        TransportConfig::Stdio(stdio) => {
                            format!("stdio://{}", stdio.command)
                        }
                        TransportConfig::Http(http) => http.url.clone(),
                        // non_exhaustive: forward-compatible
                        _ => format!("unknown://{}", s.name),
                    };
                    entries.push(OutputServerEntry {
                        uri,
                        name: Some(s.name.clone()),
                    });
                }
            }
            Err(e) => {
                tracing::debug!("parse_config failed for {}: {e}", cfg.path.display());
            }
        }
    }

    entries
}

/// `forge-mcp call <server> <tool> [args]`
///
/// Calls a tool and emits `{ "result": { "content": [...], "isError": bool } }`.
pub async fn handle_call(args: &CallArgs, flags: OutputFlags) -> Result<(), CliError> {
    let (cmd, env_args) = parse_stdio_uri(&args.server)?;
    let conn = forge_core::connect_stdio(&cmd, &env_args, &HashMap::new())
        .await
        .map_err(CliError::Connection)?;

    // Parse optional JSON arguments.
    let tool_args: Value = match &args.args {
        Some(json_str) => serde_json::from_str(json_str)
            .map_err(|e| CliError::Config(format!("invalid tool args JSON: {e}")))?,
        None => Value::Null,
    };

    let result = forge_core::call_tool(&conn, &args.tool, tool_args, None)
        .await
        .map_err(CliError::Connection)?;

    // Convert content items to our minimal ContentItem.
    // We serialise via serde_json to avoid a direct rmcp dependency in forge-mcp.
    let content: Vec<ContentItem> = result
        .content
        .iter()
        .map(|c| {
            // Serialise to Value and extract "type" + "text" fields.
            let v = serde_json::to_value(c).unwrap_or(Value::Null);
            let kind = v
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("unknown")
                .to_string();
            let text = v
                .get("text")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();
            ContentItem { kind, text }
        })
        .collect();

    let meta = if flags.verbose {
        Some(CallMeta {
            server_uri: args.server.clone(),
            tool: args.tool.clone(),
        })
    } else {
        None
    };

    print_json(
        &CallOutput {
            result: CallResult {
                content,
                is_error: result.is_error,
            },
            _meta: meta,
        },
        flags,
    );

    Ok(())
}

/// `forge-mcp info <server>`
///
/// Emits `{ "server": {...}, "capabilities": {...} }`.
pub async fn handle_info(args: &InfoArgs, flags: OutputFlags) -> Result<(), CliError> {
    let (cmd, env_args) = parse_stdio_uri(&args.server)?;
    let conn = forge_core::connect_stdio(&cmd, &env_args, &HashMap::new())
        .await
        .map_err(CliError::Connection)?;

    let server_name = conn.server_name().to_string();
    let server_version = conn.server_version().to_string();
    let capabilities = serde_json::to_value(conn.server_capabilities())
        .unwrap_or(Value::Object(serde_json::Map::new()));

    let meta = if flags.verbose {
        Some(InfoMeta {
            server_uri: args.server.clone(),
        })
    } else {
        None
    };

    print_json(
        &InfoOutput {
            server: ServerInfo {
                name: server_name,
                version: server_version,
            },
            capabilities,
            _meta: meta,
        },
        flags,
    );

    Ok(())
}

/// `forge-mcp grep <pattern> [server]`
///
/// Searches tool names and descriptions.  If a server URI is given, searches
/// only that server.  Otherwise searches all discovered servers.
pub async fn handle_grep(args: &GrepArgs, flags: OutputFlags) -> Result<(), CliError> {
    let pattern_lower = args.pattern.to_lowercase();

    let servers: Vec<String> = match &args.server {
        Some(uri) => vec![uri.clone()],
        None => {
            // Discover local servers; only include stdio:// URIs with absolute paths
            // (skips npx/docker/uvx launchers that would require network/docker).
            discover_all_servers()
                .into_iter()
                .filter_map(|e| {
                    // Only include stdio:// URIs that look like an absolute path
                    if let Some(path) = e.uri.strip_prefix("stdio://")
                        && path.starts_with('/') && std::path::Path::new(path).exists() {
                            return Some(e.uri);
                    }
                    None
                })
                .collect()
        }
    };

    let mut matches: Vec<GrepMatch> = Vec::new();

    for server_uri in &servers {
        let (cmd, env_args) = match parse_stdio_uri(server_uri) {
            Ok(v) => v,
            Err(e) => {
                tracing::warn!("skipping {server_uri}: {e}");
                continue;
            }
        };
        // Use a short timeout for grep so we don't hang on unreachable servers.
        let conn = match forge_core::connect_stdio_with_timeout(&cmd, &env_args, &HashMap::new(), 5).await {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("could not connect to {server_uri}: {e}");
                continue;
            }
        };
        let tools = match forge_core::list_tools(&conn).await {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!("list_tools failed for {server_uri}: {e}");
                continue;
            }
        };
        for tool in &tools {
            let name_lower = tool.name.to_lowercase();
            let desc_lower = tool
                .description
                .as_deref()
                .unwrap_or("")
                .to_lowercase();
            if name_lower.contains(&pattern_lower) || desc_lower.contains(&pattern_lower) {
                matches.push(GrepMatch {
                    server: server_uri.clone(),
                    tool: tool.name.to_string(),
                    description: tool
                        .description
                        .as_deref()
                        .unwrap_or("")
                        .to_string(),
                });
            }
        }
    }

    let meta = if flags.verbose {
        Some(GrepMeta {
            pattern: args.pattern.clone(),
            total: matches.len(),
        })
    } else {
        None
    };

    print_json(&GrepOutput { matches, _meta: meta }, flags);
    Ok(())
}

pub async fn handle_test(_args: &TestArgs, _flags: OutputFlags) -> Result<(), CliError> {
    eprintln!("forge-mcp test — not yet implemented (STORY-026)");
    Ok(())
}

pub async fn handle_audit(_args: &AuditArgs, _flags: OutputFlags) -> Result<(), CliError> {
    eprintln!("forge-mcp audit — not yet implemented (STORY-026)");
    Ok(())
}

pub async fn handle_tui(_args: &TuiArgs, _flags: OutputFlags) -> Result<(), CliError> {
    eprintln!("forge-mcp tui — not yet implemented (STORY-027)");
    Ok(())
}

pub async fn handle_daemon(_args: &DaemonArgs, _flags: OutputFlags) -> Result<(), CliError> {
    eprintln!("forge-mcp daemon — not yet implemented (STORY-010)");
    Ok(())
}

/// Dispatch a parsed [`Commands`] variant to its handler.
pub async fn dispatch(cmd: &Commands, flags: OutputFlags) -> Result<(), CliError> {
    match cmd {
        Commands::List(args) => handle_list(args, flags).await,
        Commands::Call(args) => handle_call(args, flags).await,
        Commands::Info(args) => handle_info(args, flags).await,
        Commands::Grep(args) => handle_grep(args, flags).await,
        Commands::Test(args) => handle_test(args, flags).await,
        Commands::Audit(args) => handle_audit(args, flags).await,
        Commands::Tui(args) => handle_tui(args, flags).await,
        Commands::Daemon(args) => handle_daemon(args, flags).await,
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    /// Top-level parser used only in tests to exercise dispatch routing.
    #[derive(Parser, Debug)]
    struct TestCli {
        #[command(subcommand)]
        command: Commands,
    }

    fn parse(args: &[&str]) -> Commands {
        TestCli::parse_from(args).command
    }

    // AC-001: subcommand dispatch routes correctly
    #[test]
    fn test_bc_5_11_001_subcommand_dispatch() {
        // Each supported subcommand parses without error.
        let cases: &[&[&str]] = &[
            &["forge-mcp", "list"],
            &["forge-mcp", "list", "stdio://myserver"],
            &["forge-mcp", "call", "stdio://myserver", "my_tool"],
            &["forge-mcp", "info", "stdio://myserver"],
            &["forge-mcp", "grep", "search_pattern"],
            &["forge-mcp", "test", "stdio://myserver"],
            &["forge-mcp", "audit", "stdio://myserver"],
            &["forge-mcp", "tui"],
            &["forge-mcp", "daemon", "start"],
            &["forge-mcp", "daemon", "stop"],
            &["forge-mcp", "daemon", "restart"],
            &["forge-mcp", "daemon", "sessions"],
        ];

        for args in cases {
            let result = TestCli::try_parse_from(*args);
            assert!(
                result.is_ok(),
                "failed to parse args {:?}: {:?}",
                args,
                result.err()
            );
        }
    }

    // AC-001: unknown subcommand is rejected by clap (exits non-zero in binary,
    // but here we just assert the parse returns an Err).
    #[test]
    fn test_bc_5_11_001_unknown_subcommand_rejected() {
        let result = TestCli::try_parse_from(["forge-mcp", "frobnicate"]);
        assert!(
            result.is_err(),
            "unknown subcommand should fail to parse"
        );
    }

    // parse_stdio_uri: valid stdio URI
    #[test]
    fn test_parse_stdio_uri_valid() {
        let (cmd, args) = parse_stdio_uri("stdio:///usr/bin/my-server").unwrap();
        assert_eq!(cmd, "/usr/bin/my-server");
        assert!(args.is_empty());
    }

    // parse_stdio_uri: invalid scheme
    #[test]
    fn test_parse_stdio_uri_invalid_scheme() {
        assert!(parse_stdio_uri("http://example.com").is_err());
    }
}
