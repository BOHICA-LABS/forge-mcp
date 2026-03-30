//! Subcommand definitions and dispatch stubs for forge-mcp.
//!
//! Each subcommand is declared as a clap-derive struct/enum. The handler
//! functions are stubs that will be fleshed out in subsequent stories
//! (STORY-024 → STORY-026 etc.).
//!
//! **Performance constraint (NFR-001):** No subsystem crates are initialised
//! at argument-parse time. All heavy initialisation (TUI, daemon socket,
//! connection pool) happens inside the handler functions, which are only
//! called after successful parse.

use clap::Subcommand;

use crate::exit_codes::CliError;

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

// ── Handler stubs ─────────────────────────────────────────────────────────────
//
// Each handler prints a placeholder message and returns `Ok(())`.
// Subsequent stories will replace these stubs with real implementations.

pub fn handle_list(_args: &ListArgs) -> Result<(), CliError> {
    println!("[forge-mcp list] — not yet implemented (STORY-024)");
    Ok(())
}

pub fn handle_call(_args: &CallArgs) -> Result<(), CliError> {
    println!("[forge-mcp call] — not yet implemented (STORY-025)");
    Ok(())
}

pub fn handle_info(_args: &InfoArgs) -> Result<(), CliError> {
    println!("[forge-mcp info] — not yet implemented (STORY-024)");
    Ok(())
}

pub fn handle_grep(_args: &GrepArgs) -> Result<(), CliError> {
    println!("[forge-mcp grep] — not yet implemented (STORY-024)");
    Ok(())
}

pub fn handle_test(_args: &TestArgs) -> Result<(), CliError> {
    println!("[forge-mcp test] — not yet implemented (STORY-026)");
    Ok(())
}

pub fn handle_audit(_args: &AuditArgs) -> Result<(), CliError> {
    println!("[forge-mcp audit] — not yet implemented (STORY-026)");
    Ok(())
}

pub fn handle_tui(_args: &TuiArgs) -> Result<(), CliError> {
    println!("[forge-mcp tui] — not yet implemented (STORY-027)");
    Ok(())
}

pub fn handle_daemon(_args: &DaemonArgs) -> Result<(), CliError> {
    println!("[forge-mcp daemon] — not yet implemented (STORY-010)");
    Ok(())
}

/// Dispatch a parsed [`Commands`] variant to its handler.
///
/// Returns `Ok(())` on success or a [`CliError`] that the caller maps to
/// an exit code via [`crate::exit_codes::exit_code_for_error`].
pub fn dispatch(cmd: &Commands) -> Result<(), CliError> {
    match cmd {
        Commands::List(args) => handle_list(args),
        Commands::Call(args) => handle_call(args),
        Commands::Info(args) => handle_info(args),
        Commands::Grep(args) => handle_grep(args),
        Commands::Test(args) => handle_test(args),
        Commands::Audit(args) => handle_audit(args),
        Commands::Tui(args) => handle_tui(args),
        Commands::Daemon(args) => handle_daemon(args),
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

    // Dispatch succeeds (returns Ok) for all stub handlers.
    #[test]
    fn dispatch_returns_ok_for_all_stubs() {
        let cmds = [
            parse(&["forge-mcp", "list"]),
            parse(&["forge-mcp", "call", "stdio://s", "t"]),
            parse(&["forge-mcp", "info", "stdio://s"]),
            parse(&["forge-mcp", "grep", "pat"]),
            parse(&["forge-mcp", "test", "stdio://s"]),
            parse(&["forge-mcp", "audit", "stdio://s"]),
            parse(&["forge-mcp", "tui"]),
            parse(&["forge-mcp", "daemon", "start"]),
        ];
        for cmd in &cmds {
            assert!(dispatch(cmd).is_ok(), "dispatch returned Err for {:?}", cmd);
        }
    }
}
