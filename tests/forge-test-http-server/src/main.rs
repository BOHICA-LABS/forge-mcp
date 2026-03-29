mod config;
mod server;
#[cfg(test)]
mod tests;

use config::HttpMockConfig;
use server::MockHttpServer;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    // Load config from environment variable (JSON-encoded), or use defaults
    let config: HttpMockConfig = if let Ok(cfg_json) = std::env::var("MOCK_HTTP_CONFIG") {
        serde_json::from_str(&cfg_json)?
    } else {
        HttpMockConfig::default()
    };

    let ct = CancellationToken::new();

    let (addr, _handle) = MockHttpServer::start(config, ct.clone()).await?;

    // Print the bound port to stdout — test harnesses read this line
    println!("{}", addr.port());
    // Ensure stdout is flushed before we block
    use std::io::Write;
    std::io::stdout().flush()?;

    // Run until SIGINT / SIGTERM (or cancellation)
    tokio::select! {
        _ = tokio::signal::ctrl_c() => {}
        _ = ct.cancelled() => {}
    }

    Ok(())
}
