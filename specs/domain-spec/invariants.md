---
document_type: domain-spec-section
level: L2
section: invariants
version: "1.2"
status: draft
producer: business-analyst
timestamp: 2026-03-29T11:05:00
phase: 1a
inputs: [product-brief.md, market-intel.md, domain-research]
input-hash: ""
traces_to: L2-INDEX.md
---

# Domain Invariants

> **Sharded L2 section (DF-021).** Navigate via `L2-INDEX.md`.
> Domain invariants are business rules that must hold at all times during system
> operation. Violation of any DI is a defect.
> **v1.1 — Updated from domain research reconciliation.** Added DI-016 through
> DI-018 for client capability advertisement, server-initiated method handling,
> and Streamable HTTP transport. Refined DI-002 and DI-004 with rmcp specifics.

## Connection and Protocol Invariants

**DI-001 — Capability negotiation before method invocation.** A server connection must complete capability negotiation before any tool calls, resource reads, or other protocol methods are dispatched. Rationale: The MCP spec requires initialize/initialized handshake before any other messages. Violation would produce undefined server behavior. Traces to: CAP-002, CAP-004.

**DI-002 — Protocol method gated by negotiated capabilities.** A protocol method must not be invoked on a server that did not advertise the corresponding capability during negotiation. This applies bidirectionally: client must not send `tools/call` to a server without tools capability, and client must not process sampling requests from a server if the client did not advertise sampling capability. In rmcp, server capabilities are exposed via `ServerCapabilities` struct fields (tools, resources, prompts, logging, completions, tasks). Rationale: Prevents protocol violations and confusing error states. Traces to: CAP-004, CAP-005.

**DI-003 — Session identity uniqueness.** Each named session must have a unique identifier within the daemon's session pool. No two active sessions may share the same session ID. Rationale: Prevents cross-session message routing errors. Traces to: CAP-003.

**DI-004 — Transport managed by rmcp only.** Forge MCP must not implement custom transport-layer protocol logic. All JSON-RPC framing, message serialization, and transport management must be delegated to the rmcp SDK (`warpdotdev/rmcp`). This includes stdio pipe management and Streamable HTTP bidirectional streaming. Rationale: Hard constraint from product brief — custom protocol code is the primary risk in existing tools (mcp-probe and mcpeek both rolled their own protocol). Traces to: CAP-002, ASM-001.

## Client Capability Invariants (new)

**DI-016 — Client capabilities must be explicitly advertised.** Forge MCP must advertise all client capabilities it supports (sampling, elicitation, roots) during the initialize handshake via `ClientCapabilitiesBuilder`. A server may depend on these client capabilities for its functionality — failing to advertise sampling means servers cannot delegate LLM inference, failing to advertise roots means servers cannot query filesystem boundaries. Rationale: Incomplete client capability advertisement causes silent feature loss. Traces to: CAP-004, CAP-005.

**DI-017 — Server-initiated methods require handler registration.** For every client capability advertised, Forge MCP must register a handler before completing the initialized notification. If sampling is advertised, a sampling handler (proxying to external LLM) must be ready. If elicitation is advertised, a user-input handler must be ready. Unhandled server-initiated requests violate the protocol contract. Rationale: Advertising a capability without handling it causes server-side errors. Traces to: CAP-005.

**DI-018 — Streamable HTTP is the only remote transport.** Forge MCP must support Streamable HTTP (not deprecated SSE) for HTTP-based connections. The SSE transport from MCP 2024-11-05 is deprecated and must not be implemented as a primary path. Rationale: Streamable HTTP is the current spec transport; SSE compatibility is at rmcp's discretion. Traces to: CAP-002.

## Traffic Inspection Invariants

**DI-005 — Traffic capture must not modify message content.** Captured JSON-RPC messages must be byte-identical to the messages exchanged on the wire. The capture layer is strictly read-only — no mutation, injection, or reordering. Rationale: Modified captures would undermine debugging trust. Traces to: CAP-009.

**DI-006 — Capture ordering preserves wire order.** Messages in a traffic capture must appear in the exact order they were observed on the transport. Reordering for display (e.g., sorting by method) must not affect the underlying capture. Rationale: Temporal ordering is essential for debugging protocol sequences. Traces to: CAP-009, CAP-010.

**DI-007 — Replay must target an explicit server.** Traffic replay must require an explicit target server designation. Replayed messages must never be accidentally sent to the original server or to an unintended destination. Rationale: Replay against production without intent could cause data mutations. Traces to: CAP-010.

## Health Monitoring Invariants

**DI-008 — Metric collection latency budget.** Health metric collection must not impact observed server latency by more than 1%. Metrics are derived passively from observed traffic, not from synthetic probes. Rationale: Monitoring that degrades the thing it monitors is counterproductive. Traces to: CAP-013, success criteria (< 1% overhead).

**DI-009 — Alert threshold state consistency.** An alert threshold must transition through exactly three states: normal → breached → recovered. The system must not emit duplicate breach alerts for the same threshold crossing. Rationale: Alert fatigue from duplicate alerts reduces trust. Traces to: CAP-014.

## Security Auditing Invariants

**DI-010 — Findings require evidence.** Every security finding must include the specific JSON-RPC message or pattern that triggered the detection. Findings without evidence must not be emitted. Rationale: Actionability — security teams need evidence to triage. Traces to: CAP-016, CAP-017.

**DI-011 — Confidence score bounds.** Security finding confidence scores must be in the range [0.0, 1.0] inclusive. A score of 1.0 indicates deterministic detection (e.g., known-dangerous tool name, SSRF to 169.254.169.254); lower scores indicate heuristic matches. Rationale: Bounded scores enable consistent threshold-based filtering. Traces to: CAP-016.

**DI-012 — Suppression is explicit and auditable.** Suppressed security findings must retain their original severity and evidence. Suppression is an overlay, not a deletion. All suppressions must be traceable to a user action. Rationale: Compliance auditability. Traces to: CAP-018.

## Output and CLI Invariants

**DI-013 — JSON on stdout, diagnostics on stderr.** In CLI mode, structured data output (JSON) must go exclusively to stdout. Human-readable messages, progress indicators, and diagnostics must go exclusively to stderr. Rationale: Unix philosophy — enables clean piping and programmatic consumption. Traces to: CAP-011, CAP-012.

**DI-014 — Exit code semantics.** Exit codes must follow documented semantics: 0 = success, 1 = test failure (conformance), 2 = connection error. Exit codes must not vary by output format or verbosity level. Rationale: CI/CD systems depend on predictable exit codes. Traces to: CAP-011, CAP-020.

## Pagination and List Invariants

**DI-019 — List methods must exhaust pagination.** When displaying or reporting tool, resource, or prompt lists, Forge MCP must iterate all pagination cursors until no `nextCursor` is returned. Partial list results must never be presented as complete. Rationale: Servers may paginate results at arbitrary page sizes; displaying only the first page would miss tools/resources. Traces to: CAP-005.

**DI-020 — Tool execution errors are not JSON-RPC errors.** Tool execution failures are returned as successful JSON-RPC responses with `result.isError: true` in the content, not as JSON-RPC error objects. Forge MCP must distinguish between protocol-level errors (JSON-RPC error codes) and tool-level errors (`isError` flag) in traffic display, health metrics, and security analysis. Rationale: Conflating the two leads to incorrect error rate metrics and misleading security findings. Traces to: CAP-005, CAP-009, CAP-013.

## Config Invariants

**DI-015 — Config sources are read-only.** Forge MCP must never modify editor/IDE configuration files. Config sources are read and compared but never written to. Rationale: Modifying a user's editor config without explicit reconciliation consent would violate trust. Traces to: CAP-021, CAP-022.
