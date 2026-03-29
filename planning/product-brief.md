---
document_type: product-brief
level: L1
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-03-29T00:56:00
phase: 1a
inputs: ["seed-brief (human-provided)"]
input-hash: ""
traces_to: ""
---

# Product Brief: Forge MCP

## What Is This?

Forge MCP is a Rust-based command-line tool that gives AI platform teams a single
binary for discovering, inspecting, debugging, monitoring, and security-auditing
MCP (Model Context Protocol) servers. It combines a scriptable CLI, an interactive
TUI dashboard, and a protocol conformance suite — replacing the fragmented collection
of 6+ single-purpose MCP tools that each cover only a sliver of the workflow.

## Who Is It For?

Forge MCP targets professionals who operate, build, or secure MCP server
infrastructure. These users share a common pain: the gap between "MCP server is
running" and "MCP server is correct, healthy, and safe" is filled by guesswork,
raw logs, and manual JSON wrangling.

| Persona | Pain Point | Current Workaround |
|---------|-----------|-------------------|
| **AI platform engineers** at companies deploying MCP-connected AI products | Cannot see inside MCP connections — traffic, errors, latency, and failure modes are opaque | Tail raw JSON-RPC logs, write ad-hoc curl scripts, correlate timestamps manually across disconnected tools |
| **DevEx engineers** managing editor/IDE MCP configurations across teams | MCP config JSON drifts silently between Claude Desktop, Cursor, VS Code, and Windsurf installations | Copy-paste JSON between config files, manually diff when something breaks, no automated drift detection |
| **AI agent developers** building with Claude Code, Gemini CLI, or custom agents | Need lightweight tool discovery that doesn't bloat agent context; Node.js CLIs add ~300ms startup and high token overhead | Use heavy Node.js inspector tools, accept the latency/token cost, or skip discovery and hardcode tool names |
| **Security teams** evaluating MCP servers before production deployment | Static scanners (mcpsec, ramparts) analyze source code but miss runtime behavioral issues — permission escalation, dangerous tool patterns, auth handling gaps | Skip the runtime audit, or do expensive manual code review and ad-hoc live testing |
| **MCP server authors** validating spec compliance of their implementations | No automated conformance suite exists; the official inspector is interactive-only and doesn't produce CI/CD-friendly results | Test manually against the official inspector, write custom integration tests, or ship and hope |

## Scope

### In Scope

1. **Server discovery and connection management** — auto-import and parse MCP
   server configurations from Claude Desktop, Cursor, VS Code, and Windsurf;
   connect to servers via stdio and Streamable HTTP transports; maintain named
   persistent sessions with daemon-mode connection pooling for concurrent access.

2. **Full MCP 2025-11-25 protocol coverage** — first-class support for all 10
   spec capabilities: tools, resources, resource templates, prompts, sampling
   proxy, elicitation (form + URL mode), roots, logging, completions, tasks,
   and the extensions system. Graceful capability negotiation with older servers.

3. **Interactive TUI dashboard** — multi-pane ratatui terminal UI providing a
   server browser, capability explorer (tools/resources/prompts), JSON-RPC
   traffic inspector with syntax highlighting, and real-time health metrics
   panel with sparklines and latency histograms. Keyboard-first (vi-style hjkl)
   with mouse as supplementary input. Responsive from 80×24 to ultra-wide.

4. **Protocol-level traffic inspection** — capture, filter, search, and replay
   all JSON-RPC messages between client and server, with per-message timing
   analysis and syntax-highlighted display. This is the "Wireshark for MCP"
   capability that no existing tool provides.

5. **Non-interactive CLI mode** — scriptable subcommands (`list`, `call`, `info`,
   `grep`, `test`) with structured JSON output for piping, designed for AI agent
   consumption with < 500 token overhead for the full discover → inspect → call
   workflow.

6. **Server health monitoring** — continuous collection of latency histograms,
   error rate trends, throughput counters, and connection status over time, with
   configurable alerting thresholds. Enables proactive detection of degraded
   servers before they impact production AI workflows.

7. **Runtime security auditing** — live behavioral analysis layered on top of
   traffic inspection: flag dangerous tool patterns (file system access,
   code execution, network calls), detect permission escalation attempts,
   verify root enforcement, validate auth handling, and generate structured
   compliance reports. Complements static scanners; focuses on what servers
   actually *do* at runtime.

8. **Protocol conformance testing** — automated spec compliance suite runnable
   against any MCP server, validating capability negotiation, error handling,
   transport compliance, and method coverage against MCP 2025-11-25. CI/CD-friendly
   structured output (JSON, JUnit XML) for integration into build pipelines.

9. **Config sync and drift detection** — detect configuration differences across
   editor/IDE MCP configs, report discrepancies with actionable detail, and
   support reconciliation workflows. Prevents the "works on my machine" class
   of MCP config bugs.

10. **Server comparison and diff** — side-by-side comparison of two MCP servers:
    tool schema diff, capability differences, and response behavior delta. Useful
    for migration validation, version upgrade verification, and debugging
    staging-vs-production discrepancies.

### Out of Scope

- **Not an MCP server** — Forge MCP is a client/inspector only. It connects to
  servers but does not serve MCP capabilities itself.
- **No embedded LLM** — sampling requests are proxied to external LLM APIs.
  Forge MCP never runs inference locally.
- **No web UI or Electron** — terminal-native only. The TUI is the richest
  interface; no browser-based dashboard.
- **No MCP registry or marketplace** — Forge MCP discovers locally configured
  servers, not a remote catalog.
- **No payment protocol (x402)** — out of scope for the inspection/debugging
  mission.
- **No static source-code scanning** — tools like mcpsec and ramparts cover
  static analysis. Forge MCP focuses on runtime behavioral analysis.
- **No MCP server scaffolding or codegen** — mcporter covers config-to-code
  generation. Forge MCP inspects existing servers.

## Success Criteria

| Outcome | Metric | Target |
|---------|--------|--------|
| CLI startup performance | Cold start to first output (no daemon) | < 50ms |
| Agent context efficiency | Total tokens for discover → inspect → call workflow | < 500 tokens |
| Protocol completeness | MCP 2025-11-25 spec capabilities covered | 10/10 |
| Cross-platform distribution | Zero-dependency static binary targets | 5 (Linux x64, Linux ARM64, macOS Intel, macOS ARM64, Windows x64) |
| TUI rendering performance | Sustained frame rate at 100 events/sec ingest | ≥ 60fps with < 5MB RSS |
| Conformance suite coverage | Percentage of MCP 2025-11-25 spec methods exercised | ≥ 90% |
| Security audit coverage | Percentage of OWASP MCP top-10 risk categories covered by runtime checks | ≥ 80% |
| Binary size | Stripped binary with LTO | < 25MB |

## Constraints & Integration Points

### Hard Technical Constraints

- **Language:** Rust (2024 edition). Static binaries. Zero runtime dependencies
  (no Python, Node.js, or system library requirements on target machines).
- **MCP SDK:** Must use the `rmcp` crate v1.3+ (official Rust MCP SDK). Protocol
  handling must NOT be reimplemented — this is a hard constraint to ensure spec
  compliance and reduce maintenance burden. Rolling custom protocol code is the
  primary risk in existing Rust MCP tools.
- **TUI framework:** `ratatui` v0.30+ with `crossterm` backend. No ncurses
  dependency.
- **Async runtime:** `tokio` (required by rmcp).
- **CLI framework:** `clap` with derive macros.
- **MCP spec version:** 2025-11-25. Must gracefully negotiate capabilities with
  servers implementing older spec versions (2024-11-05).

### Integration Points

- **Editor config formats:** Must natively parse MCP server configuration from
  Claude Desktop (`claude_desktop_config.json`), Cursor
  (`~/.cursor/mcp.json`), VS Code (`settings.json` MCP section), and Windsurf
  config files. These formats differ in structure and location; Forge MCP must
  handle all four without requiring user transformation.
- **CI/CD systems:** Conformance test output must be consumable by standard CI
  runners — JSON for programmatic consumption, JUnit XML for test result
  integration (GitHub Actions, GitLab CI, Jenkins).
- **Unix philosophy:** CLI subcommands must produce clean, pipeable output.
  JSON output on stdout, human-readable diagnostics on stderr. Exit codes
  follow conventional semantics (0 = success, 1 = test failure, 2 = connection
  error, etc.).

## Overflow Context (Optional — Reference Only)

### Competitive Landscape Detail

The MCP CLI ecosystem as of early 2026 has 6+ tools, each covering a narrow
slice of the workflow:

| Tool | Language | Stars | Key Strength | Key Gap |
|------|----------|-------|-------------|---------|
| **mcporter** | TypeScript | ~3.2K | Config discovery + server codegen | No TUI, no monitoring, no security |
| **mcpc** | TypeScript | — | Broadest MCP coverage, OAuth, sessions | No TUI, no monitoring, heavy runtime |
| **mcp-cli** | Bun | — | Simplest agent-first CLI, glob search | Tools-only, no resources/prompts/sampling |
| **mcp-probe** | Rust | — | TUI debugger | Rolls own protocol (not rmcp), spec risk |
| **mcpeek** | Rust | — | Simple TUI inspector | MCP 2024-11-05 only, no 2025-11-25 |
| **mcpsec** | Rust | — | Static security scanner | No runtime analysis |
| **ramparts** | Rust | — | YARA + LLM static analysis | No runtime analysis |
| **@modelcontextprotocol/inspector** | TypeScript | — | Official reference inspector | Interactive-only, not CI-friendly, heavy |

**Forge MCP's unique position:** The only tool that combines the official rmcp
SDK (spec compliance guarantee) + TUI dashboard + traffic inspection + health
monitoring + runtime security auditing + conformance testing in a single
zero-dependency binary. Every competitor covers at most 2-3 of these six
dimensions.

### Prioritized Release Plan

| Release | Priority | Capabilities | Target |
|---------|----------|-------------|--------|
| **v0.1.0** | P0 (Core) | Server discovery, full protocol coverage, TUI dashboard, traffic inspection, CLI mode, health monitoring, runtime security | MVP — covers the complete inspect/debug/monitor/audit loop |
| **v0.2.0** | P1 | Protocol conformance testing, config drift detection | CI/CD integration — enables automated quality gates |
| **v0.3.0** | P2 | Server comparison & diff | Advanced workflows — migration and upgrade validation |

### TUI Design Direction

The TUI is the signature differentiator and warrants careful UX treatment:

- **Color system:** Support 16-color, 256-color, and truecolor terminals with
  auto-detection. Dark and light terminal background awareness.
- **Layout:** Character-cell grid with consistent spacing. Multi-pane layout
  that adapts from 80×24 minimum to ultra-wide terminals.
- **Navigation:** Vi-style keybindings (hjkl for movement, / for search, :
  for commands) with a discoverable shortcut bar for discoverability.
- **Accessibility:** No color-only indicators (always pair with shape/text).
  Full keyboard-only operation. Proper focus management across panes.
- **Rendering:** Unicode box drawing characters with ASCII fallback for
  limited terminals.
- **Widget contracts:** Tables, lists, panels, charts, status badges, JSON
  viewer with syntax highlighting, sparklines for time-series metrics.

### Daemon Architecture Note

Connection pooling via a background daemon process enables:
- Persistent sessions that survive CLI invocations
- Shared connections across concurrent TUI and CLI usage
- Warm startup for repeated agent interactions (sub-millisecond after first connect)

The daemon is an implementation detail, not a user-facing service. It starts
lazily on first connection and shuts down after an idle timeout.

### Why Rust + rmcp

The two existing Rust MCP tools (mcp-probe, mcpeek) both chose to implement
their own protocol handling. This creates ongoing spec compliance risk as MCP
evolves rapidly (2024-11-05 → 2025-11-25 added sampling, elicitation, tasks,
extensions). By building on the official `rmcp` SDK:

1. **Spec compliance is inherited** — protocol changes are absorbed via crate updates
2. **Reduced maintenance surface** — protocol bugs are fixed upstream
3. **Community alignment** — rmcp is the Rust community's standard MCP implementation
4. **Feature velocity** — new spec capabilities (tasks, extensions) arrive as SDK features

### GitHub Organization

BOHICA-LABS (public repository: `forge-mcp`)
