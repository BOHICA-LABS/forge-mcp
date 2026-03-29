---
document_type: prd
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-03-29T11:25:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md, planning/domain-research.md, planning/market-intel.md]
input-hash: ""
traces_to: domain-spec/L2-INDEX.md
supplements: [interface-definitions.md, error-taxonomy.md, nfr-catalog.md, glossary.md]
---

# Product Requirements Document: Forge MCP

> **Context Engineering Principle — Extended ToC Pattern:**
> Each section provides a concise summary with references to full detail.
> Critical constraints are frontloaded. Validation rules at end.
> Signal density over coverage.

> **BC Index Model:** Each Behavioral Contract (BC) lives in its own file under
> `behavioral-contracts/`. Tables in Section 2 provide one-line summaries linking
> to individual BC files. Full contract details are NOT inlined here.

> **PRD Supplement Model:** Sections 3–5 are extracted to `prd-supplements/`.
> See `supplements` frontmatter for file listing.

## 1. Product Overview

### 1.1 Problem Statement

AI platform teams deploying MCP servers face an opaque operational gap: the distance
between "server is running" and "server is correct, healthy, and safe" is filled by
guesswork, raw JSON-RPC logs, and 6+ fragmented single-purpose tools that each cover
only 1–2 workflow dimensions. No tool performs runtime security analysis of live MCP
traffic. 36.7% of 7,000+ analyzed MCP servers are vulnerable to SSRF (BlueRock
research). Supply chain attacks on MCP packages are actively occurring. The MCP
ecosystem has 10,000–16,000 active servers and 97M monthly SDK downloads but the
tooling layer remains radically fragmented.

### 1.2 Solution Vision

Forge MCP is a single Rust binary that unifies six workflow dimensions — server
discovery, protocol inspection, health monitoring, runtime security auditing,
conformance testing, and config drift detection — into a scriptable CLI and
interactive TUI dashboard. Built on the official rmcp SDK for inherited spec
compliance, it targets < 50ms cold start and < 500 token overhead for AI agent
consumption. The TUI provides "Wireshark for MCP" traffic inspection with real-time
health metrics. The security auditor performs runtime behavioral analysis that static
scanners cannot — detecting SSRF, permission escalation, tool poisoning, and rug
pull attacks on live traffic.

### 1.3 Key Differentiators

| ID | Differentiator | Description |
|----|---------------|-------------|
| KD-001 | Runtime Security Analysis | Only tool performing runtime behavioral analysis of live MCP traffic. Static scanners miss what servers actually do. Maps to OWASP AST10. |
| KD-002 | Unified 6-Dimension Tool | Only tool covering discovery + inspection + monitoring + security + conformance + config management. Competitors cover ≤ 3 dimensions. |
| KD-003 | Official SDK Compliance | Only Rust MCP tool built on rmcp SDK. Existing Rust tools roll custom protocol code, carrying spec drift risk. |
| KD-004 | Agent-Optimized CLI | < 50ms cold start, < 500 token overhead for discover→inspect→call. Rust binary vs. ~300ms Node.js alternatives. |
| KD-005 | CI/CD Conformance Testing | First automated MCP conformance suite with JUnit XML + JSON output. Official inspector is interactive-only. |
| KD-006 | Zero-Dependency Static Binary | Single binary, no runtime dependencies. Runs on air-gapped systems, minimal containers, CI runners without setup. |

### 1.4 Target Users

| Persona | Description | Volume | Pain Level |
|---------|-------------|--------|------------|
| AI Platform Engineers | Deploy and operate MCP-connected AI products | 50K–150K | Blocker (opaque connections, no monitoring) |
| DevEx Engineers | Manage editor/IDE MCP configs across teams | 20K–50K | High (silent config drift) |
| AI Agent Developers | Build agents needing lightweight tool discovery | 50K–200K | Medium (token/latency overhead) |
| Security Teams | Evaluate MCP servers before production | 3K–8K | Blocker (no runtime audit tool exists) |
| MCP Server Authors | Validate spec compliance of implementations | 5K–15K | High (no automated conformance suite) |

### 1.5 Out of Scope

- **Not an MCP server** — client/inspector only, does not serve MCP capabilities
- **No embedded LLM** — sampling requests proxied to external LLM APIs only
- **No web UI or Electron** — terminal-native only (TUI + CLI)
- **No MCP registry or marketplace** — discovers locally configured servers only
- **No payment protocol (x402)** — out of scope for inspection/debugging mission
- **No static source-code scanning** — mcpsec/ramparts cover static analysis; Forge MCP is runtime-only
- **No MCP server scaffolding or codegen** — mcporter covers config-to-code generation
- **No editor config write-back** — configs are read-only (DI-015); reconciliation is advisory

## 2. Behavioral Contracts Index

> Individual BC files live in `behavioral-contracts/`.
> Grouped by subsystem aligned with L2 domain capabilities.
> Each BC uses hierarchical numbering: BC-S.SS.NNN.

### 2.1 Server Discovery & Connection Management (CAP-001, CAP-002, CAP-003)

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-1.01.001 | Config file discovery and path resolution | P0 |
| BC-1.01.002 | Dual-schema config parsing (mcpServers vs servers) | P0 |
| BC-1.01.003 | Config source aggregation with conflict attribution | P0 |
| BC-1.02.001 | Stdio transport connection establishment | P0 |
| BC-1.02.002 | Streamable HTTP transport connection establishment | P0 |
| BC-1.02.003 | Connection lifecycle management (keepalive, graceful shutdown) | P0 |
| BC-1.03.001 | Daemon lazy start and session pooling | P0 |
| BC-1.03.002 | Named session persistence across CLI invocations | P0 |
| BC-1.03.003 | Daemon socket conflict detection and recovery | P0 |

> Full contracts: `behavioral-contracts/BC-1.01.001.md` through `BC-1.03.003.md`

### 2.2 MCP Protocol Operations (CAP-004, CAP-005)

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-2.04.001 | Bidirectional capability negotiation | P0 |
| BC-2.04.002 | Client capability advertisement (sampling, elicitation, roots) | P0 |
| BC-2.04.003 | Graceful degradation with older spec versions | P0 |
| BC-2.05.001 | Tool list and invocation with pagination | P0 |
| BC-2.05.002 | Resource list, read, and subscription management | P0 |
| BC-2.05.003 | Prompt list and retrieval with pagination | P0 |
| BC-2.05.004 | Sampling proxy to external LLM | P0 |
| BC-2.05.005 | Elicitation request handling (form + URL modes) | P0 |
| BC-2.05.006 | Roots list response and change notification | P0 |
| BC-2.05.007 | Logging level control and message display | P0 |
| BC-2.05.008 | Completion/autocomplete requests | P0 |
| BC-2.05.009 | Progress tracking and cancellation | P0 |
| BC-2.05.010 | Tool error vs protocol error distinction | P0 |

> Full contracts: `behavioral-contracts/BC-2.04.001.md` through `BC-2.05.010.md`

### 2.3 TUI Dashboard (CAP-006, CAP-007, CAP-008)

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-3.06.001 | Multi-pane adaptive layout (80×24 to ultra-wide) | P0 |
| BC-3.06.002 | Color system auto-detection and degradation | P0 |
| BC-3.07.001 | Vi-style keyboard navigation across panes | P0 |
| BC-3.07.002 | Search and command mode (/ and :) | P0 |
| BC-3.07.003 | Mouse supplementary input | P0 |
| BC-3.08.001 | JSON-RPC syntax-highlighted message rendering | P0 |
| BC-3.08.002 | Sparkline and histogram health metric visualization | P0 |
| BC-3.08.003 | Server browser with status badges | P0 |
| BC-3.08.004 | Capability explorer (tools/resources/prompts tree) | P0 |
| BC-3.08.005 | Accessibility — no color-only indicators | P0 |

> Full contracts: `behavioral-contracts/BC-3.06.001.md` through `BC-3.08.005.md`

### 2.4 Traffic Inspection (CAP-009, CAP-010)

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-4.09.001 | Transparent JSON-RPC message capture | P0 |
| BC-4.09.002 | Per-message timing and throughput analysis | P0 |
| BC-4.09.003 | Capture buffer management with bounded memory | P0 |
| BC-4.10.001 | Traffic filtering by method, direction, time, content | P0 |
| BC-4.10.002 | Full-text payload search | P0 |
| BC-4.10.003 | Message sequence replay against target server | P0 |

> Full contracts: `behavioral-contracts/BC-4.09.001.md` through `BC-4.10.003.md`

### 2.5 CLI Mode (CAP-011, CAP-012)

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-5.11.001 | Subcommand dispatch (list, call, info, grep, test) | P0 |
| BC-5.11.002 | Structured JSON output on stdout | P0 |
| BC-5.11.003 | Exit code semantics compliance | P0 |
| BC-5.12.001 | Agent-optimized minimal token output | P0 |
| BC-5.12.002 | Pipeable output for shell composition | P0 |

> Full contracts: `behavioral-contracts/BC-5.11.001.md` through `BC-5.12.002.md`

### 2.6 Health Monitoring (CAP-013, CAP-014, CAP-015)

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-6.13.001 | Passive latency and throughput metric collection | P0 |
| BC-6.13.002 | Error rate trend tracking (protocol vs tool errors) | P0 |
| BC-6.14.001 | Configurable alerting thresholds | P0 |
| BC-6.14.002 | Alert state machine (normal → breached → recovered) | P0 |
| BC-6.15.001 | Time-series metric visualization in TUI | P0 |
| BC-6.15.002 | Metric snapshot JSON export via CLI | P0 |

> Full contracts: `behavioral-contracts/BC-6.13.001.md` through `BC-6.15.002.md`

### 2.7 Security Auditing (CAP-016, CAP-017, CAP-018)

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-7.16.001 | Dangerous tool pattern detection (filesystem, exec, network) | P1 |
| BC-7.16.002 | SSRF attempt detection (private IPs, metadata endpoints) | P1 |
| BC-7.16.003 | Schema drift / rug pull detection | P1 |
| BC-7.16.004 | Finding severity classification with confidence scores | P1 |
| BC-7.17.001 | Permission escalation detection | P1 |
| BC-7.17.002 | Root enforcement compliance verification | P1 |
| BC-7.17.003 | Authentication handling validation | P1 |
| BC-7.18.001 | Structured security audit report generation | P1 |
| BC-7.18.002 | User-defined finding suppression rules | P1 |
| BC-7.18.003 | OWASP AST10 coverage mapping | P1 |

> Full contracts: `behavioral-contracts/BC-7.16.001.md` through `BC-7.18.003.md`

### 2.8 Conformance Testing (CAP-019, CAP-020)

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-8.19.001 | Capability negotiation conformance validation | P1 |
| BC-8.19.002 | Method coverage and error handling conformance | P1 |
| BC-8.19.003 | Transport compliance validation | P1 |
| BC-8.20.001 | JUnit XML output generation | P1 |
| BC-8.20.002 | JSON conformance report output | P1 |

> Full contracts: `behavioral-contracts/BC-8.19.001.md` through `BC-8.20.002.md`

### 2.9 Config Drift Detection (CAP-021, CAP-022)

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-9.21.001 | Cross-editor config comparison | P1 |
| BC-9.21.002 | Drift report with actionable detail | P1 |
| BC-9.22.001 | Reconciliation workflow guidance | P2 |

> Full contracts: `behavioral-contracts/BC-9.21.001.md` through `BC-9.22.001.md`

### 2.10 Server Comparison (CAP-023, CAP-024, CAP-025)

| BC ID | Title | Priority |
|-------|-------|----------|
| BC-10.23.001 | Tool schema diff between two servers | P2 |
| BC-10.24.001 | Capability set delta comparison | P2 |
| BC-10.25.001 | Response behavior delta testing | P2 |

> Full contracts: `behavioral-contracts/BC-10.23.001.md` through `BC-10.25.001.md`

## 3. Interface Definition

> **Supplement:** Full interface definitions in `prd-supplements/interface-definitions.md`.
> Summary: 6 primary subcommands (list, call, info, grep, test, audit), TUI mode via
> `forge-mcp tui`, daemon control via `forge-mcp daemon`. JSON output on stdout,
> diagnostics on stderr. Exit codes: 0=success, 1=test failure, 2=connection error,
> 3=config error, 4=security finding. See supplement for full CLI help, JSON schemas,
> config file schema, and flag interaction rules.

## 4. Non-Functional Requirements

> **Supplement:** Full NFR catalog in `prd-supplements/nfr-catalog.md`.
> Summary: 15 NFRs covering performance (startup <50ms, TUI 60fps, <5MB RSS),
> security (precision >80%, AST10 ≥83%), reliability (5-platform binary <25MB),
> and scalability (100 events/sec, bounded capture <100MB). All NFRs have numerical
> targets and validation methods. See supplement for complete NFR registry.

## 5. Error Taxonomy

> **Supplement:** Full error taxonomy in `prd-supplements/error-taxonomy.md`.
> Summary: E-xxx-NNN error codes across 7 categories: CON (connection), CFG (config),
> PRO (protocol), TUI (terminal UI), SEC (security), CAP (capture), MON (monitoring).
> Three severity levels: broken (non-zero exit), degraded (zero exit with warning),
> cosmetic (display-only). See supplement for complete error catalog.

## 5b. Glossary

> **Supplement:** Domain terminology in `prd-supplements/glossary.md`.
> Defines MCP-specific terms, JSON-RPC concepts, and Forge MCP internal terminology
> for consistent use across all downstream artifacts.

## 6. Competitive Differentiator Traceability

> Maps each key differentiator from Section 1.3 to BCs that implement it.

### 6.1 KD-001 — Runtime Security Analysis

| BC ID | Contribution |
|-------|-------------|
| BC-7.16.001 | Dangerous tool pattern detection provides core behavioral analysis |
| BC-7.16.002 | SSRF detection addresses the 36.7% vulnerability rate finding |
| BC-7.16.003 | Schema drift detection catches rug pull attacks — unique capability |
| BC-7.16.004 | Confidence scoring enables precision-focused filtering |
| BC-7.17.001 | Permission escalation maps to AST03 Over-Privileged Skills |
| BC-7.17.002 | Root enforcement maps to AST06 Weak Isolation |
| BC-7.17.003 | Auth validation catches runtime auth bypass attempts |
| BC-7.18.001 | Structured reports enable enterprise compliance workflows |
| BC-7.18.003 | AST10 mapping gives industry-standard framework backing |
| BC-4.09.001 | Traffic capture provides the data feed for all security analysis |

### 6.2 KD-002 — Unified 6-Dimension Tool

| BC ID | Contribution |
|-------|-------------|
| BC-1.01.001–BC-1.03.003 | Dimension 1: Discovery and connection management |
| BC-2.04.001–BC-2.05.010 | Protocol operations enabling all inspection workflows |
| BC-4.09.001–BC-4.10.003 | Dimension 2: Traffic inspection |
| BC-6.13.001–BC-6.15.002 | Dimension 3: Health monitoring |
| BC-7.16.001–BC-7.18.003 | Dimension 4: Security auditing |
| BC-8.19.001–BC-8.20.002 | Dimension 5: Conformance testing |
| BC-9.21.001–BC-9.22.001 | Dimension 6: Config drift detection |

### 6.3 KD-003 — Official SDK Compliance (rmcp)

| BC ID | Contribution |
|-------|-------------|
| BC-1.02.001 | Stdio transport via rmcp — no custom framing |
| BC-1.02.002 | Streamable HTTP via rmcp — session management delegated |
| BC-2.04.001 | Capability negotiation via rmcp ClientCapabilitiesBuilder |
| BC-2.05.001–BC-2.05.010 | All protocol methods invoked through rmcp Peer API |

### 6.4 KD-004 — Agent-Optimized CLI

| BC ID | Contribution |
|-------|-------------|
| BC-5.11.001 | Scriptable subcommands for automation |
| BC-5.11.002 | JSON stdout enables programmatic consumption |
| BC-5.12.001 | < 500 token output directly supports agent workflows |
| BC-5.12.002 | Pipeable design enables Unix-style composition |

### 6.5 KD-005 — CI/CD Conformance Testing

| BC ID | Contribution |
|-------|-------------|
| BC-8.19.001 | Capability negotiation validation catches spec violations |
| BC-8.19.002 | Method coverage ensures comprehensive testing |
| BC-8.20.001 | JUnit XML for GitHub Actions/GitLab CI/Jenkins |
| BC-8.20.002 | JSON output for programmatic consumption |

### 6.6 KD-006 — Zero-Dependency Static Binary

| BC ID | Contribution |
|-------|-------------|
| All BCs | Architectural property — single binary delivers all capabilities without runtime dependencies |

## 7. Requirements Traceability Matrix

| BC ID | Source (L2 CAP) | Module(s) | Priority | Test Type |
|-------|----------------|-----------|----------|-----------|
| BC-1.01.001 | CAP-001 | [architect] | P0 | unit/integration |
| BC-1.01.002 | CAP-001 | [architect] | P0 | unit/property |
| BC-1.01.003 | CAP-001 | [architect] | P0 | integration |
| BC-1.02.001 | CAP-002 | [architect] | P0 | integration |
| BC-1.02.002 | CAP-002 | [architect] | P0 | integration |
| BC-1.02.003 | CAP-002 | [architect] | P0 | integration |
| BC-1.03.001 | CAP-003 | [architect] | P0 | integration |
| BC-1.03.002 | CAP-003 | [architect] | P0 | integration |
| BC-1.03.003 | CAP-003 | [architect] | P0 | integration |
| BC-2.04.001 | CAP-004 | [architect] | P0 | integration/property |
| BC-2.04.002 | CAP-004 | [architect] | P0 | integration |
| BC-2.04.003 | CAP-004 | [architect] | P0 | integration |
| BC-2.05.001 | CAP-005 | [architect] | P0 | integration/property |
| BC-2.05.002 | CAP-005 | [architect] | P0 | integration |
| BC-2.05.003 | CAP-005 | [architect] | P0 | integration |
| BC-2.05.004 | CAP-005 | [architect] | P0 | integration |
| BC-2.05.005 | CAP-005 | [architect] | P0 | integration |
| BC-2.05.006 | CAP-005 | [architect] | P0 | unit |
| BC-2.05.007 | CAP-005 | [architect] | P0 | integration |
| BC-2.05.008 | CAP-005 | [architect] | P0 | integration |
| BC-2.05.009 | CAP-005 | [architect] | P0 | integration |
| BC-2.05.010 | CAP-005 | [architect] | P0 | unit/property |
| BC-3.06.001 | CAP-006 | [architect] | P0 | integration/manual |
| BC-3.06.002 | CAP-006 | [architect] | P0 | unit |
| BC-3.07.001 | CAP-007 | [architect] | P0 | integration |
| BC-3.07.002 | CAP-007 | [architect] | P0 | integration |
| BC-3.07.003 | CAP-007 | [architect] | P0 | integration |
| BC-3.08.001 | CAP-008 | [architect] | P0 | unit/integration |
| BC-3.08.002 | CAP-008 | [architect] | P0 | unit |
| BC-3.08.003 | CAP-008 | [architect] | P0 | integration |
| BC-3.08.004 | CAP-008 | [architect] | P0 | integration |
| BC-3.08.005 | CAP-008 | [architect] | P0 | manual/audit |
| BC-4.09.001 | CAP-009 | [architect] | P0 | integration/property |
| BC-4.09.002 | CAP-009 | [architect] | P0 | unit |
| BC-4.09.003 | CAP-009 | [architect] | P0 | unit/property |
| BC-4.10.001 | CAP-010 | [architect] | P0 | unit/integration |
| BC-4.10.002 | CAP-010 | [architect] | P0 | unit |
| BC-4.10.003 | CAP-010 | [architect] | P0 | integration |
| BC-5.11.001 | CAP-011 | [architect] | P0 | integration |
| BC-5.11.002 | CAP-011 | [architect] | P0 | unit/property |
| BC-5.11.003 | CAP-011 | [architect] | P0 | unit |
| BC-5.12.001 | CAP-012 | [architect] | P0 | unit/integration |
| BC-5.12.002 | CAP-012 | [architect] | P0 | integration |
| BC-6.13.001 | CAP-013 | [architect] | P0 | unit/integration |
| BC-6.13.002 | CAP-013 | [architect] | P0 | unit |
| BC-6.14.001 | CAP-014 | [architect] | P0 | unit |
| BC-6.14.002 | CAP-014 | [architect] | P0 | unit/property |
| BC-6.15.001 | CAP-015 | [architect] | P0 | integration/manual |
| BC-6.15.002 | CAP-015 | [architect] | P0 | unit |
| BC-7.16.001 | CAP-016 | [architect] | P1 | unit/integration |
| BC-7.16.002 | CAP-016 | [architect] | P1 | unit/property |
| BC-7.16.003 | CAP-016 | [architect] | P1 | integration |
| BC-7.16.004 | CAP-016 | [architect] | P1 | unit |
| BC-7.17.001 | CAP-017 | [architect] | P1 | integration |
| BC-7.17.002 | CAP-017 | [architect] | P1 | integration |
| BC-7.17.003 | CAP-017 | [architect] | P1 | integration |
| BC-7.18.001 | CAP-018 | [architect] | P1 | unit/integration |
| BC-7.18.002 | CAP-018 | [architect] | P1 | unit |
| BC-7.18.003 | CAP-018 | [architect] | P1 | unit |
| BC-8.19.001 | CAP-019 | [architect] | P1 | integration |
| BC-8.19.002 | CAP-019 | [architect] | P1 | integration |
| BC-8.19.003 | CAP-019 | [architect] | P1 | integration |
| BC-8.20.001 | CAP-020 | [architect] | P1 | unit |
| BC-8.20.002 | CAP-020 | [architect] | P1 | unit |
| BC-9.21.001 | CAP-021 | [architect] | P1 | unit/integration |
| BC-9.21.002 | CAP-021 | [architect] | P1 | unit |
| BC-9.22.001 | CAP-022 | [architect] | P2 | integration |
| BC-10.23.001 | CAP-023 | [architect] | P2 | unit/integration |
| BC-10.24.001 | CAP-024 | [architect] | P2 | unit |
| BC-10.25.001 | CAP-025 | [architect] | P2 | integration |
