//! Effectful config-file discovery.
//!
//! `discover_configs` is the only public entry point.  It:
//! 1. Resolves the home directory (once, fails fast with `E-CFG-001` if missing).
//! 2. Calls the pure `os_paths` table to get expected paths.
//! 3. Probes each path for existence and readability.
//! 4. Returns a deterministically ordered `Vec<DiscoveredConfig>`.
//!
//! This function is **read-only** (invariant DI-015): it never creates,
//! modifies, or deletes any file.

use std::path::{Path, PathBuf};

use crate::{
    error::DiscoveryError,
    paths::{Os, make_stubs, os_paths},
    types::DiscoveredConfig,
};

/// Discover MCP configuration files for all supported editors.
///
/// # Parameters
/// * `os`           – target OS; pass `Os::current()` for the running system.
/// * `home`         – explicit home directory; pass `None` to auto-detect via
///   `dirs::home_dir()`.
/// * `project_root` – optional project root for project/workspace-scoped
///   config files.  When `None`, those entries are omitted.
///
/// # Errors
/// Returns `DiscoveryError::HomeDirectoryMissing` (`E-CFG-001`) when
/// `home` is `None` **and** `dirs::home_dir()` returns `None`.
///
/// # Guarantees
/// * Deterministic order across repeated invocations.
/// * Never panics; permission denied and unreadable paths are captured in
///   `DiscoveredConfig::access_error`.
/// * No filesystem writes (invariant DI-015).
pub fn discover_configs(
    os: Os,
    home: Option<&Path>,
    project_root: Option<&Path>,
) -> Result<Vec<DiscoveredConfig>, DiscoveryError> {
    // Resolve home directory ─────────────────────────────────────────────────
    let home_buf: PathBuf;
    let home: &Path = match home {
        Some(h) => h,
        None => {
            home_buf =
                dirs::home_dir().ok_or(DiscoveryError::HomeDirectoryMissing)?;
            &home_buf
        }
    };

    // Build path table (pure) ────────────────────────────────────────────────
    let entries = os_paths(os, home, project_root);
    let mut configs = make_stubs(entries);

    // Probe filesystem (effectful, read-only) ────────────────────────────────
    for cfg in &mut configs {
        probe(&cfg.path, &mut cfg.exists, &mut cfg.access_error);
    }

    Ok(configs)
}

/// Probe a single path.
///
/// Uses `std::fs::symlink_metadata` (does **not** follow symlinks for the
/// existence check itself, but reports the symlink path as-is per AC-009).
/// If the symlink target is unreadable we capture the error in `access_error`.
fn probe(path: &Path, exists: &mut bool, access_error: &mut Option<String>) {
    match std::fs::symlink_metadata(path) {
        Ok(_meta) => {
            *exists = true;
            // Try to open the file (or follow symlink if it is one) to detect
            // permission-denied errors.
            if let Err(e) = std::fs::File::open(path) {
                *access_error = Some(e.to_string());
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            // Path does not exist — leave exists=false, no error.
        }
        Err(e) => {
            // Some other error (e.g. permission denied on parent directory).
            *exists = true;
            *access_error = Some(e.to_string());
        }
    }
}
