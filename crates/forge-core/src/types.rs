//! Unified domain types for MCP server definitions.
//!
//! This module is **pure** — no I/O, no side effects.
//! Defined here as L1 domain kernel per STORY-001 architecture.

use std::collections::HashMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// Re-export EditorKind from forge-discovery's types for use in ServerEntry.
// We re-define here to avoid a circular dependency: forge-core must not depend
// on forge-discovery (which depends on forge-core). So we define EditorKind
// in both crates, or — per the architecture — we keep forge-core's ServerEntry
// self-contained with its own source-editor type.

/// Which editor produced the config that defined this server.
///
/// Mirrors `forge_discovery::EditorKind` — kept separate to avoid a circular
/// dependency (forge-core ← forge-discovery ← forge-core would be a cycle).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum EditorKind {
    /// Anthropic Claude Desktop application.
    ClaudeDesktop,
    /// Cursor IDE.
    Cursor,
    /// Visual Studio Code.
    VSCode,
    /// Windsurf (Codeium) IDE.
    Windsurf,
}

/// Transport configuration for stdio (subprocess) servers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StdioConfig {
    /// Executable to launch.
    pub command: String,
    /// Command-line arguments passed to the executable.
    pub args: Vec<String>,
    /// Environment variables passed to the subprocess.
    /// Values are stored as-is; expansion is deferred to connection time (BC-1.02.001).
    pub env: HashMap<String, String>,
}

/// Transport configuration for HTTP / SSE servers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpConfig {
    /// Base URL of the MCP HTTP endpoint.
    pub url: String,
    /// HTTP headers to include in every request.
    pub headers: HashMap<String, String>,
}

/// Discriminated union of transport configurations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum TransportConfig {
    /// Process-based stdio transport.
    Stdio(StdioConfig),
    /// HTTP / SSE transport.
    Http(HttpConfig),
}

/// Attribution: which config file produced a given server entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigSource {
    /// Which editor's config file was parsed.
    pub editor: EditorKind,
    /// Absolute path to the config file.
    pub path: PathBuf,
}

/// A fully-parsed MCP server definition, normalised from any supported editor schema.
///
/// This is the unified output type produced by `parse_config()` in `forge-discovery`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerEntry {
    /// Server name (the key under `mcpServers` / `servers`).
    pub name: String,
    /// Transport type discriminant.
    pub transport: Transport,
    /// Full transport configuration (command/args/env or url/headers).
    pub config: TransportConfig,
    /// Which editor config file this server was read from.
    pub source_editor: EditorKind,
    /// Path of the config file this entry was parsed from.
    pub source_path: PathBuf,
    /// Whether this server is active.
    /// Cursor's `"disabled": true` maps to `enabled: false`.
    /// Defaults to `true` when the field is absent.
    pub enabled: bool,
    /// Tool names that are always permitted without user confirmation.
    /// Maps from Cursor's `"alwaysAllow"` field. Defaults to empty.
    pub always_allow: Vec<String>,
}

/// Transport discriminant (without carrying config data).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Transport {
    /// stdio subprocess transport.
    Stdio,
    /// HTTP / SSE transport.
    Http,
}

// ── Aggregation types ─────────────────────────────────────────────────────────

/// A single source entry within a conflict record.
///
/// Captures the editor, file path, and the full `ServerEntry` from that source.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictSource {
    /// Which editor's config file produced this entry.
    pub editor: EditorKind,
    /// Path of the config file.
    pub path: PathBuf,
    /// The server entry as parsed from this source.
    pub entry: ServerEntry,
}

/// Records that two or more config sources define the same server name with
/// different parameters.
///
/// The winning entry is recorded in [`ServerRegistry`]; the loser(s) are
/// preserved here for user-driven resolution.
///
/// Error code **E-CFG-006** is emitted for every `ConflictRecord`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConflictRecord {
    /// Server name that appeared in multiple sources with conflicting definitions.
    pub server_name: String,
    /// All sources that define this server (≥ 2 entries).
    pub sources: Vec<ConflictSource>,
}

/// Unified registry of all discovered MCP servers across all editors.
///
/// Produced by `forge_discovery::aggregator::aggregate_configs`.
///
/// - `servers` maps server name → winning `ServerEntry`.
/// - `conflicts` lists all detected conflicts (servers defined differently in
///   multiple config files).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServerRegistry {
    /// Winning server entry per unique server name.
    /// Ordered by insertion (first-discovered wins on conflict).
    pub servers: std::collections::HashMap<String, ServerEntry>,
    /// One `ConflictRecord` per server name that had conflicting definitions.
    pub conflicts: Vec<ConflictRecord>,
}
