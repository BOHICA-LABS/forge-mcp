//! forge-test-server: configurable mock MCP server for integration tests.
//!
//! Communicates via stdio using newline-delimited JSON-RPC.

mod config;
mod handler;

use std::path::PathBuf;

use clap::Parser;
use config::MockConfig;
use handler::MockServer;
use rmcp::transport::stdio;
use rmcp::ServiceExt;

#[derive(Parser, Debug)]
#[command(name = "forge-test-server")]
#[command(about = "Configurable mock MCP server for forge-mcp integration tests")]
struct Cli {
    /// Path to JSON configuration file.
    #[arg(long)]
    config: Option<PathBuf>,

    /// Number of tools to advertise (quick default — overridden by --config).
    #[arg(long, default_value_t = 3)]
    tools: usize,

    /// Number of resources to advertise (quick default).
    #[arg(long, default_value_t = 2)]
    resources: usize,

    /// Number of prompts to advertise (quick default).
    #[arg(long, default_value_t = 1)]
    prompts: usize,

    /// Pagination page size (0 = no pagination).
    #[arg(long, default_value_t = 0)]
    page_size: usize,

    /// Enable loop cursor mode for testing client loop-detection.
    #[arg(long, default_value_t = false)]
    loop_cursor: bool,

    /// Milliseconds to delay every response (error injection).
    #[arg(long, default_value_t = 0)]
    delay_ms: u64,

    /// Crash (exit 1) when a tool is called.
    #[arg(long, default_value_t = false)]
    crash_on_tool_call: bool,

    /// Return isError=true from tool calls.
    #[arg(long, default_value_t = false)]
    error_tool_result: bool,

    /// Emit a malformed (non-JSON) line and exit immediately.
    #[arg(long, default_value_t = false)]
    emit_malformed: bool,

    /// Drift tool schema after N tool/list calls (0 = no drift).
    #[arg(long, default_value_t = 0)]
    drift_after: usize,
}

#[tokio::main]
async fn main() {
    // Log to stderr so it doesn't corrupt the stdio JSON-RPC stream.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::WARN.into()),
        )
        .init();

    let cli = Cli::parse();

    // If --emit-malformed: write garbage and exit — used by error-injection tests.
    if cli.emit_malformed {
        println!("{{this is not valid json}}");
        std::process::exit(0);
    }

    let mock_config = if let Some(path) = cli.config {
        let data = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Cannot read config file {}: {e}", path.display()));
        serde_json::from_str::<MockConfig>(&data)
            .unwrap_or_else(|e| panic!("Invalid config file {}: {e}", path.display()))
    } else {
        let mut cfg = MockConfig::from_counts(cli.tools, cli.resources, cli.prompts);
        cfg.pagination.page_size = cli.page_size;
        cfg.pagination.loop_cursor = cli.loop_cursor;
        cfg.error_injection.delay_ms = cli.delay_ms;
        cfg.error_injection.crash_on_tool_call = cli.crash_on_tool_call;
        cfg.error_injection.error_tool_result = cli.error_tool_result;
        cfg.error_injection.emit_malformed = cli.emit_malformed;
        if cli.drift_after > 0 {
            cfg.drift_config = Some(config::DriftConfig {
                drift_after_n_calls: cli.drift_after,
            });
        }
        cfg
    };

    let server = MockServer::new(mock_config);

    let (stdin, stdout) = stdio();
    match server.serve((stdin, stdout)).await {
        Ok(running) => {
            // Wait for the client to disconnect.
            running
                .waiting()
                .await
                .inspect_err(|e| tracing::warn!("Server error: {e:?}"))
                .ok();
        }
        Err(e) => {
            tracing::warn!("Server failed to initialize: {e:?}");
        }
    }
}
