# forge-mcp

> **Discover, inspect, debug, monitor, and security-audit MCP servers.**

[![CI](https://github.com/BOHICA-LABS/forge-mcp/actions/workflows/ci.yml/badge.svg)](https://github.com/BOHICA-LABS/forge-mcp/actions/workflows/ci.yml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE-MIT)

`forge-mcp` is a Rust CLI tool for working with [Model Context Protocol (MCP)](https://modelcontextprotocol.io) servers. It gives you a unified interface to discover servers on your machine, inspect their capabilities, capture live traffic, run health checks, audit for security issues, and verify protocol conformance.

## Features

| Command | Description |
|---------|-------------|
| `forge-mcp discover` | Scan filesystem & network for MCP servers |
| `forge-mcp inspect <server>` | List tools, resources, and prompts |
| `forge-mcp traffic <server>` | Capture and display live MCP JSON-RPC traffic |
| `forge-mcp health <server>` | Run health checks and latency analysis |
| `forge-mcp audit <server>` | Security audit: prompt injection, scope violations |
| `forge-mcp conform <server>` | MCP specification conformance test suite |
| `forge-mcp daemon` | Background monitoring daemon |
| `forge-mcp tui` | Interactive terminal UI dashboard |

## Installation

```bash
cargo install forge-mcp
```

Or build from source:

```bash
git clone https://github.com/BOHICA-LABS/forge-mcp
cd forge-mcp
cargo build --release
./target/release/forge-mcp --help
```

## Workspace Structure

```
forge-mcp/
├── crates/
│   ├── forge-config/       # L0 — configuration management
│   ├── forge-core/         # L1 — MCP protocol client (wraps rmcp)
│   ├── forge-traffic/      # L1 — traffic capture & replay
│   ├── forge-discovery/    # L2 — server discovery
│   ├── forge-health/       # L2 — health monitoring
│   ├── forge-security/     # L2 — security auditing
│   ├── forge-daemon/       # L3 — background daemon
│   ├── forge-conformance/  # L3 — conformance testing
│   ├── forge-tui/          # L4 — terminal UI
│   └── forge-mcp/          # L4 — binary entry point
```

## Development

```bash
# Run all checks
cargo check --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings
cargo fmt --check

# Build release binary
cargo build --release -p forge-mcp
```

## License

Licensed under either of:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

at your option.

---

*Built with the [Dark Factory](https://github.com/BOHICA-LABS/dark-factory) pipeline.*
