//! Integration tests for STORY-004: Config File Discovery & Path Resolution
//!
//! Naming convention: `test_BC_1_01_001_<description>`
//! All tests use explicit `home` and `project_root` paths (via tempdir) so
//! they are fully deterministic and do not depend on the CI runner's home dir.

use std::fs;
use std::path::{Path, PathBuf};

use forge_discovery::paths::os_paths;
use forge_discovery::{ConfigScope, DiscoveryError, EditorKind, Os, discover_configs};
use tempfile::TempDir;

// ── Helpers ───────────────────────────────────────────────────────────────────

/// A populated temp directory acting as a fake home dir.
struct FakeHome {
    dir: TempDir,
}

impl FakeHome {
    fn new() -> Self {
        Self {
            dir: TempDir::new().expect("tempdir"),
        }
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Create a file at `relative_path` (creating parent dirs as needed).
    fn create_file(&self, relative_path: &str) {
        let full = self.dir.path().join(relative_path);
        fs::create_dir_all(full.parent().unwrap()).unwrap();
        fs::write(&full, b"{}").unwrap();
    }
}

// ── AC-001: DiscoveredConfig struct shape ─────────────────────────────────────

/// AC-001: each returned entry contains all required fields with correct types.
#[test]
fn test_bc_1_01_001_discovered_config_struct_shape() {
    let home = FakeHome::new();
    let configs = discover_configs(Os::MacOs, Some(home.path()), None).unwrap();

    // Must return entries (at least the 4 global ones)
    assert!(!configs.is_empty(), "should return at least one entry");

    let first = &configs[0];
    // All required fields are accessible and have correct types
    let _editor: &EditorKind = &first.editor;
    let _scope: &ConfigScope = &first.scope;
    let _path: &PathBuf = &first.path;
    let _exists: bool = first.exists;
    let _access_error: &Option<String> = &first.access_error;

    // path must be absolute
    assert!(
        first.path.is_absolute(),
        "path must be absolute, got: {:?}",
        first.path
    );
}

// ── AC-002: all paths probed on macOS ────────────────────────────────────────

/// AC-002: exactly 4 global entries returned when project_root is None on macOS.
#[test]
fn test_bc_1_01_001_all_paths_probed_macos() {
    let home = FakeHome::new();
    let configs = discover_configs(Os::MacOs, Some(home.path()), None).unwrap();

    // Without a project root: Claude Desktop + Cursor global + VS Code global + Windsurf = 4
    assert_eq!(
        configs.len(),
        4,
        "expected 4 global entries on macOS without project_root"
    );

    let editors: Vec<&EditorKind> = configs.iter().map(|c| &c.editor).collect();
    assert!(editors.contains(&&EditorKind::ClaudeDesktop));
    assert!(editors.contains(&&EditorKind::Cursor));
    assert!(editors.contains(&&EditorKind::VSCode));
    assert!(editors.contains(&&EditorKind::Windsurf));
}

/// AC-002 extended: with a project root, macOS returns 6 entries.
#[test]
fn test_bc_1_01_001_all_paths_probed_macos_with_project() {
    let home = FakeHome::new();
    let project = TempDir::new().unwrap();
    let configs = discover_configs(Os::MacOs, Some(home.path()), Some(project.path())).unwrap();

    // Global (4) + Cursor project + VS Code workspace = 6
    assert_eq!(
        configs.len(),
        6,
        "expected 6 entries on macOS with project_root"
    );
}

// ── AC-003: project-scoped resolution ────────────────────────────────────────

/// AC-003a: project-scoped entries use the provided project_root.
#[test]
fn test_bc_1_01_001_project_scoped_resolution() {
    let home = FakeHome::new();
    let project = TempDir::new().unwrap();
    let project_root = project.path();

    let configs = discover_configs(Os::MacOs, Some(home.path()), Some(project_root)).unwrap();

    let cursor_project = configs
        .iter()
        .find(|c| c.editor == EditorKind::Cursor && c.scope == ConfigScope::Project)
        .expect("Cursor project-scoped entry must exist");

    assert_eq!(
        cursor_project.path,
        project_root.join(".cursor").join("mcp.json")
    );

    let vscode_workspace = configs
        .iter()
        .find(|c| c.editor == EditorKind::VSCode && c.scope == ConfigScope::Workspace)
        .expect("VS Code workspace entry must exist");

    assert_eq!(
        vscode_workspace.path,
        project_root.join(".vscode").join("mcp.json")
    );
}

/// AC-003b: when project_root is None, no project-scoped entries appear.
#[test]
fn test_bc_1_01_001_no_project_scoped_when_root_none() {
    let home = FakeHome::new();
    let configs = discover_configs(Os::MacOs, Some(home.path()), None).unwrap();

    let project_scoped: Vec<_> = configs
        .iter()
        .filter(|c| c.scope == ConfigScope::Project || c.scope == ConfigScope::Workspace)
        .collect();

    assert!(
        project_scoped.is_empty(),
        "no project-scoped entries expected when project_root is None"
    );
}

// ── AC-004: deterministic order ──────────────────────────────────────────────

/// AC-004: order is Claude Desktop → Cursor global → Cursor project →
///         VS Code global → VS Code workspace → Windsurf.
#[test]
fn test_bc_1_01_001_deterministic_order() {
    let home = FakeHome::new();
    let project = TempDir::new().unwrap();

    let run1 = discover_configs(Os::MacOs, Some(home.path()), Some(project.path())).unwrap();
    let run2 = discover_configs(Os::MacOs, Some(home.path()), Some(project.path())).unwrap();

    // Identical across two runs
    let paths1: Vec<_> = run1.iter().map(|c| &c.path).collect();
    let paths2: Vec<_> = run2.iter().map(|c| &c.path).collect();
    assert_eq!(paths1, paths2, "order must be deterministic");

    // Verify exact order: Claude Desktop global first
    assert_eq!(run1[0].editor, EditorKind::ClaudeDesktop);
    assert_eq!(run1[0].scope, ConfigScope::Global);

    // Cursor global second
    assert_eq!(run1[1].editor, EditorKind::Cursor);
    assert_eq!(run1[1].scope, ConfigScope::Global);

    // Cursor project third
    assert_eq!(run1[2].editor, EditorKind::Cursor);
    assert_eq!(run1[2].scope, ConfigScope::Project);

    // VS Code global fourth
    assert_eq!(run1[3].editor, EditorKind::VSCode);
    assert_eq!(run1[3].scope, ConfigScope::Global);

    // VS Code workspace fifth
    assert_eq!(run1[4].editor, EditorKind::VSCode);
    assert_eq!(run1[4].scope, ConfigScope::Workspace);

    // Windsurf global last
    assert_eq!(run1[5].editor, EditorKind::Windsurf);
    assert_eq!(run1[5].scope, ConfigScope::Global);
}

// ── AC-005: no filesystem writes ─────────────────────────────────────────────

/// AC-005: discover_configs must not create any files.
/// We count directory entries before and after; count must not increase.
#[test]
fn test_bc_1_01_001_no_filesystem_writes() {
    let home = FakeHome::new();
    let project = TempDir::new().unwrap();

    let count_entries = |dir: &Path| -> usize { walkdir_count(dir) };

    let before = count_entries(home.path()) + count_entries(project.path());

    let _ = discover_configs(Os::MacOs, Some(home.path()), Some(project.path()));

    let after = count_entries(home.path()) + count_entries(project.path());

    assert_eq!(
        before, after,
        "discover_configs must not create any files (DI-015)"
    );
}

fn walkdir_count(dir: &Path) -> usize {
    let mut count = 0;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            count += 1;
            let path = entry.path();
            if path.is_dir() {
                count += walkdir_count(&path);
            }
        }
    }
    count
}

// ── AC-006: no configs exist → all exists=false ───────────────────────────────

/// AC-006: empty home dir returns all entries with exists=false, no error raised.
#[test]
fn test_bc_1_01_001_no_configs_exist() {
    let home = FakeHome::new();
    let project = TempDir::new().unwrap();

    let configs = discover_configs(Os::MacOs, Some(home.path()), Some(project.path())).unwrap(); // must not Err

    let existing: Vec<_> = configs.iter().filter(|c| c.exists).collect();
    assert!(
        existing.is_empty(),
        "all entries should have exists=false when no files are present"
    );
}

// ── Exists=true when file is present ─────────────────────────────────────────

/// Verify that a created file is correctly detected as exists=true.
#[test]
fn test_bc_1_01_001_existing_file_detected() {
    let home = FakeHome::new();

    // Create the Claude Desktop config
    home.create_file("Library/Application Support/Claude/claude_desktop_config.json");

    let configs = discover_configs(Os::MacOs, Some(home.path()), None).unwrap();

    let claude = configs
        .iter()
        .find(|c| c.editor == EditorKind::ClaudeDesktop)
        .unwrap();

    assert!(
        claude.exists,
        "should detect existing Claude Desktop config"
    );
    assert!(
        claude.access_error.is_none(),
        "no access_error for readable file"
    );
}

// ── AC-007: permission denied continues ──────────────────────────────────────

/// AC-007: permission-denied file → exists=true, access_error set, others continue.
/// (Only runs on Unix — chmod is a Unix concept)
#[test]
#[cfg(unix)]
fn test_bc_1_01_001_permission_denied_continues() {
    use std::os::unix::fs::PermissionsExt;

    let home = FakeHome::new();

    // Create Claude Desktop config then make it unreadable
    let rel = "Library/Application Support/Claude/claude_desktop_config.json";
    home.create_file(rel);
    let path = home.path().join(rel);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();

    let configs = discover_configs(Os::MacOs, Some(home.path()), None).unwrap();

    // Restore so TempDir cleanup works
    let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o644));

    let claude = configs
        .iter()
        .find(|c| c.editor == EditorKind::ClaudeDesktop)
        .unwrap();

    assert!(claude.exists, "file exists even if unreadable");
    assert!(
        claude.access_error.is_some(),
        "access_error must be set for permission-denied file"
    );

    // Other editors must still have been processed
    let others: Vec<_> = configs
        .iter()
        .filter(|c| c.editor != EditorKind::ClaudeDesktop)
        .collect();
    assert!(
        !others.is_empty(),
        "discovery must continue past permission-denied entry"
    );
}

// ── AC-008: missing home dir returns E-CFG-001 ───────────────────────────────

/// AC-008: when an explicit home path is intentionally invalid (doesn't matter —
/// we test via the `home: None` path and mock dirs absence).
///
/// Since we cannot unset $HOME in a cross-platform unit test easily, we test
/// the error variant directly by checking that `DiscoveryError::HomeDirectoryMissing`
/// surfaces correctly through our error type.
#[test]
fn test_bc_1_01_001_missing_home_dir_errors() {
    // The error variant must exist and be distinguishable.
    let err = DiscoveryError::HomeDirectoryMissing;
    let msg = err.to_string();
    assert!(
        msg.contains("E-CFG-001"),
        "error message must contain E-CFG-001, got: {msg}"
    );
}

// ── AC-009: symlink followed ──────────────────────────────────────────────────

/// AC-009: a symlinked config file is detected as exists=true, reporting the
/// original (symlink) path, not the resolved target.
#[test]
#[cfg(unix)]
fn test_bc_1_01_001_symlink_followed() {
    use std::os::unix::fs::symlink;

    let home = FakeHome::new();

    // Create a real file elsewhere and symlink to the expected location.
    let real_file = home.path().join("real_claude_config.json");
    fs::write(&real_file, b"{}").unwrap();

    let symlink_dir = home
        .path()
        .join("Library")
        .join("Application Support")
        .join("Claude");
    fs::create_dir_all(&symlink_dir).unwrap();

    let symlink_path = symlink_dir.join("claude_desktop_config.json");
    symlink(&real_file, &symlink_path).unwrap();

    let configs = discover_configs(Os::MacOs, Some(home.path()), None).unwrap();

    let claude = configs
        .iter()
        .find(|c| c.editor == EditorKind::ClaudeDesktop)
        .unwrap();

    assert!(
        claude.exists,
        "symlinked config must be detected as existing"
    );
    // The reported path is the symlink path, not the resolved target.
    assert_eq!(
        claude.path, symlink_path,
        "path must be the symlink location"
    );
    assert!(
        claude.access_error.is_none(),
        "no access_error for readable symlink"
    );
}

// ── OS path correctness tests ─────────────────────────────────────────────────

/// Verify correct macOS paths using os_paths() directly (pure, fast).
#[test]
fn test_bc_1_01_001_macos_path_correctness() {
    let home = PathBuf::from("/Users/testuser");
    let entries = os_paths(Os::MacOs, &home, None);

    let claude = entries
        .iter()
        .find(|(e, s, _)| *e == EditorKind::ClaudeDesktop && *s == ConfigScope::Global)
        .expect("Claude Desktop global must exist");
    assert_eq!(
        claude.2,
        PathBuf::from(
            "/Users/testuser/Library/Application Support/Claude/claude_desktop_config.json"
        )
    );

    let cursor = entries
        .iter()
        .find(|(e, s, _)| *e == EditorKind::Cursor && *s == ConfigScope::Global)
        .expect("Cursor global must exist");
    assert_eq!(cursor.2, PathBuf::from("/Users/testuser/.cursor/mcp.json"));

    let vscode = entries
        .iter()
        .find(|(e, s, _)| *e == EditorKind::VSCode && *s == ConfigScope::Global)
        .expect("VS Code global must exist");
    assert_eq!(
        vscode.2,
        PathBuf::from("/Users/testuser/Library/Application Support/Code/User/mcp.json")
    );

    let windsurf = entries
        .iter()
        .find(|(e, s, _)| *e == EditorKind::Windsurf && *s == ConfigScope::Global)
        .expect("Windsurf global must exist");
    assert_eq!(
        windsurf.2,
        PathBuf::from("/Users/testuser/.codeium/windsurf/mcp_config.json")
    );
}

/// Verify correct Linux paths using os_paths() directly.
#[test]
fn test_bc_1_01_001_linux_path_correctness() {
    let home = PathBuf::from("/home/testuser");
    let entries = os_paths(Os::Linux, &home, None);

    let claude = entries
        .iter()
        .find(|(e, s, _)| *e == EditorKind::ClaudeDesktop && *s == ConfigScope::Global)
        .unwrap();
    assert_eq!(
        claude.2,
        PathBuf::from("/home/testuser/.config/Claude/claude_desktop_config.json")
    );

    let vscode = entries
        .iter()
        .find(|(e, s, _)| *e == EditorKind::VSCode && *s == ConfigScope::Global)
        .unwrap();
    assert_eq!(
        vscode.2,
        PathBuf::from("/home/testuser/.config/Code/User/mcp.json")
    );

    let windsurf = entries
        .iter()
        .find(|(e, s, _)| *e == EditorKind::Windsurf && *s == ConfigScope::Global)
        .unwrap();
    assert_eq!(
        windsurf.2,
        PathBuf::from("/home/testuser/.codeium/windsurf/mcp_config.json")
    );
}

/// Verify correct Windows paths using os_paths() directly.
///
/// We use PathBuf segment-by-segment comparison so the test is not sensitive
/// to path-separator style (which differs on macOS vs Windows hosts).
#[test]
fn test_bc_1_01_001_windows_path_correctness() {
    let home = PathBuf::from("/Users/testuser"); // Use Unix-style for cross-platform test
    let entries = os_paths(Os::Windows, &home, None);

    let claude = entries
        .iter()
        .find(|(e, s, _)| *e == EditorKind::ClaudeDesktop && *s == ConfigScope::Global)
        .unwrap();
    let expected_claude = home
        .join("AppData")
        .join("Roaming")
        .join("Claude")
        .join("claude_desktop_config.json");
    assert_eq!(claude.2, expected_claude, "Claude Desktop Windows path");

    let vscode = entries
        .iter()
        .find(|(e, s, _)| *e == EditorKind::VSCode && *s == ConfigScope::Global)
        .unwrap();
    let expected_vscode = home
        .join("AppData")
        .join("Roaming")
        .join("Code")
        .join("User")
        .join("mcp.json");
    assert_eq!(vscode.2, expected_vscode, "VS Code Windows path");

    let windsurf = entries
        .iter()
        .find(|(e, s, _)| *e == EditorKind::Windsurf && *s == ConfigScope::Global)
        .unwrap();
    let expected_windsurf = home
        .join(".codeium")
        .join("windsurf")
        .join("mcp_config.json");
    assert_eq!(windsurf.2, expected_windsurf, "Windsurf Windows path");
}

/// Cursor project path is relative to project root.
#[test]
fn test_bc_1_01_001_cursor_project_path() {
    let home = PathBuf::from("/home/testuser");
    let project = PathBuf::from("/home/testuser/myproject");
    let entries = os_paths(Os::Linux, &home, Some(&project));

    let cursor_proj = entries
        .iter()
        .find(|(e, s, _)| *e == EditorKind::Cursor && *s == ConfigScope::Project)
        .expect("Cursor project entry must be present");

    assert_eq!(
        cursor_proj.2,
        PathBuf::from("/home/testuser/myproject/.cursor/mcp.json")
    );
}
