//! # forge-mcp
//!
//! Forge MCP — CLI for discovering, inspecting, debugging, monitoring, and
//! security-auditing MCP servers.
//!
//! This is the L4 binary entry point. It wires all subsystem crates together
//! and exposes them through a unified command-line interface.

use clap::{Parser, Subcommand};

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

#[derive(Subcommand, Debug)]
enum Commands {
    /// Discover MCP servers on this machine and local network
    Discover {
        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },
    /// Inspect a specific MCP server (tools, resources, prompts)
    Inspect {
        /// MCP server URI (e.g. stdio://path/to/server or http://localhost:3000)
        server: String,
        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },
    /// Capture and display live MCP traffic
    Traffic {
        /// MCP server URI to monitor
        server: String,
    },
    /// Run health checks against an MCP server
    Health {
        /// MCP server URI to check
        server: String,
    },
    /// Run security audit against an MCP server
    Audit {
        /// MCP server URI to audit
        server: String,
    },
    /// Run MCP conformance test suite against a server
    Conform {
        /// MCP server URI to test
        server: String,
    },
    /// Start the background monitoring daemon
    Daemon {
        /// Detach and run in background
        #[arg(long)]
        detach: bool,
    },
    /// Launch the interactive terminal UI
    Tui,
}

fn main() {
    let cli = Cli::parse();

    // Initialise tracing based on verbosity flag
    let level = match cli.verbose {
        0 => tracing::Level::WARN,
        1 => tracing::Level::DEBUG,
        _ => tracing::Level::TRACE,
    };
    tracing_subscriber::fmt()
        .with_max_level(level)
        .with_target(false)
        .init();

    match cli.command {
        None => {
            // No subcommand — print help and exit 0
            use clap::CommandFactory;
            Cli::command().print_help().expect("failed to print help");
            println!();
        }
        Some(cmd) => {
            tracing::debug!(?cmd, "dispatching subcommand");
            eprintln!(
                "forge-mcp: subcommand not yet implemented — \
                 this scaffold will be fleshed out in subsequent stories."
            );
            std::process::exit(1);
        }
    }
}
