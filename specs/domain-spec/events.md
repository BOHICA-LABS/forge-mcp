---
document_type: domain-spec-section
level: L2
section: events
version: "1.2"
status: draft
producer: business-analyst
timestamp: 2026-03-29T11:05:00
phase: 1a
inputs: [product-brief.md, market-intel.md, domain-research]
input-hash: ""
traces_to: L2-INDEX.md
---

# Domain Events

> **Sharded L2 section (DF-021).** Navigate via `L2-INDEX.md`.
> Domain events represent significant state changes in the system. Each event
> has a trigger, preconditions, and downstream effects.
> **v1.1 — Updated from domain research reconciliation.** Added Server-Initiated
> Method Events section covering sampling, elicitation, and task lifecycle.

## Connection Lifecycle Events

**ServerDiscovered** — A new MCP server entry is found during config import. Trigger: Config source parsed and a previously unknown server name extracted. Precondition: Config source file exists and is readable. Outcome: Server added to internal registry with status=disconnected. Consumers: TUI server browser, CLI list command. Traces to: CAP-001.

**ConnectionRequested** — User or system initiates a connection to a registered server. Trigger: User action (TUI select, CLI connect) or daemon auto-reconnect. Precondition: Server exists in registry. Outcome: Transport connection attempt initiated via rmcp. Consumers: Connection manager, session pool. Traces to: CAP-002.

**ConnectionEstablished** — Transport layer reports successful connection. Trigger: rmcp transport handshake completes. Precondition: ConnectionRequested was issued. Outcome: Server status transitions to connecting, capability negotiation begins. Consumers: Session manager. Traces to: CAP-002.

**CapabilityNegotiated** — MCP initialize/initialized handshake completes. Trigger: Server responds to initialize request. Precondition: ConnectionEstablished. Outcome: Both server capabilities (tools, resources, prompts, logging, completions, tasks) and client capabilities (roots, sampling, elicitation) are stored on session. Server status transitions to connected. Protocol methods and server-initiated request handlers become available. Consumers: TUI capability explorer, CLI info command, traffic inspector. Traces to: CAP-004.

**ConnectionLost** — Transport reports unexpected disconnection. Trigger: EOF on stdio, HTTP connection dropped, or heartbeat timeout. Precondition: Session was active. Outcome: Server status transitions to error, pending requests fail, daemon may schedule reconnect. Consumers: Health monitor (error rate), TUI status display, alert system. Traces to: CAP-002, FM-002.

**SessionCreated** — Named persistent session established in daemon pool. Trigger: First connection to a server, or explicit session create command. Precondition: ConnectionEstablished and CapabilityNegotiated. Outcome: Session registered in pool with unique ID, available for CLI and TUI use. Consumers: Daemon session pool. Traces to: CAP-003.

**SessionExpired** — Daemon idle timeout triggers session cleanup. Trigger: No activity on session for configured idle duration. Precondition: Session exists in pool. Outcome: Graceful shutdown sent to server, session removed from pool. Consumers: Daemon lifecycle manager. Traces to: CAP-003.

## Traffic Events

**MessageCaptured** — A JSON-RPC message is observed on a session's transport. Trigger: Any message sent or received on an active session (including server-initiated sampling/elicitation requests). Precondition: Traffic capture is enabled for the session. Outcome: Message appended to traffic capture with timestamp and direction tag. Consumers: Traffic inspector, health metric collector, security auditor. Traces to: CAP-009.

**TrafficFilterApplied** — User applies or changes a filter on captured traffic. Trigger: User enters filter criteria (method, direction, time range, pattern). Precondition: Traffic capture contains messages. Outcome: Filtered view computed and displayed; underlying capture unchanged (DI-006). Consumers: TUI traffic panel, CLI grep command. Traces to: CAP-010.

**MessageReplayed** — A captured message sequence is sent to a target server. Trigger: User initiates replay command with explicit target. Precondition: Capture exists, target server is connected (DI-007). Outcome: Messages dispatched to target, responses captured separately. Consumers: Debugging workflow. Traces to: CAP-010.

## Server-Initiated Method Events (new)

**SamplingRequested** — Server sends a sampling/createMessage request to the client. Trigger: Server needs LLM inference to complete its operation. Precondition: Client advertised sampling capability during negotiation (DI-016). Outcome: Forge MCP proxies the request to configured external LLM API. If no LLM provider configured, returns error to server (DEC-016). On success, returns sampling result to server. Traffic capture records both the request and the proxied response. Consumers: Traffic inspector, TUI sampling indicator, security auditor (may flag suspicious sampling patterns). Traces to: CAP-005, DI-017, FM-016.

**ElicitationRequested** — Server sends an elicitation request asking for user input. Trigger: Server needs user credentials, OAuth, or interactive input. Precondition: Client advertised elicitation capability (DI-016). Outcome: In TUI mode, display form or URL to user. In CLI mode, return error to server (DEC-017). On user response, forward input to server. On timeout, return timeout error (FM-017). Consumers: TUI elicitation panel, traffic inspector. Traces to: CAP-005, DI-017.

**TaskStatusChanged** — A server task transitions between states. Trigger: Server sends task status update (working → completed, working → failed, working → input_required, etc.). Precondition: Task was initiated via a method invocation. Outcome: Task status updated in session state. TUI shows progress indicator. CLI with `--wait` flag checks for completion. Consumers: TUI task panel, CLI wait logic. Traces to: CAP-005, DEC-019.

## Health Monitoring Events

**HealthMetricRecorded** — A new metric data point is derived from observed traffic. Trigger: Periodic aggregation window closes, or message observation updates running counters. Precondition: Health monitoring is active for the session. Outcome: Metric value appended to time series. Consumers: TUI sparkline/histogram, CLI metric export, alert evaluator. Traces to: CAP-013.

**HealthThresholdBreached** — A health metric crosses a configured alert threshold. Trigger: Metric value exceeds threshold for the configured window. Precondition: Alert threshold is configured and in normal state. Outcome: Threshold state transitions to breached, alert event emitted. Consumers: TUI alert indicator, CLI alert output. Traces to: CAP-014, DI-009.

**HealthThresholdRecovered** — A previously breached metric returns to normal range. Trigger: Metric value falls below threshold for configured window. Precondition: Threshold state is breached. Outcome: Threshold state transitions to recovered. Consumers: TUI alert indicator. Traces to: CAP-014, DI-009.

## Security Events

**SecurityFindingDetected** — Runtime analysis identifies a security concern. Trigger: Traffic pattern matches a dangerous behavior rule (e.g., SSRF to 169.254.169.254, dangerous tool invocation, permission escalation). Precondition: Security auditing is active and message has been analyzed. Outcome: Finding created with severity, OWASP AST10 category (AST01-AST10), evidence, and confidence score. Deterministic detections (metadata IP, known-dangerous tool names) get confidence ≥ 0.9; heuristic detections get lower scores. Consumers: TUI security panel, CLI audit report, compliance report generator. Traces to: CAP-016, CAP-017, DI-010.

**SchemaDriftDetected** — Tool, resource, or prompt metadata has changed since the last connection to a server. Trigger: Capability metadata hash comparison on connection detects differences. Precondition: Server was previously connected and metadata hashes were stored. Outcome: Security finding generated with metadata diff evidence. If the change was announced via `list_changed` notification, finding severity is lower (info/medium). If the change was silent (no notification), finding severity is higher (high) — potential rug pull attack (FM-020). Consumers: Security auditor, TUI security panel, traffic inspector. Traces to: CAP-016, CAP-023, FM-020, R-014.

**SecurityFindingSuppressed** — User suppresses a finding as accepted risk. Trigger: Explicit user action (suppress command with reason). Precondition: Finding exists and is not already suppressed. Outcome: Finding retains severity and evidence but is marked suppressed with user attribution (DI-012). Consumers: Compliance report (shows suppressed findings separately). Traces to: CAP-018.

## Conformance Testing Events

**ConformanceTestPassed** — A single conformance test completes with expected behavior. Trigger: Server response matches spec-defined expected behavior. Precondition: Test preconditions met, server connected. Outcome: Test result recorded as pass with evidence. Consumers: Suite aggregator, CI output formatter. Traces to: CAP-019.

**ConformanceTestFailed** — A conformance test detects non-compliant behavior. Trigger: Server response deviates from spec-defined expected behavior. Precondition: Test preconditions met. Outcome: Test result recorded as fail with request/response evidence. Consumers: Suite aggregator, CI output formatter. Traces to: CAP-019.

## Config Events

**ConfigDriftDetected** — Comparison of config sources reveals discrepancies. Trigger: User invokes config diff, or periodic scan completes. Precondition: At least two config sources exist for comparison. Outcome: Drift report generated listing servers with differing parameters across editors. Consumers: TUI config panel, CLI drift report. Traces to: CAP-021.

**ConfigSourceInvalid** — A config source file is missing, unreadable, or malformed. Trigger: Attempted parse of a config file fails. Precondition: Config file path resolved for an editor. Outcome: Config source status set to invalid/missing, warning emitted. Server entries from this source are unavailable. Consumers: Discovery pipeline, TUI status. Traces to: CAP-001, DEC-009.
