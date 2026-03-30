//! # forge-mcp
//!
//! Forge MCP — CLI for discovering, inspecting, debugging, monitoring, and
//! security-auditing MCP servers.
//!
//! This is the L4 binary entry point. It wires all subsystem crates together
//! and exposes them through a unified command-line interface.
//!
//! **Performance (NFR-001):** clap parses arguments before any subsystem is
//! initialised.  `--help` must complete in < 50 ms cold.

mod commands;
mod exit_codes;

use clap::Parser;
use commands::Commands;
use exit_codes::exit_code_for_error;

/// Forge MCP — discover, inspect, and audit MCP servers.
#[derive(Parser, Debug)]
#[command(
    name = "forge-mcp",
    version,
    about = "Discover, inspect, debug, monitor, and security-audit MCP servers.",
    long_about = None,
)]
struct Cli {
    /// Increase verbosity (-v = debug, -vv = trace)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

fn main() {
    // ── 1. Parse args (fast path — no subsystem init) ───────────────────────
    let cli = Cli::parse();

    // ── 2. Initialise tracing after parse so --help stays sub-1ms ──────────
    let level = match cli.verbose {
        0 => tracing::Level::WARN,
        1 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .init();

    // ── 3. Dispatch ─────────────────────────────────────────────────────────
    let result = match &cli.command {
        None => {
            // No subcommand — print help and exit 0.
            use clap::CommandFactory;
            Cli::command().print_help().expect("failed to print help");
            println!();
            return;
        }
        Some(cmd) => {
            tracing::debug!(?cmd, "dispatching subcommand");
            commands::dispatch(cmd)
        }
    };

    // ── 4. Map errors → exit codes ──────────────────────────────────────────
    if let Err(err) = result {
        eprintln!("forge-mcp: error: {err}");
        std::process::exit(exit_code_for_error(&err));
    }
}
