---
document_type: domain-spec-section
level: L2
section: capabilities
version: "1.2"
status: draft
producer: business-analyst
timestamp: 2026-03-29T11:05:00
phase: 1a
inputs: [product-brief.md, market-intel.md, domain-research]
input-hash: ""
traces_to: L2-INDEX.md
---

# Domain Capabilities

> **Sharded L2 section (DF-021).** Navigate via `L2-INDEX.md`.
> **v1.1 — Updated from domain research reconciliation.** Key changes: CAP-004
> refined to distinguish server vs. client capabilities, CAP-005 clarified with
> server-initiated methods, transport description corrected (Streamable HTTP
> replaces deprecated SSE), rmcp SDK confirmed as `warpdotdev/rmcp`.

## Brief Capability 1: Server Discovery and Connection Management

**CAP-001 — Config Import and Parsing.** Parse MCP server configuration from Claude Desktop (`claude_desktop_config.json`), Cursor (`~/.cursor/mcp.json` global or `.cursor/mcp.json` project-scoped), VS Code (`mcp.json` — user-level or `.vscode/mcp.json` workspace-scoped), and Windsurf (`~/.codeium/windsurf/mcp_config.json`). Must handle two distinct JSON schemas: the `"mcpServers"` top-level key (Claude Desktop, Cursor, Windsurf) and the `"servers"` top-level key with explicit `"type"` field (VS Code). Normalize heterogeneous config formats into a unified internal server registry. Priority: P0.

**CAP-002 — Transport Connection.** Establish connections to MCP servers via stdio and Streamable HTTP transports using the rmcp SDK (`warpdotdev/rmcp`). Streamable HTTP replaces the deprecated SSE transport and uses HTTP for bidirectional streaming with JSON-RPC 2.0. Manage connection lifecycle including handshake, capability negotiation, keepalive, and graceful shutdown. Priority: P0.

**CAP-003 — Session Persistence and Pooling.** Maintain named persistent sessions via a background daemon process. Pool connections for concurrent access by TUI and CLI. Warm startup for repeated agent interactions after initial connect. Lazy daemon start on first connection, idle timeout shutdown. Priority: P0.

## Brief Capability 2: Full MCP Protocol Coverage

**CAP-004 — Capability Negotiation.** Perform bidirectional capability negotiation per MCP 2025-11-25 spec. MCP distinguishes **server capabilities** (tools, resources, prompts, logging, completions, tasks, extensions, experimental) from **client capabilities** (roots, sampling, elicitation, tasks, extensions, experimental). Forge MCP must advertise appropriate client capabilities to enable server-initiated features like sampling and elicitation. Gracefully degrade when connecting to servers on older spec versions (2024-11-05). In rmcp, capabilities are built via `ClientCapabilitiesBuilder` with fluent `enable_*()` methods (e.g., `enable_sampling()`, `enable_roots()`, `enable_elicitation()`). Priority: P0.

**CAP-005 — Protocol Method Invocation.** Invoke any MCP protocol method through a unified dispatch interface (~25 distinct method names per MCP spec). Client-initiated methods: `tools/call`, `tools/list`, `resources/read`, `resources/list`, `resources/subscribe`, `resources/unsubscribe`, `prompts/get`, `prompts/list`, `completion/complete`, `logging/setLevel`, `ping`. Server-initiated methods (handled by client): `sampling/createMessage` (proxy to external LLM APIs with tool calling and parallel calls support), `elicitation/create` (form-based or URL-based user input), `roots/list` queries. Handle notifications bidirectionally: client→server (`notifications/initialized`, `notifications/roots/list_changed`, `notifications/cancelled`, `notifications/progress`) and server→client (`notifications/resources/list_changed`, `notifications/resources/updated`, `notifications/tools/list_changed`, `notifications/prompts/list_changed`, `notifications/message` for logging, `notifications/progress`, `notifications/cancelled`). Handle the extensions system for non-standard capabilities and tasks for long-running operations. Support cursor-based pagination for list methods (`tools/list`, `resources/list`, `prompts/list`). Priority: P0.

## Brief Capability 3: Interactive TUI Dashboard

**CAP-006 — Multi-Pane Layout.** Render a ratatui-based terminal UI with server browser, capability explorer, traffic inspector, and health metrics panels. Adapt layout from 80×24 minimum to ultra-wide terminals. Support 16-color, 256-color, and truecolor with auto-detection. Priority: P0.

**CAP-007 — Keyboard-First Navigation.** Implement vi-style navigation (hjkl movement, `/` search, `:` commands) across all TUI panes. Maintain discoverable shortcut bar. Support mouse as supplementary input. Ensure full keyboard-only operation for accessibility. Priority: P0.

**CAP-008 — TUI Data Rendering.** Render JSON-RPC messages with syntax highlighting, sparklines for time-series health metrics, latency histograms, tables, lists, status badges, and Unicode box-drawing (ASCII fallback for limited terminals). No color-only indicators — always pair with shape/text. Priority: P0.

## Brief Capability 4: Protocol-Level Traffic Inspection

**CAP-009 — Traffic Capture.** Intercept and record all JSON-RPC messages between client and server without modifying message content. Attach per-message timestamps and compute timing analysis (latency, gaps, throughput). Must capture both client-initiated and server-initiated messages (sampling requests, elicitation requests, notifications). Priority: P0.

**CAP-010 — Traffic Filter, Search, and Replay.** Filter captured traffic by method name, direction, time range, and content pattern. Full-text search across message payloads. Replay captured message sequences against a server for debugging. Priority: P0.

## Brief Capability 5: Non-Interactive CLI Mode

**CAP-011 — Scriptable Subcommands.** Expose `list`, `call`, `info`, `grep`, `test` subcommands with structured JSON output on stdout and human-readable diagnostics on stderr. Conventional exit codes (0=success, 1=test failure, 2=connection error). Priority: P0.

**CAP-012 — Agent-Optimized Output.** Produce minimal-token output for the discover → inspect → call workflow, targeting < 500 tokens total overhead. Clean, pipeable output suitable for AI agent consumption and shell scripting. Priority: P0.

## Brief Capability 6: Server Health Monitoring

**CAP-013 — Metric Collection.** Continuously collect latency histograms, error rate trends, throughput counters, and connection status for each monitored server. Collect without impacting server latency by more than 1%. Priority: P0.

**CAP-014 — Alerting Thresholds.** Support configurable thresholds for latency, error rate, and throughput. Trigger alerts when metrics breach thresholds. Emit structured alert events for consumption by TUI and CLI. Priority: P0.

**CAP-015 — Time-Series Visualization.** Display metric history as sparklines and histograms in TUI. Expose metric snapshots as JSON via CLI for integration with external monitoring systems. Priority: P0.

## Brief Capability 7: Runtime Security Auditing

**CAP-016 — Dangerous Pattern Detection.** Analyze live MCP traffic to flag dangerous tool patterns: file system access, code execution, network calls, SSRF attempts (targeting private IPs, cloud metadata endpoints like 169.254.169.254), and data exfiltration vectors. Classify findings by severity with confidence scores. Map to OWASP AST10 categories (AST01-AST10). Priority: P1.

**CAP-017 — Permission and Auth Verification.** Detect permission escalation attempts, verify root enforcement compliance (are servers respecting declared roots boundaries?), validate authentication handling in server responses. Map findings to OWASP AST10 risk categories, specifically AST03 (Over-Privileged Skills) and AST06 (Weak Isolation). Priority: P1.

**CAP-018 — Compliance Report Generation.** Generate structured security audit reports (JSON, human-readable) summarizing findings, severity distribution, and OWASP AST10 coverage. Support user-defined suppression rules for accepted risks. Priority: P1.

## Brief Capability 8: Protocol Conformance Testing

**CAP-019 — Spec Compliance Suite.** Execute automated tests validating capability negotiation, error handling, transport compliance, and method coverage against MCP 2025-11-25. Target ≥ 90% of spec methods exercised. Test both server capabilities and server responses to client capability advertisements. Priority: P1.

**CAP-020 — CI/CD Output Formats.** Produce conformance test results in JSON and JUnit XML formats for integration with GitHub Actions, GitLab CI, and Jenkins. Meaningful exit codes for pass/fail gating. Priority: P1.

## Brief Capability 9: Config Sync and Drift Detection

**CAP-021 — Cross-Editor Config Comparison.** Detect configuration differences across Claude Desktop, Cursor, VS Code, and Windsurf MCP configs. Report discrepancies with actionable detail including which servers differ and how. Priority: P1.

**CAP-022 — Reconciliation Workflow.** Guide users through resolving detected config drift with clear choices (accept source A, accept source B, merge). Prevent silent drift from causing "works on my machine" failures. Priority: P2.

## Brief Capability 10: Server Comparison and Diff

**CAP-023 — Tool Schema Diff.** Compare tool definitions between two MCP servers: parameter schemas, descriptions, and annotations (including readOnlyHint, destructiveHint, openWorldHint, idempotentHint). Highlight additions, removals, and changes. Priority: P2.

**CAP-024 — Capability Delta.** Compare capability sets between two servers: which capabilities each supports, protocol version differences, and feature gaps. Include both server-side and client-side capability differences. Priority: P2.

**CAP-025 — Response Behavior Delta.** Issue identical requests to two servers and compare response structures, timing characteristics, and error handling behavior. Useful for staging-vs-production and version upgrade validation. Priority: P2.
