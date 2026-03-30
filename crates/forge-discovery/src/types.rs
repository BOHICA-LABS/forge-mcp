//! Core types for config file discovery.
//!
//! This module is **pure** — no I/O, no side effects.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Which editor produced this config file.
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

/// Whether the config file is editor-global or project/workspace-scoped.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ConfigScope {
    /// Editor-level global config (applies to all projects).
    Global,
    /// Project-local config (relative to project root).
    Project,
    /// Workspace-scoped config (VS Code `.vscode/mcp.json`).
    Workspace,
}

/// A single discovered (or probed) MCP config file location.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredConfig {
    /// Which editor this config file belongs to.
    pub editor: EditorKind,
    /// The absolute path where the config file is expected.
    pub path: PathBuf,
    /// Whether this is a global, project-local, or workspace config.
    pub scope: ConfigScope,
    /// `true` if the file exists on the filesystem at the time of discovery.
    pub exists: bool,
    /// Non-`None` when the file exists but could not be read (e.g. permission denied).
    pub access_error: Option<String>,
}
