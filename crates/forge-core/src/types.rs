//! Unified domain types for MCP server definitions.
//!
//! This module is **pure** — no I/O, no side effects.
//! Defined here as L1 domain kernel per STORY-001 architecture.

use std::collections::HashMap;
use std::path::PathBuf;

use rmcp::model::{ClientCapabilities, ServerCapabilities};
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

// ── Capability negotiation types ──────────────────────────────────────────────

/// The result of a completed MCP capability negotiation handshake.
///
/// Produced by the `initialize` / `initialized` exchange. rmcp performs the
/// actual JSON-RPC round-trip; this struct captures the negotiated results in
/// a Forge-specific, ergonomic wrapper.
///
/// ## Protocol flow
/// ```text
/// Client → Server  initialize(clientInfo, clientCapabilities)
/// Server → Client  InitializeResult(serverCapabilities, protocolVersion)
/// Client → Server  notifications/initialized
/// ```
///
/// After that exchange this struct holds both sides and the agreed protocol
/// version so capability-guard methods can answer without further I/O.
#[derive(Debug, Clone)]
pub struct NegotiatedCapabilities {
    /// What the server advertised in its `InitializeResult`.
    pub server: ServerCapabilities,
    /// What Forge MCP advertised in the `initialize` request.
    pub client: ClientCapabilities,
    /// The protocol version string the server reported (e.g. `"2025-06-18"`).
    pub protocol_version: String,
}

impl NegotiatedCapabilities {
    /// Create from the rmcp-parsed handshake results.
    pub fn new(
        server: ServerCapabilities,
        client: ClientCapabilities,
        protocol_version: impl Into<String>,
    ) -> Self {
        Self {
            server,
            client,
            protocol_version: protocol_version.into(),
        }
    }
}

// ── Spec version compatibility ─────────────────────────────────────────────────

/// Known MCP specification versions in chronological order.
///
/// Aligned with the versions known to `rmcp 1.3`:
/// `2024-11-05`, `2025-03-26`, `2025-06-18`.
///
/// Per AC-004 / BC-2.04.003: unknown or future versions are treated as the
/// highest known spec (`2025-06-18`) semantics, with a warning logged.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum SpecVersion {
    /// `2024-11-05` — logging added; no elicitation, no streamable HTTP.
    V2024_11_05,
    /// `2025-03-26` — streamable HTTP added; no elicitation.
    V2025_03_26,
    /// `2025-06-18` — full feature set including elicitation.
    V2025_06_18,
}

impl SpecVersion {
    /// Parse a version string into a known `SpecVersion`, returning `None`
    /// for unrecognised strings.
    pub fn parse(version: &str) -> Option<Self> {
        match version {
            "2024-11-05" => Some(Self::V2024_11_05),
            "2025-03-26" => Some(Self::V2025_03_26),
            "2025-06-18" => Some(Self::V2025_06_18),
            _ => None,
        }
    }

    /// Returns the canonical version string for this spec version.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::V2024_11_05 => "2024-11-05",
            Self::V2025_03_26 => "2025-03-26",
            Self::V2025_06_18 => "2025-06-18",
        }
    }
}

/// The set of optional MCP features controlled by version-based degradation.
///
/// These flags represent protocol methods / transport features that are only
/// available in specific spec versions.  The capability guard in
/// `McpConnection` ANDs these flags with the server's advertised capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeatureSet {
    /// `elicitation/create` — first available in `2025-11-25`.
    pub elicitation: bool,
    /// Streamable-HTTP transport — first available in `2025-11-25`.
    pub streamable_http: bool,
    /// Server-side logging — first available in `2024-11-05`.
    pub logging: bool,
    /// Basic tools / resources / prompts — available in all known versions.
    pub tools: bool,
    /// Resources capability — available in all known versions.
    pub resources: bool,
    /// Prompts capability — available in all known versions.
    pub prompts: bool,
}

impl FeatureSet {
    /// All features enabled (latest spec).
    pub const ALL: Self = Self {
        elicitation: true,
        streamable_http: true,
        logging: true,
        tools: true,
        resources: true,
        prompts: true,
    };

    /// Conservative feature set used for unknown/future versions.
    ///
    /// Enables only universally-supported features; anything version-specific
    /// is disabled until the version can be confirmed.
    pub const CONSERVATIVE: Self = Self {
        elicitation: false,
        streamable_http: false,
        logging: false,
        tools: true,
        resources: true,
        prompts: true,
    };
}

/// Returns the `FeatureSet` available for a given protocol version string.
///
/// # Version mapping
/// | Version string | Feature set |
/// |----------------|-------------|
/// | `"2025-06-18"` | All features (elicitation, streamable HTTP, logging, tools, resources, prompts) |
/// | `"2025-03-26"` | No elicitation; streamable HTTP, logging + basic features |
/// | `"2024-11-05"` | No elicitation, no streamable HTTP; logging + basic features |
/// | *(unknown)*    | Conservative set (basic tools/resources/prompts only), no crash |
pub fn features_for_version(version: &str) -> FeatureSet {
    match SpecVersion::parse(version) {
        Some(SpecVersion::V2025_06_18) => FeatureSet::ALL,
        Some(SpecVersion::V2025_03_26) => FeatureSet {
            elicitation: false,
            streamable_http: true,
            logging: true,
            tools: true,
            resources: true,
            prompts: true,
        },
        Some(SpecVersion::V2024_11_05) => FeatureSet {
            elicitation: false,
            streamable_http: false,
            logging: true,
            tools: true,
            resources: true,
            prompts: true,
        },
        // Unknown or future version → conservative defaults, no crash (AC-004).
        // Per story AC-004 / EC-002: future unknown versions continue without crash.
        None => FeatureSet::CONSERVATIVE,
    }
}

#[cfg(test)]
mod version_tests {
    #![allow(non_snake_case)]
    use super::*;

    // ── SpecVersion::parse ───────────────────────────────────────────────────

    #[test]
    fn test_spec_version_parse_known_versions() {
        assert_eq!(SpecVersion::parse("2024-11-05"), Some(SpecVersion::V2024_11_05));
        assert_eq!(SpecVersion::parse("2025-03-26"), Some(SpecVersion::V2025_03_26));
        assert_eq!(SpecVersion::parse("2025-06-18"), Some(SpecVersion::V2025_06_18));
    }

    #[test]
    fn test_spec_version_parse_unknown_returns_none() {
        assert_eq!(SpecVersion::parse("2026-01-01"), None);
        assert_eq!(SpecVersion::parse(""), None);
        assert_eq!(SpecVersion::parse("garbage"), None);
        assert_eq!(SpecVersion::parse("2025-11-25"), None); // not a real version
        assert_eq!(SpecVersion::parse("2024-10-07"), None); // pre-stable
    }

    #[test]
    fn test_spec_version_ordering() {
        assert!(SpecVersion::V2024_11_05 < SpecVersion::V2025_03_26);
        assert!(SpecVersion::V2025_03_26 < SpecVersion::V2025_06_18);
    }

    // ── features_for_version ─────────────────────────────────────────────────

    /// AC-001 (version detection): latest version enables all features.
    #[test]
    fn test_BC_2_04_003_features_latest_version_all_enabled() {
        let fs = features_for_version("2025-06-18");
        assert!(fs.elicitation, "elicitation must be enabled for 2025-06-18");
        assert!(fs.streamable_http, "streamable_http must be enabled for 2025-06-18");
        assert!(fs.logging, "logging must be enabled for 2025-06-18");
        assert!(fs.tools);
        assert!(fs.resources);
        assert!(fs.prompts);
    }

    /// AC-003 (method gating): older version disables elicitation.
    #[test]
    fn test_BC_2_04_003_features_2024_11_05_no_elicitation() {
        let fs = features_for_version("2024-11-05");
        assert!(!fs.elicitation, "elicitation must be disabled for 2024-11-05");
        assert!(!fs.streamable_http, "streamable_http must be disabled for 2024-11-05");
        assert!(fs.logging, "logging must be enabled for 2024-11-05");
        assert!(fs.tools);
        assert!(fs.resources);
        assert!(fs.prompts);
    }

    /// 2025-03-26 has streamable HTTP but not elicitation.
    #[test]
    fn test_features_2025_03_26_has_streamable_no_elicitation() {
        let fs = features_for_version("2025-03-26");
        assert!(!fs.elicitation, "elicitation must be disabled for 2025-03-26");
        assert!(fs.streamable_http, "streamable_http must be enabled for 2025-03-26");
        assert!(fs.logging);
        assert!(fs.tools);
        assert!(fs.resources);
        assert!(fs.prompts);
    }

    /// AC-004 (unknown version continues): unknown / future version → conservative, no crash.
    #[test]
    fn test_BC_2_04_003_features_unknown_version_conservative() {
        // Unknown version must not crash and must return conservative set.
        let fs = features_for_version("2026-01-01");
        assert!(!fs.elicitation, "unknown version must disable elicitation");
        assert!(fs.tools, "unknown version must still allow basic tools");
        assert!(fs.resources);
        assert!(fs.prompts);
    }

    #[test]
    fn test_BC_2_04_003_features_empty_string_conservative() {
        let fs = features_for_version("");
        assert!(!fs.elicitation);
        assert!(fs.tools);
    }
}
