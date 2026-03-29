---
document_type: domain-spec-section
level: L2
section: entities
version: "1.2"
status: draft
producer: business-analyst
timestamp: 2026-03-29T11:05:00
phase: 1a
inputs: [product-brief.md, market-intel.md, domain-research]
input-hash: ""
traces_to: L2-INDEX.md
---

# Domain Entities

> **Sharded L2 section (DF-021).** Navigate via `L2-INDEX.md`.
> **v1.1 — Updated from domain research reconciliation.** Key changes: added
> Negotiated Capabilities entity with client/server capability distinction,
> added Sampling Request, Elicitation Request, Root, and Task entities,
> refined Tool/Resource/Prompt with MCP interaction model roles, expanded
> Security Finding with OWASP AST10 categories and known attack vectors.

## MCP Server

A registered MCP server that Forge MCP can connect to. Attributes: name (unique identifier), transport type (stdio | streamable-http), connection parameters (command + args for stdio, URL for HTTP), config source (which editor config it was imported from), connection status (disconnected | connecting | connected | error), supported capabilities set, protocol version. A server may appear in multiple config sources with potentially differing parameters.

## MCP Client Session

An active connection session between Forge MCP and an MCP server. Attributes: session ID (named, persistent), server reference, transport connection, negotiated capabilities, creation timestamp, last activity timestamp, message count. Sessions survive individual CLI invocations when managed by the daemon. Multiple sessions to the same server are allowed for parallel access.

## Transport Connection

The underlying communication channel to an MCP server. Types:
- **stdio**: Manages child process with stdin/stdout pipes. Client writes JSON-RPC to server's stdin, reads from stdout. Server must not write non-protocol data to stdout (use stderr for logging). UTF-8 encoded, newline-delimited messages. Best for local/dev, single-client scenarios.
- **Streamable HTTP**: Replaces the deprecated SSE transport from MCP 2024-11-05. Server exposes a single MCP endpoint URL. Client sends JSON-RPC as HTTP POST; server responds with either single JSON response (`application/json`) or SSE stream (`text/event-stream`) for multi-message responses. Client can GET the endpoint for server-initiated message streams. Supports `Mcp-Session-Id` header for session binding and `Last-Event-ID` for stream resumption.

Attributes: transport type, connection state, bytes sent/received, error count, session ID (HTTP only). Managed by the rmcp SDK (`warpdotdev/rmcp`) — Forge MCP does not implement transport logic. rmcp also supports TCP transport for direct socket connections.

## Negotiated Capabilities

The set of capabilities agreed upon between client and server during the initialize/initialized handshake. MCP distinguishes between **server capabilities** and **client capabilities**:

- **Server capabilities** (what the server offers): tools, resources, prompts, logging, completions, tasks, extensions, experimental.
- **Client capabilities** (what the client can handle): roots, sampling, elicitation, tasks, extensions, experimental.

The client declares what it can handle (e.g., sampling requests from server, elicitation requests, root listings), and the server declares what it offers. Forge MCP must advertise client capabilities for sampling, elicitation, and roots to enable servers that depend on these features. Attributes: server capabilities set, client capabilities set, protocol version, server info, client info. In rmcp, capabilities are built via `ServerCapabilitiesBuilder` and `ClientCapabilitiesBuilder` with fluent `enable_*()` methods.

## Tool

An MCP tool exposed by a server. Attributes: name, description, input schema (JSON Schema), output schema (optional), annotations (readOnlyHint, destructiveHint, openWorldHint, idempotentHint — hints for client/LLM guidance on tool behavior), availability (enabled/disabled). Tools are **model-controlled** (the AI decides when to invoke them). Tools support side effects and require user consent for destructive operations when destructiveHint is set. Discovered via `tools/list`, invoked via `tools/call`. Servers notify of changes via `tools/list_changed` notification.

## Resource

A server-side data source accessible by URI. Attributes: URI, name, description, MIME type, content (text or binary blob). Resources are **application-controlled** (the client application decides when to access them, not the AI model). They provide context data to AI models — documents, database records, API responses. Discovered via `resources/list`, read via `resources/read`. Servers notify of changes via `resources/list_changed` notification.

## Resource Template

A parameterized URI pattern for dynamic resource access. Attributes: URI template (RFC 6570), name, description, MIME type. Templates enable dynamic resource discovery without enumerating all possible URIs.

## Prompt

A reusable prompt template with optional arguments. Attributes: name, description, arguments (name, description, required flag). Prompts are **user-controlled** (explicitly selected by users, e.g., via slash commands). They generate structured messages with role/content pairs that can include text and embedded resource references. Discovered via `prompts/list`, retrieved via `prompts/get`.

## Sampling Request

A server-initiated request asking the client to perform LLM inference. This is a **client capability** — the server can only issue sampling requests if the client advertised sampling support during negotiation. Attributes: messages (conversation context), model preferences, system prompt, max tokens, temperature, stop sequences. Forge MCP proxies sampling requests to external LLM APIs (no embedded LLM). The 2025-11-25 spec added tool calling within sampling and parallel tool calls.

## Elicitation Request

A server-initiated request asking the client to obtain user input. This is a **client capability** — servers can only issue elicitation requests if the client advertised elicitation support. Supports two modes: form-based (structured input fields) and URL-based (redirect user to external URL, e.g., OAuth flows). Attributes: request type (form | URL), schema (for forms), URL (for redirects), description, required fields. Used for credentials, OAuth, payments, and other interactive flows.

## Root

A client-defined URI or filesystem boundary that tells the server what scope of resources it may access. This is a **client capability** — the client advertises roots to constrain server operations. Attributes: URI, name. Roots enable safe, scoped operations without granting broad filesystem or network access. Listed via `roots/list`; servers are notified of changes via `roots/list_changed`.

## Task

A long-running server operation tracked by ID. Tasks enable async workflows where the server starts work that may take significant time. Attributes: task ID, status (working | input_required | completed | failed | cancelled), progress information, result. Both client and server can declare task capability. Clients can query task status and cancel tasks. Added to MCP in the November 2025 spec update.

## Resource Subscription

A client subscription to change notifications for a specific resource. Created via `resources/subscribe`, removed via `resources/unsubscribe`. Server sends `notifications/resources/updated` when a subscribed resource changes. Attributes: resource URI, subscription status (active | inactive), server reference, session reference. Requires the server to advertise `resources` capability with `subscribe: true`. Enables reactive UIs that update when server-side data changes.

## Paginated List Response

A response to a list method (`tools/list`, `resources/list`, `prompts/list`) that may contain a subset of results with a cursor for fetching more. Attributes: items (the returned entries), next cursor (opaque string, absent when no more results). Forge MCP must handle pagination transparently — iterating cursors until exhausted to present complete lists. Cursor format is server-defined and opaque.

## JSON-RPC Message

A single protocol message exchanged between client and server. Four types per JSON-RPC 2.0: **Request** (has `id`, expects response), **Response** (has `id`, exactly one of `result` or `error`), **Notification** (no `id`, fire-and-forget), **Batch** (JSON array of requests/notifications — server processes independently, responses may arrive in any order). Attributes: message ID (String | Number | null for requests/responses; absent for notifications), method name, direction (client→server | server→client), params/result/error payload, timestamp (capture time), latency (time to response for request/response pairs), raw JSON content, is_batch flag. Messages follow JSON-RPC 2.0 framing as mandated by MCP. Direction matters because some methods are server-initiated (sampling, elicitation) while most are client-initiated. MCP error codes: standard JSON-RPC (-32700 through -32603) plus implementation-defined server errors (-32000 to -32099). Tool execution errors use `result.isError: true` (not JSON-RPC error codes).

## Traffic Capture

An ordered collection of JSON-RPC messages from a session. Attributes: session reference, start/end timestamps, message count, filter criteria (if filtered), total bytes captured. Captures are append-only during a session and can be exported for offline analysis.

## Health Metric

A time-series measurement of server behavior. Types: latency histogram (per-method), error rate (errors/total over window), throughput (requests/second), connection status (up/down). Attributes: server reference, metric type, timestamp, value, window size. Metrics are collected passively from observed traffic without synthetic probes.

## Security Finding

A security issue detected by runtime behavioral analysis. Attributes: finding ID, severity (critical | high | medium | low | info), category (OWASP AST10 mapping), description, evidence (the specific message or pattern that triggered it), confidence score (0.0–1.0), suppressed flag (user-accepted risk). Findings are generated from traffic analysis, not source code.

OWASP AST10 categories (Dec 2025): AST01 Malicious Skills, AST02 Supply Chain Compromise, AST03 Over-Privileged Skills, AST04 Insecure Metadata, AST05 Unsafe Deserialization, AST06 Weak Isolation, AST07 Update Drift, AST08 Poor Scanning, AST09 No Governance, AST10 Cross-Platform Reuse.

Known MCP attack vectors from domain research: SSRF (36.7% of 7,000+ scanned servers per BlueRock "MCP fURI" research), prompt injection (tool description poisoning), tool poisoning (corrupted outputs/descriptors), permission escalation, data exfiltration via unrestricted URIs. Notable CVEs: CVE-2025-68145/68143/68144 (Anthropic Git/Filesystem MCP servers RCE), CVE-2025-6515 (oatpp-mcp prompt hijacking).

## Config Source

A file on disk containing MCP server configuration for a specific editor/IDE. Types: Claude Desktop, Cursor, VS Code, Windsurf. Attributes: file path, editor type, schema type (`mcpServers` for Claude Desktop/Cursor/Windsurf, `servers` with explicit `type` field for VS Code), parse status (valid | invalid | missing), server entries extracted, last modified timestamp. Config sources are read-only — Forge MCP inspects but does not modify them.

Config file paths vary by OS and editor:
- **Claude Desktop**: `~/Library/Application Support/Claude/claude_desktop_config.json` (macOS), `%APPDATA%\Claude\claude_desktop_config.json` (Windows), `~/.config/Claude/claude_desktop_config.json` (Linux).
- **Cursor**: `~/.cursor/mcp.json` (global), `.cursor/mcp.json` (project-scoped).
- **VS Code**: `~/Library/Application Support/Code/User/mcp.json` (macOS user), `.vscode/mcp.json` (workspace). Uses `"servers"` top-level key (not `"mcpServers"`), explicit `"type": "stdio"|"http"` field.
- **Windsurf**: `~/.codeium/windsurf/mcp_config.json`. Supports environment variable interpolation.

Editor-specific extensions: Cursor supports `"disabled"` (bool) and `"alwaysAllow"` (tool name array). VS Code supports IntelliSense schema validation and input variables for secrets.

## Conformance Test

A single spec compliance check against an MCP server. Attributes: test ID, spec section reference (MCP 2025-11-25), description, preconditions, expected behavior, actual result (pass | fail | skip | error), execution duration, evidence (request/response that triggered pass or fail). Tests are grouped into suites by capability area.

## Conformance Suite

A collection of conformance tests targeting a specific capability area. Attributes: suite name, capability area, test count, pass/fail/skip counts, execution duration, output format (JSON, JUnit XML). Suites compose into a full spec coverage run.

## Alert Threshold

A user-configured boundary for health metrics. Attributes: metric type, comparison operator, threshold value, window size, alert state (normal | breached | recovered). When a metric crosses a threshold, an alert event is emitted.

## Server Diff Result

The output of comparing two MCP servers. Attributes: source server, target server, tool schema changes (added/removed/modified), capability differences, response behavior deltas, comparison timestamp. Used for migration validation and version upgrade verification.
