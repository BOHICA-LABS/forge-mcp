---
document_type: story-index
level: L3
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
traces_to: prd.md
cycle: v0.1.0-greenfield
---

# Story Index — Forge MCP

> **Phase 2 Story Decomposition**
> 65 BCs → 60 implementation stories + 5 SR refinement stories = 65 total stories.
> All stories sharded: one file per story in `.factory/stories/stories/`.

## Summary

| Metric | Value |
|--------|-------|
| Total stories | 65 |
| P0 stories | 46 |
| P1 stories | 16 |
| P2 stories | 6 |
| Waves | 7 (Wave 0–Wave 6) |
| Epics | 12 (EPIC-00 through EPIC-10 + SR refinements) |
| Total story points | 276 |

## Epic Story Counts

| Epic | Name | Stories | Points | Priority |
|------|------|---------|--------|----------|
| EPIC-00 | Infrastructure & Testing Foundation | 3 | 10 | P0 |
| EPIC-01 | Server Discovery & Connection | 9 | 38 | P0 |
| EPIC-02 | MCP Protocol Core | 9 | 47 | P0 |
| EPIC-03 | TUI Dashboard | 9 | 47 | P0 |
| EPIC-04 | Traffic Inspection | 6 | 28 | P0 |
| EPIC-05 | CLI Mode | 5 | 21 | P0 |
| EPIC-06 | Health Monitoring | 6 | 27 | P0 |
| EPIC-07 | Security Auditing | 7 | 36 | P1 |
| EPIC-08 | Conformance Testing | 4 | 18 | P1 |
| EPIC-09 | Config Drift Detection | 3 | 12 | P1 |
| EPIC-10 | Server Comparison | 3 | 12 | P2 |
| SR | Spec Review Refinements | 1 | 5 | P0/P1 |

---

## Full Story Registry

| Story ID | Title | Epic | Wave | Points | Priority | Depends On | Status |
|----------|-------|------|------|--------|----------|------------|--------|
| STORY-001 | Cargo Workspace Scaffold & CI Pipeline | EPIC-00 | 0 | 3 | P0 | -- | draft |
| STORY-002 | Mock MCP Server (stdio) — forge-test-server | EPIC-00 | 0 | 3 | P0 | STORY-001 | draft |
| STORY-003 | Mock MCP Server (HTTP) — forge-test-http-server | EPIC-00 | 0 | 2 | P0 | STORY-001 | draft |
| STORY-004 | Config File Discovery & Path Resolution | EPIC-01 | 1 | 5 | P0 | STORY-001 | draft |
| STORY-005 | Dual-Schema Config Parsing (mcpServers vs servers) | EPIC-01 | 1 | 5 | P0 | STORY-004 | draft |
| STORY-006 | Config Source Aggregation & Conflict Attribution | EPIC-01 | 1 | 3 | P0 | STORY-005 | draft |
| STORY-007 | Stdio Transport Connection Establishment | EPIC-01 | 1 | 5 | P0 | STORY-002, STORY-004 | draft |
| STORY-008 | Streamable HTTP Transport Connection Establishment | EPIC-01 | 1 | 5 | P0 | STORY-003, STORY-004 | draft |
| STORY-009 | Connection Lifecycle Management (Keepalive & Shutdown) | EPIC-01 | 1 | 3 | P0 | STORY-007, STORY-008 | draft |
| STORY-010 | Daemon Lazy Start & Session Pooling | EPIC-01 | 1 | 5 | P0 | STORY-007, STORY-008 | draft |
| STORY-011 | Named Session Persistence Across CLI Invocations | EPIC-01 | 1 | 3 | P0 | STORY-010 | draft |
| STORY-012 | Daemon Socket Conflict Detection & Recovery | EPIC-01 | 1 | 3 | P0 | STORY-010 | draft |
| STORY-013 | Bidirectional Capability Negotiation | EPIC-02 | 1 | 5 | P0 | STORY-007, STORY-008 | draft |
| STORY-014 | Client Capability Advertisement (Sampling, Elicitation, Roots) | EPIC-02 | 1 | 5 | P0 | STORY-013 | draft |
| STORY-015 | Graceful Degradation with Older Spec Versions | EPIC-02 | 1 | 3 | P0 | STORY-013 | draft |
| STORY-016 | Tool List & Invocation with Pagination | EPIC-02 | 2 | 8 | P0 | STORY-013 | draft |
| STORY-017 | Resource List, Read & Subscription Management | EPIC-02 | 2 | 5 | P0 | STORY-013 | draft |
| STORY-018 | Prompt List & Retrieval with Pagination | EPIC-02 | 2 | 5 | P0 | STORY-013 | draft |
| STORY-019 | Sampling Proxy to External LLM | EPIC-02 | 2 | 5 | P0 | STORY-014 | draft |
| STORY-020 | Elicitation Request Handling (Form + URL Modes) | EPIC-02 | 2 | 5 | P0 | STORY-014 | draft |
| STORY-021 | Roots, Logging, Completion & Protocol Utilities | EPIC-02 | 2 | 5 | P0 | STORY-013 | draft |
| STORY-022 | Progress Tracking, Cancellation & Error Distinction | EPIC-02 | 2 | 5 | P0 | STORY-016 | draft |
| STORY-023 | CLI Subcommand Dispatch & Exit Code Semantics | EPIC-05 | 2 | 5 | P0 | STORY-013 | draft |
| STORY-024 | Structured JSON Output & Agent-Optimized Tokens | EPIC-05 | 2 | 5 | P0 | STORY-023 | draft |
| STORY-025 | Pipeable Output & Shell Composition | EPIC-05 | 2 | 3 | P0 | STORY-024 | draft |
| STORY-026 | Metric Snapshot JSON Export via CLI | EPIC-06 | 2 | 3 | P0 | STORY-023, STORY-030 | draft |
| STORY-027 | Transparent JSON-RPC Message Capture | EPIC-04 | 3 | 5 | P0 | STORY-013 | draft |
| STORY-028 | Per-Message Timing & Throughput Analysis | EPIC-04 | 3 | 5 | P0 | STORY-027 | draft |
| STORY-029 | Capture Buffer Management with Bounded Memory | EPIC-04 | 3 | 5 | P0 | STORY-027 | draft |
| STORY-030 | Traffic Filtering by Method, Direction, Time & Content | EPIC-04 | 3 | 5 | P0 | STORY-027 | draft |
| STORY-031 | Full-Text Payload Search | EPIC-04 | 3 | 3 | P0 | STORY-029 | draft |
| STORY-032 | Message Sequence Replay Against Target Server | EPIC-04 | 3 | 5 | P0 | STORY-029 | draft |
| STORY-033 | Passive Latency & Throughput Metric Collection | EPIC-06 | 3 | 5 | P0 | STORY-027 | draft |
| STORY-034 | Error Rate Trend Tracking | EPIC-06 | 3 | 5 | P0 | STORY-033 | draft |
| STORY-035 | Configurable Alerting Thresholds | EPIC-06 | 3 | 5 | P0 | STORY-033 | draft |
| STORY-036 | Alert State Machine (Normal → Breached → Recovered) | EPIC-06 | 3 | 5 | P0 | STORY-035 | draft |
| STORY-037 | TUI Main Dashboard Layout & Adaptive Panes | EPIC-03 | 4 | 8 | P0 | STORY-006, STORY-013 | draft |
| STORY-038 | Color System Auto-Detection & Degradation | EPIC-03 | 4 | 3 | P0 | STORY-037 | draft |
| STORY-039 | Vi-Style Keyboard Navigation | EPIC-03 | 4 | 5 | P0 | STORY-037 | draft |
| STORY-040 | Search & Command Mode (/ and :) | EPIC-03 | 4 | 5 | P0 | STORY-039 | draft |
| STORY-041 | Mouse Supplementary Input | EPIC-03 | 4 | 3 | P0 | STORY-039 | draft |
| STORY-042 | JSON-RPC Syntax-Highlighted Message Rendering | EPIC-03 | 4 | 5 | P0 | STORY-037, STORY-027 | draft |
| STORY-043 | Sparkline & Histogram Health Metric Visualization | EPIC-03 | 4 | 5 | P0 | STORY-037, STORY-033 | draft |
| STORY-044 | Server Browser with Status Badges | EPIC-03 | 4 | 5 | P0 | STORY-037, STORY-006 | draft |
| STORY-045 | Capability Explorer (Tools/Resources/Prompts Tree) | EPIC-03 | 4 | 5 | P0 | STORY-037, STORY-016 | draft |
| STORY-046 | Time-Series Metric Visualization in TUI | EPIC-06 | 4 | 5 | P0 | STORY-043, STORY-033 | draft |
| STORY-047 | Dangerous Tool Pattern Detection | EPIC-07 | 5 | 5 | P1 | STORY-027 | draft |
| STORY-048 | SSRF Attempt Detection | EPIC-07 | 5 | 5 | P1 | STORY-027 | draft |
| STORY-049 | Schema Drift / Rug Pull Detection | EPIC-07 | 5 | 5 | P1 | STORY-027 | draft |
| STORY-050 | Permission Escalation & Root Enforcement Detection | EPIC-07 | 5 | 5 | P1 | STORY-047 | draft |
| STORY-051 | Authentication Handling Validation | EPIC-07 | 5 | 3 | P1 | STORY-048 | draft |
| STORY-052 | Security Audit Report Generation & OWASP Mapping | EPIC-07 | 5 | 8 | P1 | STORY-047, STORY-048, STORY-049 | draft |
| STORY-053 | Security Finding Suppression Rules | EPIC-07 | 5 | 3 | P1 | STORY-052 | draft |
| STORY-054 | Security Audit TUI View (SCR-007) | EPIC-07 | 5 | 5 | P1 | STORY-052, STORY-037 | draft |
| STORY-055 | Conformance Capability Negotiation Validation | EPIC-08 | 6 | 5 | P1 | STORY-013, STORY-002 | draft |
| STORY-056 | Conformance Method Coverage & Error Handling | EPIC-08 | 6 | 5 | P1 | STORY-055 | draft |
| STORY-057 | Conformance Transport Compliance | EPIC-08 | 6 | 3 | P1 | STORY-055, STORY-003 | draft |
| STORY-058 | Conformance Report Output (JUnit XML + JSON) | EPIC-08 | 6 | 5 | P1 | STORY-055 | draft |
| STORY-059 | Cross-Editor Config Comparison & Drift Report | EPIC-09 | 6 | 8 | P1 | STORY-006 | draft |
| STORY-060 | Config Drift Reconciliation Workflow Guidance | EPIC-09 | 6 | 3 | P2 | STORY-059 | draft |
| STORY-061 | Server Tool Schema Diff | EPIC-10 | 6 | 5 | P2 | STORY-016, STORY-006 | draft |
| STORY-062 | Capability Set Delta Comparison | EPIC-10 | 6 | 3 | P2 | STORY-013 | draft |
| STORY-063 | Response Behavior Delta Testing | EPIC-10 | 6 | 5 | P2 | STORY-016 | draft |
| STORY-064 | NFR Validation Suite & Performance Benchmarks | SR | 6 | 5 | P0 | STORY-024, STORY-033, STORY-029 | draft |
| STORY-065 | SR Refinements: Config Struct Clarity & Security Heuristics Reference | SR | 6 | 3 | P0 | STORY-005, STORY-047 | draft |

---

## Wave Summary

| Wave | Stories | Points | Theme |
|------|---------|--------|-------|
| Wave 0 | STORY-001, 002, 003 | 8 | Infrastructure: workspace + mock servers |
| Wave 1 | STORY-004–015 | 60 | Core protocol + discovery (no UI) |
| Wave 2 | STORY-016–026 | 46 | Protocol operations + CLI mode |
| Wave 3 | STORY-027–036 | 43 | Traffic inspection + health monitoring |
| Wave 4 | STORY-037–046 | 49 | TUI dashboard |
| Wave 5 | STORY-047–054 | 39 | Security auditing |
| Wave 6 | STORY-055–065 | 42 | Conformance, config drift, comparison, NFR validation |
