---
document_type: epic-registry
level: L3
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
traces_to: prd.md
cycle: v0.1.0-greenfield
---

# Epics — Forge MCP

> Stories are grouped into epics aligned with the 10 architectural subsystems + infrastructure.
> Epic IDs are two-digit (EPIC-00 through EPIC-10) plus SR for spec-review refinements.

---

## EPIC-00: Infrastructure & Testing Foundation

**Crates:** `tests/` (mock servers), CI config, root Cargo.toml
**Priority:** P0
**Wave:** Wave 0
**BCs:** None (infrastructure prerequisite)
**Description:** Establish the Cargo workspace layout, cross-compilation CI pipeline, and both DTU mock MCP servers (stdio + HTTP). All product stories depend on the workspace compiling and at least one mock server existing.

| Story | Title | Wave | Points |
|-------|-------|------|--------|
| STORY-001 | Cargo Workspace Scaffold & CI Pipeline | 0 | 3 |
| STORY-002 | Mock MCP Server (stdio) — forge-test-server | 0 | 3 |
| STORY-003 | Mock MCP Server (HTTP) — forge-test-http-server | 0 | 2 |

**Total:** 3 stories / 8 points

---

## EPIC-01: Server Discovery & Connection

**Crates:** `forge-discovery` (L1), `forge-daemon` (L3), `forge-core` transport layer (L0)
**Priority:** P0
**Waves:** Wave 1 (all)
**BCs:** BC-1.01.001, BC-1.01.002, BC-1.01.003, BC-1.02.001, BC-1.02.002, BC-1.02.003, BC-1.03.001, BC-1.03.002, BC-1.03.003
**NFRs:** NFR-001 (cold start), NFR-008 (cross-platform), NFR-014 (rmcp boundary)
**Description:** Discovers MCP server configs from 4 editors, parses dual JSON schemas, establishes stdio and HTTP transport connections via rmcp, and manages connection lifecycle through the background daemon with session pooling.

| Story | Title | Wave | Points |
|-------|-------|------|--------|
| STORY-004 | Config File Discovery & Path Resolution | 1 | 5 |
| STORY-005 | Dual-Schema Config Parsing (mcpServers vs servers) | 1 | 5 |
| STORY-006 | Config Source Aggregation & Conflict Attribution | 1 | 3 |
| STORY-007 | Stdio Transport Connection Establishment | 1 | 5 |
| STORY-008 | Streamable HTTP Transport Connection Establishment | 1 | 5 |
| STORY-009 | Connection Lifecycle Management | 1 | 3 |
| STORY-010 | Daemon Lazy Start & Session Pooling | 1 | 5 |
| STORY-011 | Named Session Persistence Across CLI Invocations | 1 | 3 |
| STORY-012 | Daemon Socket Conflict Detection & Recovery | 1 | 3 |

**Total:** 9 stories / 37 points

---

## EPIC-02: MCP Protocol Core

**Crates:** `forge-core` (L0 domain kernel)
**Priority:** P0
**Waves:** Wave 1 (negotiation), Wave 2 (operations)
**BCs:** BC-2.04.001–003, BC-2.05.001–010 (13 BCs)
**NFRs:** NFR-011 (spec versions), NFR-013 (pagination), NFR-014 (rmcp boundary)
**VPs:** VP-001, VP-002, VP-003, VP-013
**Description:** Full MCP protocol operations via rmcp Peer API. Capability negotiation, tool/resource/prompt operations with pagination, sampling proxy, elicitation, roots, logging, completion, progress tracking, and error classification.

| Story | Title | Wave | Points |
|-------|-------|------|--------|
| STORY-013 | Bidirectional Capability Negotiation | 1 | 5 |
| STORY-014 | Client Capability Advertisement (Sampling, Elicitation, Roots) | 1 | 5 |
| STORY-015 | Graceful Degradation with Older Spec Versions | 1 | 3 |
| STORY-016 | Tool List & Invocation with Pagination | 2 | 8 |
| STORY-017 | Resource List, Read & Subscription Management | 2 | 5 |
| STORY-018 | Prompt List & Retrieval with Pagination | 2 | 5 |
| STORY-019 | Sampling Proxy to External LLM | 2 | 5 |
| STORY-020 | Elicitation Request Handling (Form + URL Modes) | 2 | 5 |
| STORY-021 | Roots, Logging, Completion & Protocol Utilities | 2 | 5 |
| STORY-022 | Progress Tracking, Cancellation & Error Distinction | 2 | 5 |

**Total:** 10 stories / 51 points

---

## EPIC-03: TUI Dashboard

**Crates:** `forge-tui` (L3)
**Priority:** P0
**Wave:** Wave 4
**BCs:** BC-3.06.001–002, BC-3.07.001–003, BC-3.08.001–005 (10 BCs)
**NFRs:** NFR-002 (60fps), NFR-015 (accessibility)
**VPs:** VP-012
**UX:** SCR-001 through SCR-006 (P0 screens)
**Description:** ratatui-based interactive TUI dashboard. Multi-pane adaptive layout, color system, vi-style navigation, search/command mode, mouse input, JSON-RPC rendering, health sparklines, server browser, capability explorer, and accessibility.

| Story | Title | Wave | Points |
|-------|-------|------|--------|
| STORY-037 | TUI Main Dashboard Layout & Adaptive Panes | 4 | 8 |
| STORY-038 | Color System Auto-Detection & Degradation | 4 | 3 |
| STORY-039 | Vi-Style Keyboard Navigation | 4 | 5 |
| STORY-040 | Search & Command Mode (/ and :) | 4 | 5 |
| STORY-041 | Mouse Supplementary Input | 4 | 3 |
| STORY-042 | JSON-RPC Syntax-Highlighted Message Rendering | 4 | 5 |
| STORY-043 | Sparkline & Histogram Health Metric Visualization | 4 | 5 |
| STORY-044 | Server Browser with Status Badges | 4 | 5 |
| STORY-045 | Capability Explorer (Tools/Resources/Prompts Tree) | 4 | 5 |

**Total:** 9 stories / 44 points

---

## EPIC-04: Traffic Inspection

**Crates:** `forge-traffic` (L1)
**Priority:** P0
**Wave:** Wave 3
**BCs:** BC-4.09.001–003, BC-4.10.001–003 (6 BCs)
**NFRs:** NFR-012 (bounded memory)
**VPs:** VP-005, VP-006, VP-014
**UX:** SCR-004 (Traffic Inspector), FLOW-003
**Description:** Transparent JSON-RPC message capture into a bounded ring buffer, per-message timing analysis, traffic filtering, full-text search, and message sequence replay.

| Story | Title | Wave | Points |
|-------|-------|------|--------|
| STORY-027 | Transparent JSON-RPC Message Capture | 3 | 5 |
| STORY-028 | Per-Message Timing & Throughput Analysis | 3 | 5 |
| STORY-029 | Capture Buffer Management with Bounded Memory | 3 | 5 |
| STORY-030 | Traffic Filtering by Method, Direction, Time & Content | 3 | 5 |
| STORY-031 | Full-Text Payload Search | 3 | 3 |
| STORY-032 | Message Sequence Replay Against Target Server | 3 | 5 |

**Total:** 6 stories / 28 points

---

## EPIC-05: CLI Mode

**Crates:** `forge-mcp` binary (L4), `forge-cli` dispatch
**Priority:** P0
**Wave:** Wave 2
**BCs:** BC-5.11.001–003, BC-5.12.001–002 (5 BCs)
**NFRs:** NFR-001 (cold start), NFR-003 (token efficiency)
**Description:** Thin CLI binary entry point. clap-derived subcommand dispatch (list, call, info, grep, test, audit, tui, daemon), structured JSON stdout, exit code compliance, and agent-optimized minimal token output.

| Story | Title | Wave | Points |
|-------|-------|------|--------|
| STORY-023 | CLI Subcommand Dispatch & Exit Code Semantics | 2 | 5 |
| STORY-024 | Structured JSON Output & Agent-Optimized Tokens | 2 | 5 |
| STORY-025 | Pipeable Output & Shell Composition | 2 | 3 |

**Total:** 3 stories / 13 points

---

## EPIC-06: Health Monitoring

**Crates:** `forge-health` (L1)
**Priority:** P0
**Waves:** Wave 3 (core metrics + alerting), Wave 4 (TUI visualization)
**BCs:** BC-6.13.001–002, BC-6.14.001–002, BC-6.15.001–002 (6 BCs)
**VPs:** VP-007, VP-008
**UX:** SCR-005 (Health Metrics Panel), FLOW-004
**Description:** Passive latency/throughput collection, windowed error rate tracking, configurable alert thresholds, alert state machine, TUI time-series visualization, and CLI metric snapshot export.

| Story | Title | Wave | Points |
|-------|-------|------|--------|
| STORY-033 | Passive Latency & Throughput Metric Collection | 3 | 5 |
| STORY-034 | Error Rate Trend Tracking | 3 | 5 |
| STORY-035 | Configurable Alerting Thresholds | 3 | 5 |
| STORY-036 | Alert State Machine (Normal → Breached → Recovered) | 3 | 5 |
| STORY-026 | Metric Snapshot JSON Export via CLI | 3 | 3 |
| STORY-046 | Time-Series Metric Visualization in TUI | 4 | 5 |

**Total:** 6 stories / 28 points

---

## EPIC-07: Security Auditing

**Crates:** `forge-security` (L2)
**Priority:** P1
**Wave:** Wave 5
**BCs:** BC-7.16.001–004, BC-7.17.001–003, BC-7.18.001–003 (10 BCs)
**NFRs:** NFR-005 (>80% precision), NFR-006 (OWASP ≥83%), NFR-007 (schema integrity)
**VPs:** VP-009, VP-010, VP-011
**UX:** SCR-007 (Security Audit View), FLOW-005
**Description:** Runtime behavioral analysis of live MCP traffic. Dangerous tool pattern detection, SSRF detection, schema drift/rug pull detection, permission escalation, auth validation, structured report generation, OWASP AST10 mapping, and user-defined suppression rules.

| Story | Title | Wave | Points |
|-------|-------|------|--------|
| STORY-047 | Dangerous Tool Pattern Detection | 5 | 5 |
| STORY-048 | SSRF Attempt Detection | 5 | 5 |
| STORY-049 | Schema Drift / Rug Pull Detection | 5 | 5 |
| STORY-050 | Permission Escalation & Root Enforcement Detection | 5 | 5 |
| STORY-051 | Authentication Handling Validation | 5 | 3 |
| STORY-052 | Security Audit Report Generation & OWASP Mapping | 5 | 8 |
| STORY-053 | Security Finding Suppression Rules | 5 | 3 |
| STORY-054 | Security Audit TUI View (SCR-007) | 5 | 5 |

**Total:** 8 stories / 39 points

---

## EPIC-08: Conformance Testing

**Crates:** `forge-conformance` (L2)
**Priority:** P1
**Wave:** Wave 6
**BCs:** BC-8.19.001–003, BC-8.20.001–002 (5 BCs)
**NFRs:** NFR-010 (≥90% method coverage), NFR-011 (spec versions)
**UX:** SCR-008 (Conformance Test Runner)
**Description:** Automated MCP protocol conformance suite with capability negotiation validation, method coverage, transport compliance, and JUnit XML + JSON output for CI/CD pipelines.

| Story | Title | Wave | Points |
|-------|-------|------|--------|
| STORY-055 | Conformance Capability Negotiation Validation | 6 | 5 |
| STORY-056 | Conformance Method Coverage & Error Handling | 6 | 5 |
| STORY-057 | Conformance Transport Compliance | 6 | 3 |
| STORY-058 | Conformance Report Output (JUnit XML + JSON) | 6 | 5 |

**Total:** 4 stories / 18 points

---

## EPIC-09: Config Drift Detection

**Crates:** `forge-config` (L2)
**Priority:** P1/P2
**Wave:** Wave 6
**BCs:** BC-9.21.001–002, BC-9.22.001 (3 BCs)
**UX:** SCR-009 (Config Drift View)
**Description:** Cross-editor config comparison and drift report generation. Advisory reconciliation guidance for divergent configurations.

| Story | Title | Wave | Points |
|-------|-------|------|--------|
| STORY-059 | Cross-Editor Config Comparison & Drift Report | 6 | 8 |
| STORY-060 | Config Drift Reconciliation Workflow Guidance | 6 | 3 |

**Total:** 2 stories / 11 points

---

## EPIC-10: Server Comparison

**Crates:** `forge-config` (L2, reused for diff), `forge-core` (L0)
**Priority:** P2
**Wave:** Wave 6
**BCs:** BC-10.23.001, BC-10.24.001, BC-10.25.001 (3 BCs)
**UX:** SCR-010 (Server Comparison View)
**Description:** Side-by-side comparison of two MCP servers: tool schema diffs, capability set deltas, and response behavior delta testing.

| Story | Title | Wave | Points |
|-------|-------|------|--------|
| STORY-061 | Server Tool Schema Diff | 6 | 5 |
| STORY-062 | Capability Set Delta Comparison | 6 | 3 |
| STORY-063 | Response Behavior Delta Testing | 6 | 5 |

**Total:** 3 stories / 13 points

---

## SR: Spec Review Refinements

**Priority:** P0/P1
**Wave:** Wave 6
**Addresses:** SR-001, SR-002, SR-003, SR-004, SR-005
**Description:** Inline refinements incorporated into relevant stories (SR-001/SR-004/SR-005 into STORY-005; SR-002 into STORY-047; SR-003 into STORY-037). STORY-065 consolidates remaining spec clarifications into living spec amendments.

| Story | Title | Wave | Points |
|-------|-------|------|--------|
| STORY-064 | NFR Validation Suite & Performance Benchmarks | 6 | 5 |
| STORY-065 | SR Refinements: Config Struct Clarity & Security Heuristics | 6 | 3 |

**Total:** 2 stories / 8 points

> **SR Inline Addresses:**
> - SR-001 (transport fields clarity) → addressed inline in STORY-005
> - SR-002 (security heuristics examples) → addressed inline in STORY-047
> - SR-003 (TUI frame rate measurement) → addressed inline in STORY-037
> - SR-004 (missing schema behavior) → addressed inline in STORY-005
> - SR-005 (DiscoveredConfig properties) → addressed inline in STORY-004 and STORY-005
