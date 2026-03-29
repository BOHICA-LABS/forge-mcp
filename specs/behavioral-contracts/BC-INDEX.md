---
document_type: behavioral-contract-index
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-03-29T11:25:00
phase: 1a
traces_to: ../prd.md
---

# Behavioral Contract Index: Forge MCP

> Master index of all behavioral contracts organized by subsystem.
> Each BC file follows the `BC-S.SS.NNN.md` naming convention.
> Total: 65 behavioral contracts across 10 subsystems.

## Subsystem Summary

| S | Subsystem | CAPs | BC Count | Priority |
|---|-----------|------|----------|----------|
| 1 | Server Discovery & Connection Management | CAP-001, CAP-002, CAP-003 | 9 | P0 |
| 2 | MCP Protocol Operations | CAP-004, CAP-005 | 13 | P0 |
| 3 | TUI Dashboard | CAP-006, CAP-007, CAP-008 | 10 | P0 |
| 4 | Traffic Inspection | CAP-009, CAP-010 | 6 | P0 |
| 5 | CLI Mode | CAP-011, CAP-012 | 5 | P0 |
| 6 | Health Monitoring | CAP-013, CAP-014, CAP-015 | 6 | P0 |
| 7 | Security Auditing | CAP-016, CAP-017, CAP-018 | 10 | P1 |
| 8 | Conformance Testing | CAP-019, CAP-020 | 5 | P1 |
| 9 | Config Drift Detection | CAP-021, CAP-022 | 3 | P1/P2 |
| 10 | Server Comparison | CAP-023, CAP-024, CAP-025 | 3 | P2 |

## S=1: Server Discovery & Connection Management

| BC ID | File | Title | Priority | CAP |
|-------|------|-------|----------|-----|
| BC-1.01.001 | BC-1.01.001.md | Config file discovery and path resolution | P0 | CAP-001 |
| BC-1.01.002 | BC-1.01.002.md | Dual-schema config parsing (mcpServers vs servers) | P0 | CAP-001 |
| BC-1.01.003 | BC-1.01.003.md | Config source aggregation with conflict attribution | P0 | CAP-001 |
| BC-1.02.001 | BC-1.02.001.md | Stdio transport connection establishment | P0 | CAP-002 |
| BC-1.02.002 | BC-1.02.002.md | Streamable HTTP transport connection establishment | P0 | CAP-002 |
| BC-1.02.003 | BC-1.02.003.md | Connection lifecycle management | P0 | CAP-002 |
| BC-1.03.001 | BC-1.03.001.md | Daemon lazy start and session pooling | P0 | CAP-003 |
| BC-1.03.002 | BC-1.03.002.md | Named session persistence across CLI invocations | P0 | CAP-003 |
| BC-1.03.003 | BC-1.03.003.md | Daemon socket conflict detection and recovery | P0 | CAP-003 |

## S=2: MCP Protocol Operations

| BC ID | File | Title | Priority | CAP |
|-------|------|-------|----------|-----|
| BC-2.04.001 | BC-2.04.001.md | Bidirectional capability negotiation | P0 | CAP-004 |
| BC-2.04.002 | BC-2.04.002.md | Client capability advertisement | P0 | CAP-004 |
| BC-2.04.003 | BC-2.04.003.md | Graceful degradation with older spec versions | P0 | CAP-004 |
| BC-2.05.001 | BC-2.05.001.md | Tool list and invocation with pagination | P0 | CAP-005 |
| BC-2.05.002 | BC-2.05.002.md | Resource list, read, and subscription management | P0 | CAP-005 |
| BC-2.05.003 | BC-2.05.003.md | Prompt list and retrieval with pagination | P0 | CAP-005 |
| BC-2.05.004 | BC-2.05.004.md | Sampling proxy to external LLM | P0 | CAP-005 |
| BC-2.05.005 | BC-2.05.005.md | Elicitation request handling | P0 | CAP-005 |
| BC-2.05.006 | BC-2.05.006.md | Roots list response and change notification | P0 | CAP-005 |
| BC-2.05.007 | BC-2.05.007.md | Logging level control and message display | P0 | CAP-005 |
| BC-2.05.008 | BC-2.05.008.md | Completion/autocomplete requests | P0 | CAP-005 |
| BC-2.05.009 | BC-2.05.009.md | Progress tracking and cancellation | P0 | CAP-005 |
| BC-2.05.010 | BC-2.05.010.md | Tool error vs protocol error distinction | P0 | CAP-005 |

## S=3: TUI Dashboard

| BC ID | File | Title | Priority | CAP |
|-------|------|-------|----------|-----|
| BC-3.06.001 | BC-3.06.001.md | Multi-pane adaptive layout | P0 | CAP-006 |
| BC-3.06.002 | BC-3.06.002.md | Color system auto-detection and degradation | P0 | CAP-006 |
| BC-3.07.001 | BC-3.07.001.md | Vi-style keyboard navigation across panes | P0 | CAP-007 |
| BC-3.07.002 | BC-3.07.002.md | Search and command mode | P0 | CAP-007 |
| BC-3.07.003 | BC-3.07.003.md | Mouse supplementary input | P0 | CAP-007 |
| BC-3.08.001 | BC-3.08.001.md | JSON-RPC syntax-highlighted message rendering | P0 | CAP-008 |
| BC-3.08.002 | BC-3.08.002.md | Sparkline and histogram health metric visualization | P0 | CAP-008 |
| BC-3.08.003 | BC-3.08.003.md | Server browser with status badges | P0 | CAP-008 |
| BC-3.08.004 | BC-3.08.004.md | Capability explorer | P0 | CAP-008 |
| BC-3.08.005 | BC-3.08.005.md | Accessibility — no color-only indicators | P0 | CAP-008 |

## S=4: Traffic Inspection

| BC ID | File | Title | Priority | CAP |
|-------|------|-------|----------|-----|
| BC-4.09.001 | BC-4.09.001.md | Transparent JSON-RPC message capture | P0 | CAP-009 |
| BC-4.09.002 | BC-4.09.002.md | Per-message timing and throughput analysis | P0 | CAP-009 |
| BC-4.09.003 | BC-4.09.003.md | Capture buffer management with bounded memory | P0 | CAP-009 |
| BC-4.10.001 | BC-4.10.001.md | Traffic filtering | P0 | CAP-010 |
| BC-4.10.002 | BC-4.10.002.md | Full-text payload search | P0 | CAP-010 |
| BC-4.10.003 | BC-4.10.003.md | Message sequence replay | P0 | CAP-010 |

## S=5: CLI Mode

| BC ID | File | Title | Priority | CAP |
|-------|------|-------|----------|-----|
| BC-5.11.001 | BC-5.11.001.md | Subcommand dispatch | P0 | CAP-011 |
| BC-5.11.002 | BC-5.11.002.md | Structured JSON output on stdout | P0 | CAP-011 |
| BC-5.11.003 | BC-5.11.003.md | Exit code semantics compliance | P0 | CAP-011 |
| BC-5.12.001 | BC-5.12.001.md | Agent-optimized minimal token output | P0 | CAP-012 |
| BC-5.12.002 | BC-5.12.002.md | Pipeable output for shell composition | P0 | CAP-012 |

## S=6: Health Monitoring

| BC ID | File | Title | Priority | CAP |
|-------|------|-------|----------|-----|
| BC-6.13.001 | BC-6.13.001.md | Passive latency and throughput metric collection | P0 | CAP-013 |
| BC-6.13.002 | BC-6.13.002.md | Error rate trend tracking | P0 | CAP-013 |
| BC-6.14.001 | BC-6.14.001.md | Configurable alerting thresholds | P0 | CAP-014 |
| BC-6.14.002 | BC-6.14.002.md | Alert state machine | P0 | CAP-014 |
| BC-6.15.001 | BC-6.15.001.md | Time-series metric visualization in TUI | P0 | CAP-015 |
| BC-6.15.002 | BC-6.15.002.md | Metric snapshot JSON export via CLI | P0 | CAP-015 |

## S=7: Security Auditing

| BC ID | File | Title | Priority | CAP |
|-------|------|-------|----------|-----|
| BC-7.16.001 | BC-7.16.001.md | Dangerous tool pattern detection | P1 | CAP-016 |
| BC-7.16.002 | BC-7.16.002.md | SSRF attempt detection | P1 | CAP-016 |
| BC-7.16.003 | BC-7.16.003.md | Schema drift / rug pull detection | P1 | CAP-016 |
| BC-7.16.004 | BC-7.16.004.md | Finding severity classification with confidence scores | P1 | CAP-016 |
| BC-7.17.001 | BC-7.17.001.md | Permission escalation detection | P1 | CAP-017 |
| BC-7.17.002 | BC-7.17.002.md | Root enforcement compliance verification | P1 | CAP-017 |
| BC-7.17.003 | BC-7.17.003.md | Authentication handling validation | P1 | CAP-017 |
| BC-7.18.001 | BC-7.18.001.md | Structured security audit report generation | P1 | CAP-018 |
| BC-7.18.002 | BC-7.18.002.md | User-defined finding suppression rules | P1 | CAP-018 |
| BC-7.18.003 | BC-7.18.003.md | OWASP AST10 coverage mapping | P1 | CAP-018 |

## S=8: Conformance Testing

| BC ID | File | Title | Priority | CAP |
|-------|------|-------|----------|-----|
| BC-8.19.001 | BC-8.19.001.md | Capability negotiation conformance validation | P1 | CAP-019 |
| BC-8.19.002 | BC-8.19.002.md | Method coverage and error handling conformance | P1 | CAP-019 |
| BC-8.19.003 | BC-8.19.003.md | Transport compliance validation | P1 | CAP-019 |
| BC-8.20.001 | BC-8.20.001.md | JUnit XML output generation | P1 | CAP-020 |
| BC-8.20.002 | BC-8.20.002.md | JSON conformance report output | P1 | CAP-020 |

## S=9: Config Drift Detection

| BC ID | File | Title | Priority | CAP |
|-------|------|-------|----------|-----|
| BC-9.21.001 | BC-9.21.001.md | Cross-editor config comparison | P1 | CAP-021 |
| BC-9.21.002 | BC-9.21.002.md | Drift report with actionable detail | P1 | CAP-021 |
| BC-9.22.001 | BC-9.22.001.md | Reconciliation workflow guidance | P2 | CAP-022 |

## S=10: Server Comparison

| BC ID | File | Title | Priority | CAP |
|-------|------|-------|----------|-----|
| BC-10.23.001 | BC-10.23.001.md | Tool schema diff between two servers | P2 | CAP-023 |
| BC-10.24.001 | BC-10.24.001.md | Capability set delta comparison | P2 | CAP-024 |
| BC-10.25.001 | BC-10.25.001.md | Response behavior delta testing | P2 | CAP-025 |

## Priority Distribution

| Priority | Count | Subsystems |
|----------|-------|-----------|
| P0 | 49 | S1 (9), S2 (13), S3 (10), S4 (6), S5 (5), S6 (6) |
| P1 | 13 | S7 (10), S8 (5), S9 (2) — note S7 security is P1 per brief |
| P2 | 4 | S9 (1), S10 (3) |
