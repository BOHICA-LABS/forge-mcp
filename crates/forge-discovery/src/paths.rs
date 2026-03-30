//! Pure path-table functions for MCP config file locations.
//!
//! This module is **pure** — it performs no I/O.  All functions accept an
//! explicit `home` directory argument rather than calling `dirs::home_dir()`
//! directly so they are easily unit-testable.

use std::path::{Path, PathBuf};

use crate::types::{ConfigScope, DiscoveredConfig, EditorKind};

/// Operating-system selector, passed explicitly so callers can override in tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Os {
    MacOs,
    Windows,
    Linux,
}

impl Os {
    /// Return the `Os` variant that matches the compile-time target OS.
    pub fn current() -> Self {
        #[cfg(target_os = "macos")]
        return Os::MacOs;
        #[cfg(target_os = "windows")]
        return Os::Windows;
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        return Os::Linux;
    }
}

/// Build the full list of (editor, scope, absolute-path) tuples for `os`.
///
/// `home`        – the user's home directory.
/// `project_root` – optional project root; when `None`, project-scoped
///                  entries are omitted.
///
/// The list is **deterministic**: Claude Desktop → Cursor global → Cursor
/// project → VS Code global → VS Code workspace → Windsurf.
///
/// This function is **pure**: it performs no filesystem I/O.
pub fn os_paths(
    os: Os,
    home: &Path,
    project_root: Option<&Path>,
) -> Vec<(EditorKind, ConfigScope, PathBuf)> {
    let mut entries: Vec<(EditorKind, ConfigScope, PathBuf)> = Vec::new();

    // ── Claude Desktop ────────────────────────────────────────────────────────
    let claude_global = match os {
        Os::MacOs => home
            .join("Library")
            .join("Application Support")
            .join("Claude")
            .join("claude_desktop_config.json"),
        Os::Windows => {
            // On Windows we use %APPDATA% which is home/../AppData/Roaming
            // The caller is responsible for passing the correct `home`.
            home.join("AppData")
                .join("Roaming")
                .join("Claude")
                .join("claude_desktop_config.json")
        }
        Os::Linux => home
            .join(".config")
            .join("Claude")
            .join("claude_desktop_config.json"),
    };
    entries.push((
        EditorKind::ClaudeDesktop,
        ConfigScope::Global,
        claude_global,
    ));

    // ── Cursor global ─────────────────────────────────────────────────────────
    let cursor_global = match os {
        Os::MacOs | Os::Linux => home.join(".cursor").join("mcp.json"),
        Os::Windows => home.join(".cursor").join("mcp.json"),
    };
    entries.push((EditorKind::Cursor, ConfigScope::Global, cursor_global));

    // ── Cursor project-local ──────────────────────────────────────────────────
    if let Some(root) = project_root {
        entries.push((
            EditorKind::Cursor,
            ConfigScope::Project,
            root.join(".cursor").join("mcp.json"),
        ));
    }

    // ── VS Code global ────────────────────────────────────────────────────────
    let vscode_global = match os {
        Os::MacOs => home
            .join("Library")
            .join("Application Support")
            .join("Code")
            .join("User")
            .join("mcp.json"),
        Os::Windows => home
            .join("AppData")
            .join("Roaming")
            .join("Code")
            .join("User")
            .join("mcp.json"),
        Os::Linux => home
            .join(".config")
            .join("Code")
            .join("User")
            .join("mcp.json"),
    };
    entries.push((EditorKind::VSCode, ConfigScope::Global, vscode_global));

    // ── VS Code workspace ─────────────────────────────────────────────────────
    if let Some(root) = project_root {
        entries.push((
            EditorKind::VSCode,
            ConfigScope::Workspace,
            root.join(".vscode").join("mcp.json"),
        ));
    }

    // ── Windsurf global ───────────────────────────────────────────────────────
    let windsurf_global = match os {
        Os::MacOs | Os::Linux => home
            .join(".codeium")
            .join("windsurf")
            .join("mcp_config.json"),
        Os::Windows => home
            .join(".codeium")
            .join("windsurf")
            .join("mcp_config.json"),
    };
    entries.push((EditorKind::Windsurf, ConfigScope::Global, windsurf_global));

    entries
}

/// Convert path table entries into unprobed [`DiscoveredConfig`] stubs
/// (all `exists = false`, no `access_error`).
///
/// The caller (`discover_configs`) is responsible for probing the filesystem.
pub fn make_stubs(entries: Vec<(EditorKind, ConfigScope, PathBuf)>) -> Vec<DiscoveredConfig> {
    entries
        .into_iter()
        .map(|(editor, scope, path)| DiscoveredConfig {
            editor,
            path,
            scope,
            exists: false,
            access_error: None,
        })
        .collect()
}
