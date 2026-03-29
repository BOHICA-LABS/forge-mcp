---
document_type: dtu-assessment
level: L3
version: "1.0"
status: draft
producer: architect
timestamp: 2026-03-29T14:05:00
phase: 1b
inputs: [specs/architecture/api-surface.md, specs/architecture/module-decomposition.md]
traces_to: specs/architecture/ARCH-INDEX.md
---

# DTU Assessment: Forge MCP

## Result: DTU CLONES RECOMMENDED FOR TESTING

Forge MCP connects to external MCP servers via rmcp. While it doesn't depend on cloud services or APIs (like Stripe/GitHub), it DOES need mock MCP servers for integration testing. These are specialized DTU clones.

## Summary

| Metric | Value |
|--------|-------|
| External dependencies identified | 2 |
| DTU clones recommended | 2 |
| Total clone story points | 5 |
| Estimated Wave 1 capacity needed | 5 points |

## Dependency Analysis

| # | Service | Integration Type | Usage Scope | Fidelity | DTU? | Points | Justification |
|---|---------|-----------------|-------------|----------|------|--------|---------------|
| 1 | MCP Server (stdio) | stdio subprocess | Full protocol operations — all 25 methods, capability negotiation, pagination, notifications, error cases | L3 (Behavioral) | YES | 3 | Core testing dependency. Must simulate: capability negotiation, tool/resource/prompt serving, pagination, server-initiated methods (sampling, elicitation), error scenarios, schema drift. Cannot test against production MCP servers deterministically. |
| 2 | MCP Server (HTTP) | Streamable HTTP | HTTP transport testing — session management, SSE streams, reconnection | L2 (Stateful) | YES | 2 | Need to test HTTP-specific behaviors: session IDs, SSE streaming, reconnection. Less complex than stdio mock (subset of protocol behaviors). |

## Services NOT Requiring DTU

| # | Service | Reason |
|---|---------|--------|
| 1 | External LLM API (sampling proxy) | Mock HTTP endpoint sufficient — simple request/response, no stateful behavior needed. Use `wiremock` or similar. |
| 2 | Editor config files | Filesystem fixtures — no service needed. Test with sample JSON files. |

## DTU Architecture

### Mock MCP Server (stdio) — `forge-test-server`

- Implement using rmcp's server-side API (`ServerHandler` trait)
- Configurable behaviors: which capabilities to advertise, pagination behavior, error injection, schema drift simulation
- Runs as a subprocess (same as production MCP servers)
- Key test scenarios:
  - Successful capability negotiation (all capability combinations)
  - Pagination with configurable page sizes and cursor patterns
  - Cycle detection testing (intentionally return cycling cursors)
  - Server-initiated methods (sampling requests, elicitation requests)
  - Schema drift (change tool descriptions between connections)
  - Error injection (timeout, malformed responses, crash)
  - Batch JSON-RPC handling

### Mock MCP Server (HTTP) — `forge-test-http-server`

- HTTP server using rmcp's Streamable HTTP transport
- Configurable session behavior: session ID management, session expiry, reconnection
- SSE stream support for server-initiated messages
- Key test scenarios:
  - Session establishment and `Mcp-Session-Id` header handling
  - Session loss and re-initialization
  - SSE stream for notifications

## Development Approach

Each DTU clone is built with rmcp's server API — this ensures the mock servers use the same protocol implementation as production servers. Clones are Rust binaries in the `tests/` directory, not separate Docker containers (since they're in-process or subprocess-based).

### Note on rmcp test infrastructure

rmcp itself may provide test utilities. If `rmcp` includes mock server capabilities, prefer those over custom implementations. Research needed during implementation.
