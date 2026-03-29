---
document_type: architecture-section
level: L3
section: module-decomposition
version: "1.0"
status: draft
producer: architect
timestamp: 2026-03-29T14:05:00
phase: 1b
inputs: [domain-spec/L2-INDEX.md, specs/prd.md]
traces_to: ARCH-INDEX.md
---

# Module Decomposition

10 crates in a Cargo workspace. Each crate maps to one or more L2 subsystems.

## forge-core (S2: Protocol Operations)

Protocol adapter over rmcp. Thin wrapper providing Forge-specific types around rmcp's Peer API.

**Contains:** MCP message types, capability negotiation wrapper, protocol method dispatch, error mapping (DI-020: tool errors vs protocol errors), pagination iteration (DI-019). Client capability builder (DI-016). ClientHandler implementation for sampling proxy, elicitation, roots (DI-017).

**Pure core:** message type definitions, error classification, pagination state.
**Effectful shell:** rmcp Peer calls, network I/O.

**Traces to:** CAP-004, CAP-005, DI-001, DI-002, DI-004, DI-016, DI-017, DI-018, DI-019, DI-020.

## forge-discovery (S1: Server Discovery & Config Import)

Config file parsing for 4 editors.

**Pure core:** JSON schema parsing (mcpServers vs servers), path resolution per OS, config normalization, conflict detection.
**Effectful shell:** filesystem reads, environment variable expansion.

**Produces:** unified ServerRegistry.

**Traces to:** CAP-001, DI-015, BC-1.01.001 through BC-1.01.003.

## forge-daemon (S1: Connection Management & Sessions)

Background daemon process with session pooling.

**Effectful:** Unix domain socket / Windows named pipe listener, session lifecycle management, lazy start, idle timeout. Uses forge-core for connections.

**Traces to:** CAP-002, CAP-003, BC-1.02.001 through BC-1.03.003.

## forge-tui (S3: TUI Dashboard)

ratatui-based interactive dashboard.

**Pure core:** TUI state machine (panel focus, navigation state, input mode), layout calculation, color system selection.
**Effectful shell:** crossterm terminal I/O, event polling, frame rendering.

**Widgets:** server browser, capability explorer, traffic inspector, health metrics, security findings, JSON detail popup.

**Traces to:** CAP-006, CAP-007, CAP-008, BC-3.06.001 through BC-3.08.005.

## forge-traffic (S4: Traffic Inspection)

Message capture and analysis.

**Pure core:** ring buffer data structure, message filtering (method, direction, time, content), full-text search, timing analysis (per-message latency, throughput calculation), message ordering validation (DI-006).
**Effectful shell:** disk spill for overflow, replay dispatch via forge-core.

**Traces to:** CAP-009, CAP-010, DI-005, DI-006, DI-007, NFR-012.

## forge-health (S6: Health Monitoring)

Metrics and alerting.

**Pure core:** latency histogram calculation, error rate windowed computation, throughput counters, alert state machine (normal→breached→recovered per DI-009), metric snapshot serialization.
**Effectful shell:** timer ticks for window rotation.

**Traces to:** CAP-013, CAP-014, CAP-015, DI-008, DI-009.

## forge-security (S7: Security Auditing)

Runtime behavioral analysis.

**Pure core:** rule engine (pattern matching on message content), SSRF detection (IP classification), dangerous tool detection, schema drift detection (hash comparison), finding severity classification with confidence scoring (DI-010, DI-011), suppression overlay (DI-012), OWASP AST10 mapping.
**Effectful shell:** rule file loading.

**Traces to:** CAP-016, CAP-017, CAP-018, DI-010, DI-011, DI-012, R-004, R-014.

## forge-conformance (S8: Conformance Testing)

Protocol compliance testing.

**Pure core:** spec assertions (expected vs actual behavior), test case definitions per MCP method, result aggregation.
**Effectful shell:** test execution against live server via forge-core, JUnit XML / JSON serialization.

**Traces to:** CAP-019, CAP-020, NFR-010.

## forge-config (S9: Config Drift + S10: Server Comparison)

Config and server diffing.

**Pure core:** cross-editor config comparison, drift report generation, tool schema diffing, capability delta computation, reconciliation guidance.
**Effectful shell:** none (operates on in-memory data from forge-discovery).

**Traces to:** CAP-021, CAP-022, CAP-023, CAP-024, CAP-025.

## forge-mcp (binary crate — S5: CLI Mode)

Thin binary crate. clap-derived subcommand dispatch.

**Pure core:** output formatting (JSON, text, table), token-optimized output (NFR-003), exit code mapping (DI-014).
**Effectful shell:** stdout/stderr I/O, signal handling, panic hook.

**Traces to:** CAP-011, CAP-012, DI-013, DI-014.
