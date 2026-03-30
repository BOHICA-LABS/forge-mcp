//! Config source aggregation with conflict attribution.
//!
//! This module is **pure** — no I/O, no side effects.
//!
//! ## Responsibilities
//! - Merge `ServerEntry` lists from multiple parsed config files into a single
//!   [`ServerRegistry`].
//! - Detect conflicts: same server name, different parameters across sources.
//! - Attribute each server to its source editor and file path.
//! - Apply deterministic priority: discovery order determines the winner when
//!   two sources have conflicting definitions.
//!
//! ## Conflict semantics
//! A **conflict** arises when the same server name appears in two or more
//! sources with _different_ transport configurations (command/url, args, env,
//! or headers).  Identical definitions across sources are **not** conflicts —
//! they are silently deduplicated (POST-003 of BC-1.01.003).
//!
//! ## Priority order
//! The caller controls priority by the order of `sources` slices passed to
//! [`aggregate_configs`].  The first slice is highest priority.  Within a
//! source, servers are processed in the order they appear.
//!
//! For the canonical discovery pipeline the order mirrors BC-1.01.001:
//! project-local (highest) → global.

use indexmap::IndexMap;

use forge_core::types::{ConflictRecord, ConflictSource, ServerEntry, ServerRegistry};

/// Aggregate server entries from multiple parsed config sources into a unified
/// [`ServerRegistry`].
///
/// # Arguments
/// * `sources` — a slice of server entry lists.  Each element represents the
///   parsed output of one config file.  **The slice is ordered by priority:
///   element `0` has the highest priority.**
///
/// # Return value
/// A [`ServerRegistry`] containing:
/// - `servers`: one winning [`ServerEntry`] per unique server name.
/// - `conflicts`: one [`ConflictRecord`] per server name that was defined
///   differently in two or more sources.
///
/// # Purity
/// This function performs no I/O.  It is pure: same input → same output.
///
/// # Warning events
/// Conflict detection emits `tracing::warn!` for each conflict (E-CFG-006).
/// These go to stderr via the `tracing` subscriber configured by the caller.
///
/// # Edge cases
/// - Empty `sources` → empty registry, no conflicts (AC-004).
/// - `sources` with zero servers → empty registry (AC-004).
/// - Disabled servers (`enabled: false`) are included (AC-005).
pub fn aggregate_configs(sources: &[Vec<ServerEntry>]) -> ServerRegistry {
    // `entries_by_name` maps server name → (winning_entry, all_sources_for_that_name).
    // We use `IndexMap` to preserve insertion order (first-discovered wins).
    let mut entries_by_name: IndexMap<String, Vec<ServerEntry>> = IndexMap::new();

    for source_entries in sources {
        for entry in source_entries {
            entries_by_name
                .entry(entry.name.clone())
                .or_default()
                .push(entry.clone());
        }
    }

    let mut servers = std::collections::HashMap::new();
    let mut conflicts = Vec::new();

    for (name, candidates) in entries_by_name {
        match candidates.len() {
            0 => {
                // Unreachable in practice (we only insert non-empty), but be safe.
            }
            1 => {
                // AC-002 / POST-002: single source — no conflict.
                servers.insert(name, candidates.into_iter().next().expect("len == 1"));
            }
            _ => {
                // Check whether all candidates are identical (POST-003: dedup).
                let first = &candidates[0];
                let all_identical = candidates[1..].iter().all(|c| configs_equal(first, c));

                if all_identical {
                    // POST-003: deduplicated identical entries — first source wins,
                    // no ConflictRecord created.
                    servers.insert(name, candidates.into_iter().next().expect("len >= 2"));
                } else {
                    // POST-004 / AC-002: genuine conflict.
                    let winner = candidates[0].clone();

                    let conflict_sources: Vec<ConflictSource> = candidates
                        .into_iter()
                        .map(|e| ConflictSource {
                            editor: e.source_editor.clone(),
                            path: e.source_path.clone(),
                            entry: e,
                        })
                        .collect();

                    let source_a = &conflict_sources[0];
                    let source_b = &conflict_sources[1];

                    // AC-003 / POST-005: emit E-CFG-006 warning.
                    tracing::warn!(
                        "E-CFG-006: Config conflict: server \"{}\" defined differently in {:?} ({:?}) and {:?} ({:?})",
                        name,
                        source_a.editor,
                        source_a.path,
                        source_b.editor,
                        source_b.path,
                    );

                    conflicts.push(ConflictRecord {
                        server_name: name.clone(),
                        sources: conflict_sources,
                    });

                    servers.insert(name, winner);
                }
            }
        }
    }

    ServerRegistry { servers, conflicts }
}

/// Compare two [`ServerEntry`] values by their transport configuration only
/// (ignoring source attribution fields `source_editor` and `source_path`).
///
/// Two entries are considered "equal" for deduplication purposes when their
/// transport type and config are identical.  The `name` field is always the
/// same at this point (we group by name).  `enabled` and `always_allow` are
/// also compared because they affect runtime behaviour.
fn configs_equal(a: &ServerEntry, b: &ServerEntry) -> bool {
    a.transport == b.transport
        && a.config == b.config
        && a.enabled == b.enabled
        && a.always_allow == b.always_allow
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use forge_core::types::{EditorKind, ServerEntry, StdioConfig, Transport, TransportConfig};

    use super::*;

    // ── Helpers ───────────────────────────────────────────────────────────────

    fn stdio_entry(
        name: &str,
        command: &str,
        editor: EditorKind,
        path: &str,
        enabled: bool,
    ) -> ServerEntry {
        ServerEntry {
            name: name.to_owned(),
            transport: Transport::Stdio,
            config: TransportConfig::Stdio(StdioConfig {
                command: command.to_owned(),
                args: vec![],
                env: Default::default(),
            }),
            source_editor: editor,
            source_path: PathBuf::from(path),
            enabled,
            always_allow: vec![],
        }
    }

    fn http_entry(name: &str, url: &str, editor: EditorKind, path: &str) -> ServerEntry {
        use forge_core::types::{HttpConfig, Transport, TransportConfig};
        ServerEntry {
            name: name.to_owned(),
            transport: Transport::Http,
            config: TransportConfig::Http(HttpConfig {
                url: url.to_owned(),
                headers: Default::default(),
            }),
            source_editor: editor,
            source_path: PathBuf::from(path),
            enabled: true,
            always_allow: vec![],
        }
    }

    // ── Tests ─────────────────────────────────────────────────────────────────

    /// AC-004: empty input → empty registry, no conflicts.
    #[test]
    fn test_bc_1_01_003_empty_input() {
        let registry = aggregate_configs(&[]);
        assert!(registry.servers.is_empty(), "expected no servers");
        assert!(registry.conflicts.is_empty(), "expected no conflicts");
    }

    /// AC-001 / TV-001: single source with one server → no conflicts.
    #[test]
    fn test_bc_1_01_003_aggregate_basic() {
        let source = vec![stdio_entry(
            "server-a",
            "npx server-a",
            EditorKind::ClaudeDesktop,
            "/claude/config.json",
            true,
        )];

        let registry = aggregate_configs(&[source]);

        assert_eq!(registry.servers.len(), 1);
        assert!(registry.servers.contains_key("server-a"));
        assert!(registry.conflicts.is_empty());
    }

    /// AC-002: same server name in two sources with different configs → conflict detected.
    #[test]
    fn test_bc_1_01_003_conflict_detection() {
        let source_a = vec![stdio_entry(
            "db",
            "npx db-mcp",
            EditorKind::ClaudeDesktop,
            "/claude/config.json",
            true,
        )];
        let source_b = vec![http_entry(
            "db",
            "https://db.example.com",
            EditorKind::Cursor,
            "/cursor/.cursor/mcp.json",
        )];

        let registry = aggregate_configs(&[source_a, source_b]);

        // Winning entry should be from ClaudeDesktop (first source).
        assert_eq!(registry.servers.len(), 1);
        let winner = registry.servers.get("db").expect("db must be present");
        assert_eq!(winner.source_editor, EditorKind::ClaudeDesktop);

        // Exactly one conflict.
        assert_eq!(registry.conflicts.len(), 1);
        let conflict = &registry.conflicts[0];
        assert_eq!(conflict.server_name, "db");
        assert_eq!(conflict.sources.len(), 2);
        assert_eq!(conflict.sources[0].editor, EditorKind::ClaudeDesktop);
        assert_eq!(conflict.sources[1].editor, EditorKind::Cursor);
    }

    /// AC-003: conflict warning E-CFG-006 is emitted (we verify the conflict
    /// record is present — tracing output is hard to intercept in unit tests;
    /// the warn! call is exercised by the conflict_detection test above).
    #[test]
    fn test_bc_1_01_003_conflict_warning_emitted() {
        let source_a = vec![stdio_entry(
            "my-server",
            "npx my-server",
            EditorKind::VSCode,
            "/vscode/mcp.json",
            true,
        )];
        let source_b = vec![http_entry(
            "my-server",
            "https://my-server.example.com",
            EditorKind::Windsurf,
            "/windsurf/config.json",
        )];

        let registry = aggregate_configs(&[source_a, source_b]);

        // The conflict record must exist (proves the warning code path ran).
        assert_eq!(registry.conflicts.len(), 1);
        assert_eq!(registry.conflicts[0].server_name, "my-server");
    }

    /// TV-005 / EC-004: project-local overrides global (first source = project).
    /// The caller places project-scoped entries first.
    #[test]
    fn test_bc_1_01_003_project_local_overrides_global() {
        // Project-scoped cursor entry (higher priority — placed first).
        let project = vec![stdio_entry(
            "lint",
            "npx lint-mcp-v2",
            EditorKind::Cursor,
            "/my-project/.cursor/mcp.json",
            true,
        )];
        // Global cursor entry (lower priority — placed second).
        let global = vec![stdio_entry(
            "lint",
            "npx lint-mcp",
            EditorKind::Cursor,
            "~/.cursor/mcp.json",
            true,
        )];

        let registry = aggregate_configs(&[project, global]);

        // Winner must be the project-local entry.
        let winner = registry.servers.get("lint").expect("lint must be present");
        assert_eq!(
            winner.source_path,
            PathBuf::from("/my-project/.cursor/mcp.json")
        );

        // Different commands → conflict recorded.
        assert_eq!(registry.conflicts.len(), 1);
    }

    /// TV-001 variant: multiple editors, no overlapping names → clean merge.
    #[test]
    fn test_bc_1_01_003_clean_merge_no_conflicts() {
        let claude = vec![stdio_entry(
            "server-a",
            "npx server-a",
            EditorKind::ClaudeDesktop,
            "/claude/config.json",
            true,
        )];
        let cursor = vec![stdio_entry(
            "server-b",
            "npx server-b",
            EditorKind::Cursor,
            "/cursor/mcp.json",
            true,
        )];
        let vscode = vec![stdio_entry(
            "server-c",
            "npx server-c",
            EditorKind::VSCode,
            "/vscode/mcp.json",
            true,
        )];

        let registry = aggregate_configs(&[claude, cursor, vscode]);

        assert_eq!(
            registry.servers.len(),
            3,
            "all three servers must be present"
        );
        assert!(registry.conflicts.is_empty(), "no conflicts expected");
        assert!(registry.servers.contains_key("server-a"));
        assert!(registry.servers.contains_key("server-b"));
        assert!(registry.servers.contains_key("server-c"));
    }

    /// AC-005: servers with `enabled: false` are included in the registry.
    #[test]
    fn test_bc_1_01_003_disabled_servers_included() {
        let source = vec![
            stdio_entry(
                "active",
                "npx active",
                EditorKind::ClaudeDesktop,
                "/claude/config.json",
                true,
            ),
            stdio_entry(
                "inactive",
                "npx inactive",
                EditorKind::ClaudeDesktop,
                "/claude/config.json",
                false, // disabled
            ),
        ];

        let registry = aggregate_configs(&[source]);

        assert_eq!(
            registry.servers.len(),
            2,
            "disabled server must be included"
        );
        let inactive = registry
            .servers
            .get("inactive")
            .expect("inactive must be present");
        assert!(!inactive.enabled, "server must remain disabled");
        assert!(registry.conflicts.is_empty());
    }

    /// POST-003: same server name, identical config in two sources → no conflict,
    /// deduplicated to first.
    #[test]
    fn test_bc_1_01_003_identical_dedup_no_conflict() {
        let source_a = vec![stdio_entry(
            "server-a",
            "npx server-a",
            EditorKind::ClaudeDesktop,
            "/claude/config.json",
            true,
        )];
        let source_b = vec![stdio_entry(
            "server-a",
            "npx server-a", // identical command
            EditorKind::VSCode,
            "/vscode/mcp.json",
            true,
        )];

        let registry = aggregate_configs(&[source_a, source_b]);

        assert_eq!(registry.servers.len(), 1);
        assert!(
            registry.conflicts.is_empty(),
            "identical entries must not create a conflict"
        );
        // Winner should be from first source (ClaudeDesktop).
        let winner = registry.servers.get("server-a").expect("must be present");
        assert_eq!(winner.source_editor, EditorKind::ClaudeDesktop);
    }
}
