---
document_type: prd-supplement-glossary
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-03-29T11:25:00
phase: 1a
inputs: [prd.md, domain-spec/L2-INDEX.md, planning/domain-research.md]
input-hash: ""
traces_to: prd.md
---

# Glossary: Forge MCP

> PRD supplement — domain terms with precise definitions.
> Referenced by: all agents.

## MCP Protocol Terms

| Term | Definition |
|------|-----------|
| **MCP (Model Context Protocol)** | Open standard by Anthropic providing a standardized way for AI applications (hosts/clients) to connect to external data sources and tools (servers). Uses JSON-RPC 2.0 as wire protocol. |
| **MCP Host** | The AI application (e.g., Claude Desktop, an IDE) that initiates connections to MCP servers. Contains one or more MCP clients. |
| **MCP Client** | A protocol-level entity within a host that maintains a 1:1 session with a single MCP server. |
| **MCP Server** | A lightweight program exposing resources, tools, and prompts via MCP. Runs as a subprocess (stdio) or HTTP endpoint. |
| **Capability Negotiation** | The initialize/initialized handshake where client and server exchange supported capabilities before any method invocations. |
| **Server Capabilities** | Capabilities the server advertises: tools, resources, prompts, logging, completions, tasks, extensions. |
| **Client Capabilities** | Capabilities the client advertises: roots, sampling, elicitation, tasks, extensions. Enable server-initiated methods. |
| **Tool** | An executable function exposed by an MCP server with a name, description, and input JSON Schema. Invoked via `tools/call`. |
| **Resource** | A data source exposed by an MCP server, identified by URI. Read via `resources/read`. Can be subscribed to for change notifications. |
| **Resource Template** | A parameterized resource URI pattern exposed by an MCP server. |
| **Prompt** | A pre-defined message template exposed by an MCP server with optional arguments. Retrieved via `prompts/get`. |
| **Sampling** | A server-initiated request (`sampling/createMessage`) asking the client to perform LLM inference. Forge MCP proxies these to external LLM APIs. |
| **Elicitation** | A server-initiated request (`elicitation/create`) asking the client to collect user input via a form or URL. |
| **Roots** | Filesystem boundaries that the client exposes to the server via `roots/list`. Servers should respect these boundaries. |
| **Completions** | Argument auto-completion for prompts or resources via `completion/complete`. |
| **Tasks** | Long-running operations introduced in MCP 2025-11-25. Track async work with status polling and cancellation. |
| **Extensions** | Custom non-standard methods that both sides can negotiate support for. |
| **Progress Token** | An opaque token in `_meta.progressToken` enabling progress notifications for long-running requests. |

## Transport Terms

| Term | Definition |
|------|-----------|
| **Stdio Transport** | MCP transport where the server runs as a subprocess. Client writes JSON-RPC to stdin, reads from stdout. UTF-8, newline-delimited. |
| **Streamable HTTP Transport** | MCP transport replacing deprecated SSE. Single endpoint URL. Client POSTs JSON-RPC; server responds with JSON or SSE stream. Supports `Mcp-Session-Id` for session binding. |
| **SSE (Server-Sent Events)** | Deprecated transport from MCP 2024-11-05. Replaced by Streamable HTTP in 2025-11-25. |
| **Mcp-Session-Id** | HTTP header used in Streamable HTTP transport to bind requests to a session. Required for session continuity after initial negotiation. |

## JSON-RPC Terms

| Term | Definition |
|------|-----------|
| **JSON-RPC 2.0** | Wire protocol used by MCP. Defines request (with `id`), response (with `result` or `error`), notification (no `id`), and batch messages. |
| **Request** | JSON-RPC message with `jsonrpc`, `method`, optional `params`, and `id`. Expects a response. |
| **Notification** | JSON-RPC message with `jsonrpc` and `method` but no `id`. Fire-and-forget; no response expected. |
| **Batch Request** | JSON array of JSON-RPC requests/notifications. Response order may differ from request order; matched by `id`. |
| **isError** | A flag in MCP tool call results (`result.isError: true`) indicating tool-level failure. Distinct from JSON-RPC protocol errors — a successful JSON-RPC response can contain `isError: true`. |
| **Cursor-Based Pagination** | MCP's pagination model for list methods. Response includes `nextCursor`; client passes it in the next request to get the next page. |

## Security Terms

| Term | Definition |
|------|-----------|
| **OWASP AST10** | OWASP Agentic Skills Top 10 (December 2025). Ten risk categories for AI agent security. Runtime-detectable categories: AST01 (Malicious Skills), AST02 (Supply Chain), AST03 (Over-Privileged), AST04 (Insecure Metadata), AST05 (Unsafe Deserialization), AST06 (Weak Isolation). |
| **SSRF (Server-Side Request Forgery)** | Attack where a server makes requests to unintended destinations (private IPs, cloud metadata). 36.7% of MCP servers vulnerable (BlueRock). |
| **Rug Pull Attack** | Attack where a server presents benign tool descriptions during approval, then swaps to malicious versions post-onboarding. Detected via schema drift monitoring. |
| **Tool Poisoning** | Malicious instructions embedded in tool metadata (descriptions, schemas) invisible to users but processed by LLMs. |
| **Schema Drift** | Change in tool/resource/prompt metadata between connections. May be benign (server update) or malicious (rug pull). |
| **Confidence Score** | A [0.0, 1.0] score on security findings. 1.0 = deterministic detection (known-bad pattern); lower = heuristic match. |
| **Finding Suppression** | User-defined rules to hide known/accepted security findings. Suppression is an overlay — original finding preserved for audit trail. |

## Forge MCP Internal Terms

| Term | Definition |
|------|-----------|
| **Daemon** | Background process that maintains session pool, connection state, and metric collectors. Lazy-started on first connection; idle-timeout shutdown. |
| **Session Pool** | Collection of named persistent MCP sessions managed by the daemon. Enables concurrent TUI + CLI access and warm startup. |
| **Config Source** | An editor/IDE MCP configuration file (Claude Desktop, Cursor, VS Code, Windsurf) that Forge MCP discovers and parses. |
| **Config Registry** | Unified internal representation of servers aggregated from all config sources. Preserves source attribution for drift detection. |
| **Traffic Capture** | Complete record of JSON-RPC messages between client and server, with timestamps. Byte-identical to wire content (DI-005). |
| **Capture Buffer** | Bounded in-memory buffer holding traffic captures. Configurable size limit (default 100MB). FIFO eviction when full. |
| **Health Metrics** | Per-server latency histograms, error rates, throughput counters derived from passively observed traffic. |
| **Alert Threshold** | Configurable metric boundary. State machine: normal → breached → recovered. No duplicate alerts for same crossing. |
| **Conformance Suite** | Automated test battery validating MCP server compliance with a specific spec version. Produces JSON/JUnit XML output. |
| **Security Rule** | A detection pattern used by the security auditor. Rules have confidence scores and map to OWASP AST10 categories. |
| **Reconciliation** | Advisory workflow guiding users through resolving config drift between editors. Read-only — never modifies config files (DI-015). |

## Technology Terms

| Term | Definition |
|------|-----------|
| **rmcp** | Official Rust MCP SDK (`warpdotdev/rmcp`). Provides `Service`, `Peer`, `Transport` traits, `ClientCapabilitiesBuilder`, and transport implementations (stdio, HTTP). Forge MCP's hard dependency for all protocol operations (DI-004). |
| **ratatui** | Rust TUI framework (v0.30+). Immediate-mode rendering with Crossterm backend. Provides Layout, Widget, and StatefulWidget abstractions. |
| **Crossterm** | Cross-platform terminal manipulation library for Rust. Provides raw mode, alternate screen, event handling. Backend for ratatui. |
| **clap** | Rust CLI argument parser with derive macros. Handles subcommand routing, flag parsing, and help generation for Forge MCP's CLI. |
| **tokio** | Async runtime for Rust. Required by rmcp. Provides async I/O, timers, and task spawning. |
| **LTO (Link-Time Optimization)** | Compiler optimization applied at link time for smaller, faster binaries. Required to meet < 25MB binary size target. |
