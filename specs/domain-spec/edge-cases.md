---
document_type: domain-spec-section
level: L2
section: edge-cases
version: "1.1"
status: draft
producer: business-analyst
timestamp: 2026-03-29T11:05:00
phase: 1a
inputs: [product-brief.md, market-intel.md, domain-research]
input-hash: ""
traces_to: L2-INDEX.md
---

# Domain Edge Cases

> **Sharded L2 section (DF-021).** Navigate via `L2-INDEX.md`.
> Edge cases are valid but unusual scenarios that the system must handle
> gracefully. Each DEC defines the scenario, expected behavior, and which
> capabilities it stresses.
> **v1.1 — Updated from domain research reconciliation.** Added DEC-016 through
> DEC-019 for server-initiated method edge cases (sampling, elicitation, SSRF
> detection, task lifecycle).

## Connection Edge Cases

**DEC-001 — Server crashes mid-request.** A server process terminates (SIGKILL, OOM, unhandled panic) while Forge MCP is waiting for a JSON-RPC response. Expected: Pending request times out or detects EOF, ConnectionLost event fires, session status transitions to error, user sees clear error message (not a hang). Health monitor records the failure. Stresses: CAP-002, CAP-013, FM-002.

**DEC-002 — Server sends malformed JSON-RPC.** Server responds with invalid JSON or valid JSON that does not conform to JSON-RPC 2.0 (missing id, wrong version field, non-object response). Expected: Message rejected at protocol layer (rmcp), error logged with raw content for debugging, session remains active for subsequent valid messages. Traffic capture records the malformed message for inspection. Domain research confirms this is common — even Anthropic's own MCP servers had implementation defects (CVE-2025-68145). Stresses: CAP-009, DI-004, ASM-003.

**DEC-003 — Capability mismatch between client and server.** Server advertises a capability during negotiation but returns errors when that capability is exercised (e.g., claims tools support but returns "method not found" for tools/list). Expected: Error surfaced to user with clear context ("server advertised tools but tools/list failed"), capability marked as unreliable in session state. Security auditor may flag this as suspicious (DEC-012). Stresses: CAP-004, CAP-005, DI-002.

**DEC-004 — Simultaneous stdio and HTTP connections to same server.** A server is configured via stdio in one editor config and via HTTP in another. User attempts to connect to both. Expected: Both connections are allowed (they are independent transport-level sessions). Diff operations between the two can reveal transport-dependent behavioral differences. Session IDs must be unique (DI-003). Stresses: CAP-002, CAP-003, CAP-023.

**DEC-005 — Server with extremely slow responses.** A server takes > 30 seconds to respond to a method call (e.g., expensive computation, external API timeout). Expected: Forge MCP does not hang — TUI remains responsive, timeout is configurable per-server, pending request indicator is visible, user can cancel the request. Health monitor records the latency spike. Stresses: CAP-005, CAP-013, CAP-006.

**DEC-006 — Daemon already running on startup.** User launches Forge MCP while a daemon instance from a previous session is already running and holding the socket/port. Expected: Detect existing daemon, connect to it (reuse sessions), do not start a second daemon. If the existing daemon is unresponsive, offer to kill and restart. Stresses: CAP-003, FM-005.

## Config Edge Cases

**DEC-007 — Config file missing entirely.** An expected editor config path does not exist (e.g., Claude Desktop not installed). Expected: ConfigSourceInvalid event with status=missing. No error — silently skip and proceed with available configs. CLI `list` shows which sources were found and which were missing. Stresses: CAP-001, FM-009.

**DEC-008 — Config file exists but is empty or invalid JSON.** Config file is present but contains empty content, invalid JSON, or valid JSON with unexpected schema. Expected: Parse error reported with file path and error location. Servers from that source are unavailable but other sources are unaffected. Stresses: CAP-001, FM-009.

**DEC-009 — Config file changes while Forge MCP is running.** User modifies an editor config file while Forge MCP has already imported it. Expected: If file watching is implemented, detect change and re-import. If not, config is stale until next explicit import. Document the staleness behavior clearly. Stresses: CAP-001, CAP-021.

**DEC-010 — Same server name in multiple configs with different parameters.** Claude Desktop defines server "my-api" with stdio transport, while Cursor defines "my-api" via HTTP with different arguments. Expected: Both entries are preserved with config source attribution. Drift detection flags the discrepancy. User can inspect and compare both definitions. Stresses: CAP-001, CAP-021, DI-015.

## Traffic and Security Edge Cases

**DEC-011 — Extremely high message rate.** A server produces messages at > 1,000/sec (e.g., verbose logging output, streaming resource). Expected: Traffic capture buffers or applies backpressure without dropping messages silently. TUI may throttle display refresh but underlying capture is complete. Health monitor continues to function. Memory growth is bounded (FM-012). Stresses: CAP-009, CAP-006, DI-005.

**DEC-012 — Server that lies about capabilities.** A server advertises capabilities it doesn't implement, or fails to advertise capabilities it does implement. Expected: Discovered through conformance testing or through runtime errors when invoking methods. Security auditor flags discrepancy between advertised and actual behavior. Conformance test explicitly checks for this. Stresses: CAP-004, CAP-016, CAP-019.

**DEC-013 — Security suppression after update changes severity.** A finding was suppressed at severity=medium, but a rule update reclassifies it as severity=critical. Expected: Suppression is per-finding-instance, not per-rule. The existing suppressed finding remains suppressed but new occurrences at the new severity are not auto-suppressed. Document this behavior in suppression UX. Stresses: CAP-018, DI-012.

## Conformance and Comparison Edge Cases

**DEC-014 — Server disconnects during conformance test run.** Server crashes or network fails partway through a conformance suite. Expected: Completed tests retain their results. Remaining tests are marked as skip with reason "connection lost". Suite produces partial results with clear indication of incompleteness. Stresses: CAP-019, CAP-020, FM-002.

**DEC-015 — Comparing servers with different protocol versions.** One server runs MCP 2025-11-25, the other runs 2024-11-05. Comparison reveals capabilities that only exist in the newer spec. Expected: Diff report clearly attributes differences to protocol version vs. server implementation. Capabilities absent due to older spec are labeled as "not available in protocol version X" rather than "missing." Stresses: CAP-004, CAP-023, CAP-024.

## Server-Initiated Method Edge Cases (new)

**DEC-016 — Server sends sampling request but no LLM provider configured.** Server requires client sampling to function, sends a sampling request, but Forge MCP has no LLM API key/provider configured for proxying. Expected: Return a protocol-level error to the server indicating sampling is unavailable. Display clear user-facing message in TUI/CLI: "Server requested LLM sampling but no LLM provider is configured." Do not crash or hang. If sampling was advertised during negotiation, this represents a configuration gap — warn at connection time if sampling is advertised but no provider is set. Stresses: CAP-005, DI-016, DI-017, ASM-013.

**DEC-017 — Server sends elicitation request in non-interactive CLI mode.** Server sends an elicitation request (form or URL) while Forge MCP is running in non-interactive CLI mode (piped stdout, no TTY). Expected: Cannot display form or open URL interactively. Return a protocol-level error to the server. Log the elicitation request details to stderr for debugging. If the request is URL-based, optionally print the URL to stderr so the user can handle it manually. Stresses: CAP-005, CAP-011, DI-017, ASM-013.

**DEC-018 — SSRF attempt via tool response containing private IP.** A server's tool response includes a resource URI or redirect pointing to a private IP address (e.g., 169.254.169.254 cloud metadata, 10.x.x.x, 192.168.x.x). Expected: Security auditor flags as high-confidence SSRF finding (confidence ≥ 0.9 for metadata endpoint, ≥ 0.7 for other RFC1918 addresses). Finding mapped to AST03 (Over-Privileged) or AST06 (Weak Isolation). Evidence includes the specific URI and message context. This is a deterministic detection — not heuristic. Domain research: 36.7% of 7,000+ scanned MCP servers were vulnerable to SSRF (BlueRock). Stresses: CAP-016, CAP-017, DI-010, DI-011.

**DEC-019 — Long-running task exceeds client timeout.** Server starts a task (status: working) that runs for minutes or hours, exceeding any reasonable client-side timeout. Expected: Task status is trackable independently of request timeout. Client can poll task status, cancel the task, or disconnect and reconnect to check status later (if session persists via daemon). TUI shows task progress indicator. CLI `--wait` flag with configurable timeout. Stresses: CAP-005, CAP-003.
