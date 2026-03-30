//! # forge-discovery
//!
//! MCP server discovery for Forge MCP.
//!
//! This is an L2 crate — it depends on `forge-core` (L1) and `forge-config` (L0).
//!
//! ## Responsibilities
//! - Scan filesystem for MCP server configurations (Claude Desktop, Cursor, VS Code, Windsurf)
//! - Return a deterministically ordered list of [`DiscoveredConfig`] entries
//! - Handle missing files, permission errors, and symlinks gracefully
//! - Parse discovered config files into unified [`ServerEntry`] records
//!
//! ## Module layout
//! | Module | Pure? | Responsibility |
//! |--------|-------|---------------|
//! | `types` | ✓ | Data types: `EditorKind`, `ConfigScope`, `DiscoveredConfig` |
//! | `paths` | ✓ | OS path tables; `os_paths()` |
//! | `discovery` | ✗ | Effectful; `discover_configs()` |
//! | `error` | ✓ | `DiscoveryError`, `ConfigError` enums |
//! | `parser` | ✓ | `parse_config()` pure function |

pub mod discovery;
pub mod error;
pub mod parser;
pub mod paths;
pub mod types;

// Re-export the most commonly used items at the crate root.
pub use discovery::discover_configs;
pub use error::{ConfigError, DiscoveryError};
pub use parser::{parse_config, ParseInput};
pub use paths::Os;
pub use types::{ConfigScope, DiscoveredConfig, EditorKind};
