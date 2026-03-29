---
document_type: domain-spec-section
level: L2
section: entities
version: "1.0"
status: draft
producer: business-analyst
timestamp: 2026-03-29T10:55:00
phase: 1a
inputs: [product-brief.md, market-intel.md]
input-hash: ""
traces_to: L2-INDEX.md
---

# Domain Entities

> **Sharded L2 section (DF-021).** Navigate via `L2-INDEX.md`.

## MCP Server

A registered MCP server that Forge MCP can connect to. Attributes: name (unique identifier), transport type (stdio | streamable-http), connection parameters (command + args for stdio, URL for HTTP), config source (which editor config it was imported from), connection status (disconnected | connecting | connected | error), supported capabilities set, protocol version. A server may appear in multiple config sources with potentially differing parameters.

## MCP Client Session

An active connection session between Forge MCP and an MCP server. Attributes: session ID (named, persistent), server reference, transport connection, negotiated capabilities, creation timestamp, last activity timestamp, message count. Sessions survive individual CLI invocations when managed by the daemon. Multiple sessions to the same server are allowed for parallel access.

## Transport Connection

The underlying communication channel to an MCP server. Types: stdio (manages child process with stdin/stdout pipes) and Streamable HTTP (HTTP client with SSE support). Attributes: transport type, connection state, bytes sent/received, error count. Managed by the rmcp SDK — Forge MCP does not implement transport logic.

## Tool

An MCP tool exposed by a server. Attributes: name, description, input schema (JSON Schema), annotations (readOnlyHint, destructiveHint, openWorldHint, idempotentHint), availability (enabled/disabled). Tools are the primary interaction primitive for AI agents.

## Resource

A server-side data source accessible by URI. Attributes: URI, name, description, MIME type, content (text or binary blob). Resources provide context data to AI models — documents, database records, API responses.

## Resource Template

A parameterized URI pattern for dynamic resource access. Attributes: URI template (RFC 6570), name, description, MIME type. Templates enable dynamic resource discovery without enumerating all possible URIs.

## Prompt

A reusable prompt template with optional arguments. Attributes: name, description, arguments (name, description, required flag). Prompts standardize common interaction patterns between AI models and tools.

## JSON-RPC Message

A single protocol message exchanged between client and server. Attributes: message ID, method name, direction (client→server | server→client), params/result/error payload, timestamp (capture time), latency (time to response for request/response pairs), raw JSON content. Messages follow JSON-RPC 2.0 framing as mandated by MCP.

## Traffic Capture

An ordered collection of JSON-RPC messages from a session. Attributes: session reference, start/end timestamps, message count, filter criteria (if filtered), total bytes captured. Captures are append-only during a session and can be exported for offline analysis.

## Health Metric

A time-series measurement of server behavior. Types: latency histogram (per-method), error rate (errors/total over window), throughput (requests/second), connection status (up/down). Attributes: server reference, metric type, timestamp, value, window size. Metrics are collected passively from observed traffic without synthetic probes.

## Security Finding

A security issue detected by runtime behavioral analysis. Attributes: finding ID, severity (critical | high | medium | low | info), category (OWASP AST10 mapping), description, evidence (the specific message or pattern that triggered it), confidence score (0.0–1.0), suppressed flag (user-accepted risk). Findings are generated from traffic analysis, not source code.

## Config Source

A file on disk containing MCP server configuration for a specific editor/IDE. Types: Claude Desktop, Cursor, VS Code, Windsurf. Attributes: file path, editor type, parse status (valid | invalid | missing), server entries extracted, last modified timestamp. Config sources are read-only — Forge MCP inspects but does not modify them.

## Conformance Test

A single spec compliance check against an MCP server. Attributes: test ID, spec section reference (MCP 2025-11-25), description, preconditions, expected behavior, actual result (pass | fail | skip | error), execution duration, evidence (request/response that triggered pass or fail). Tests are grouped into suites by capability area.

## Conformance Suite

A collection of conformance tests targeting a specific capability area. Attributes: suite name, capability area, test count, pass/fail/skip counts, execution duration, output format (JSON, JUnit XML). Suites compose into a full spec coverage run.

## Alert Threshold

A user-configured boundary for health metrics. Attributes: metric type, comparison operator, threshold value, window size, alert state (normal | breached | recovered). When a metric crosses a threshold, an alert event is emitted.

## Server Diff Result

The output of comparing two MCP servers. Attributes: source server, target server, tool schema changes (added/removed/modified), capability differences, response behavior deltas, comparison timestamp. Used for migration validation and version upgrade verification.
