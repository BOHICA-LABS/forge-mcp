---
document_type: architecture-section
level: L3
section: system-overview
version: "1.0"
status: draft
producer: architect
timestamp: 2026-03-29T14:05:00
phase: 1b
inputs: [domain-spec/L2-INDEX.md, specs/prd.md]
traces_to: ARCH-INDEX.md
---

# System Overview

## Deployment

Single static Rust binary. No runtime dependencies. Targets: linux-x64, linux-arm64, macos-x64, macos-arm64, windows-x64. Binary < 25MB stripped with LTO.

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        forge-mcp binary                         │
├──────────┬──────────┬───────────────────────────────────────────┤
│  CLI     │  TUI     │  Daemon                                   │
│ (clap)   │(ratatui) │ (tokio, Unix socket / named pipe)         │
├──────────┴──────────┴───────────────────────────────────────────┤
│                    Application Layer                             │
│  ┌───────────┐ ┌──────────┐ ┌──────────┐ ┌───────────────────┐ │
│  │ Discovery │ │ Traffic  │ │  Health  │ │     Security      │ │
│  │ & Config  │ │ Capture  │ │ Metrics  │ │    Auditing       │ │
│  └─────┬─────┘ └────┬─────┘ └────┬─────┘ └────────┬──────────┘ │
│        │            │            │                 │            │
│  ┌─────┴────────────┴────────────┴─────────────────┴──────────┐ │
│  │              Protocol Adapter (thin rmcp wrapper)           │ │
│  └─────────────────────────┬───────────────────────────────────┘ │
│                            │                                     │
│  ┌─────────────────────────┴───────────────────────────────────┐ │
│  │                    rmcp SDK (transport)                      │ │
│  │            stdio | Streamable HTTP | TCP                     │ │
│  └─────────────────────────────────────────────────────────────┘ │
├─────────────────────────────────────────────────────────────────┤
│  ┌────────────┐ ┌──────────────┐ ┌───────────┐ ┌────────────┐  │
│  │Conformance │ │ Config Drift │ │  Server   │ │  Output    │  │
│  │  Testing   │ │  Detection   │ │ Comparison│ │ Formatting │  │
│  └────────────┘ └──────────────┘ └───────────┘ └────────────┘  │
└─────────────────────────────────────────────────────────────────┘
          │                    │                    │
          ▼                    ▼                    ▼
    MCP Server A         MCP Server B         MCP Server N
```

## Key Architectural Principles

1. **Layered architecture** — Presentation (CLI/TUI) → Application (subsystems) → Protocol Adapter → rmcp SDK
2. **Event-driven data flow** — MessageCaptured events propagate to metrics, security, TUI via async channels
3. **Pure core / effectful shell** — Each subsystem separates deterministic logic from I/O
4. **Single binary** — All subsystems compile into one binary; CLI subcommands select the entry point
5. **Lazy initialization** — Heavy subsystems (TUI, daemon, security rules) load only when invoked

## Cargo Workspace Structure

```
forge-mcp/
├── Cargo.toml          (workspace root)
├── crates/
│   ├── forge-mcp/      (binary crate — thin main, clap CLI)
│   ├── forge-core/     (protocol adapter, shared types)
│   ├── forge-discovery/ (config parsing, server registry)
│   ├── forge-tui/      (ratatui dashboard)
│   ├── forge-traffic/  (capture, filter, replay)
│   ├── forge-health/   (metrics, alerting)
│   ├── forge-security/ (rules engine, report generation)
│   ├── forge-conformance/ (test suite, assertions)
│   ├── forge-config/   (drift detection, comparison)
│   └── forge-daemon/   (session pool, connection management)
└── tests/              (integration tests)
```

## Runtime Model

The binary has three entry modes selected by CLI subcommand:

- **CLI mode** (default): One-shot command execution. Parse args → connect → execute → output → exit. No persistent state.
- **TUI mode** (`forge tui`): Interactive dashboard. Starts Tokio runtime, connects to daemon or directly to servers, renders ratatui frames in a loop.
- **Daemon mode** (`forge daemon`): Background process. Listens on Unix socket (or Windows named pipe), pools MCP sessions, responds to CLI/TUI requests.

All modes share the same Tokio async runtime. The daemon is optional — CLI and TUI can connect directly to MCP servers without it.
