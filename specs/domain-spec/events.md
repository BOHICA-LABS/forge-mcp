---
document_type: domain-spec-section
level: L2
section: events
version: "1.0"
status: draft
producer: business-analyst
timestamp: 2026-03-29T10:55:00
phase: 1a
inputs: [product-brief.md, market-intel.md]
input-hash: ""
traces_to: L2-INDEX.md
---

# Domain Events

> **Sharded L2 section (DF-021).** Navigate via `L2-INDEX.md`.
> Domain events represent significant state changes in the system. Each event
> has a trigger, preconditions, and downstream effects.

## Connection Lifecycle Events

**ServerDiscovered** — A new MCP server entry is found during config import. Trigger: Config source parsed and a previously unknown server name extracted. Precondition: Config source file exists and is readable. Outcome: Server added to internal registry with status=disconnected. Consumers: TUI server browser, CLI list command. Traces to: CAP-001.

**ConnectionRequested** — User or system initiates a connection to a registered server. Trigger: User action (TUI select, CLI connect) or daemon auto-reconnect. Precondition: Server exists in registry. Outcome: Transport connection attempt initiated via rmcp. Consumers: Connection manager, session pool. Traces to: CAP-002.

**ConnectionEstablished** — Transport layer reports successful connection. Trigger: rmcp transport handshake completes. Precondition: ConnectionRequested was issued. Outcome: Server status transitions to connecting, capability negotiation begins. Consumers: Session manager. Traces to: CAP-002.

**CapabilityNegotiated** — MCP initialize/initialized handshake completes. Trigger: Server responds to initialize request. Precondition: ConnectionEstablished. Outcome: Negotiated capabilities stored on session, server status transitions to connected, protocol methods become available. Consumers: TUI capability explorer, CLI info command, traffic inspector. Traces to: CAP-004.

**ConnectionLost** — Transport reports unexpected disconnection. Trigger: EOF on stdio, HTTP connection dropped, or heartbeat timeout. Precondition: Session was active. Outcome: Server status transitions to error, pending requests fail, daemon may schedule reconnect. Consumers: Health monitor (error rate), TUI status display, alert system. Traces to: CAP-002, FM-002.

**SessionCreated** — Named persistent session established in daemon pool. Trigger: First connection to a server, or explicit session create command. Precondition: ConnectionEstablished and CapabilityNegotiated. Outcome: Session registered in pool with unique ID, available for CLI and TUI use. Consumers: Daemon session pool. Traces to: CAP-003.

**SessionExpired** — Daemon idle timeout triggers session cleanup. Trigger: No activity on session for configured idle duration. Precondition: Session exists in pool. Outcome: Graceful shutdown sent to server, session removed from pool. Consumers: Daemon lifecycle manager. Traces to: CAP-003.

## Traffic Events

**MessageCaptured** — A JSON-RPC message is observed on a session's transport. Trigger: Any message sent or received on an active session. Precondition: Traffic capture is enabled for the session. Outcome: Message appended to traffic capture with timestamp. Consumers: Traffic inspector, health metric collector, security auditor. Traces to: CAP-009.

**TrafficFilterApplied** — User applies or changes a filter on captured traffic. Trigger: User enters filter criteria (method, direction, time range, pattern). Precondition: Traffic capture contains messages. Outcome: Filtered view computed and displayed; underlying capture unchanged (DI-006). Consumers: TUI traffic panel, CLI grep command. Traces to: CAP-010.

**MessageReplayed** — A captured message sequence is sent to a target server. Trigger: User initiates replay command with explicit target. Precondition: Capture exists, target server is connected (DI-007). Outcome: Messages dispatched to target, responses captured separately. Consumers: Debugging workflow. Traces to: CAP-010.

## Health Monitoring Events

**HealthMetricRecorded** — A new metric data point is derived from observed traffic. Trigger: Periodic aggregation window closes, or message observation updates running counters. Precondition: Health monitoring is active for the session. Outcome: Metric value appended to time series. Consumers: TUI sparkline/histogram, CLI metric export, alert evaluator. Traces to: CAP-013.

**HealthThresholdBreached** — A health metric crosses a configured alert threshold. Trigger: Metric value exceeds threshold for the configured window. Precondition: Alert threshold is configured and in normal state. Outcome: Threshold state transitions to breached, alert event emitted. Consumers: TUI alert indicator, CLI alert output. Traces to: CAP-014, DI-009.

**HealthThresholdRecovered** — A previously breached metric returns to normal range. Trigger: Metric value falls below threshold for configured window. Precondition: Threshold state is breached. Outcome: Threshold state transitions to recovered. Consumers: TUI alert indicator. Traces to: CAP-014, DI-009.

## Security Events

**SecurityFindingDetected** — Runtime analysis identifies a security concern. Trigger: Traffic pattern matches a dangerous behavior rule. Precondition: Security auditing is active and message has been analyzed. Outcome: Finding created with severity, OWASP AST10 category, evidence, and confidence score. Consumers: TUI security panel, CLI audit report, compliance report generator. Traces to: CAP-016, CAP-017, DI-010.

**SecurityFindingSuppressed** — User suppresses a finding as accepted risk. Trigger: Explicit user action (suppress command with reason). Precondition: Finding exists and is not already suppressed. Outcome: Finding retains severity and evidence but is marked suppressed with user attribution (DI-012). Consumers: Compliance report (shows suppressed findings separately). Traces to: CAP-018.

## Conformance Testing Events

**ConformanceTestPassed** — A single conformance test completes with expected behavior. Trigger: Server response matches spec-defined expected behavior. Precondition: Test preconditions met, server connected. Outcome: Test result recorded as pass with evidence. Consumers: Suite aggregator, CI output formatter. Traces to: CAP-019.

**ConformanceTestFailed** — A conformance test detects non-compliant behavior. Trigger: Server response deviates from spec-defined expected behavior. Precondition: Test preconditions met. Outcome: Test result recorded as fail with request/response evidence. Consumers: Suite aggregator, CI output formatter. Traces to: CAP-019.

## Config Events

**ConfigDriftDetected** — Comparison of config sources reveals discrepancies. Trigger: User invokes config diff, or periodic scan completes. Precondition: At least two config sources exist for comparison. Outcome: Drift report generated listing servers with differing parameters across editors. Consumers: TUI config panel, CLI drift report. Traces to: CAP-021.

**ConfigSourceInvalid** — A config source file is missing, unreadable, or malformed. Trigger: Attempted parse of a config file fails. Precondition: Config file path resolved for an editor. Outcome: Config source status set to invalid/missing, warning emitted. Server entries from this source are unavailable. Consumers: Discovery pipeline, TUI status. Traces to: CAP-001, DEC-009.
