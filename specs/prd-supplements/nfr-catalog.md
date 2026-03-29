---
document_type: prd-supplement-nfr-catalog
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-03-29T11:25:00
phase: 1a
inputs: [prd.md, domain-spec/L2-INDEX.md]
input-hash: ""
traces_to: prd.md
---

# Non-Functional Requirements Catalog: Forge MCP

> PRD supplement — extracted from PRD Section 4.
> Referenced by: architect, performance-engineer, formal-verifier.

## NFR Registry

| ID | Category | Requirement | Target | Validation Method | Priority | Risk Source |
|----|----------|-------------|--------|------------------|----------|-------------|
| NFR-001 | Performance | CLI cold start to first output | < 50ms | benchmark: `hyperfine` on all 5 targets | P0 | N/A (brief success criterion) |
| NFR-002 | Performance | TUI sustained frame rate at 100 events/sec ingest | ≥ 60fps with < 5MB RSS delta | benchmark: synthetic event stream, measure fps + RSS | P0 | R-005 |
| NFR-003 | Performance | Agent context efficiency for discover→inspect→call | < 500 tokens (GPT-4 tokenizer) | benchmark: measure token count of JSON output for standard workflow | P0 | N/A (brief success criterion) |
| NFR-004 | Performance | Metric collection latency impact on server | < 1% additional latency | benchmark: compare server latency with/without Forge MCP monitoring | P0 | DI-008 |
| NFR-005 | Security | Security heuristic true-positive rate | > 80% precision | test: run against corpus of 50+ known-good and 50+ known-dangerous behaviors | P1 | R-004 |
| NFR-006 | Security | OWASP AST10 runtime-applicable category coverage | ≥ 83% (≥ 5 of 6 runtime categories) | audit: map implemented rules to AST10 categories | P1 | R-011 |
| NFR-007 | Security | Schema metadata integrity verification | Compare hashes on every connection | test: verify hash comparison triggers on tool description changes | P1 | R-014 |
| NFR-008 | Reliability | Cross-platform binary distribution | 5 static binaries: linux-x64, linux-arm64, macos-x64, macos-arm64, windows-x64 | build: CI builds + integration tests on all targets | P0 | N/A (brief success criterion) |
| NFR-009 | Reliability | Binary size with LTO | < 25MB stripped | build: measure binary size in CI | P0 | ASM-010 |
| NFR-010 | Reliability | Protocol conformance spec coverage | ≥ 90% of MCP 2025-11-25 methods exercised | test: count exercised methods vs. total method list (~25) | P1 | N/A (brief success criterion) |
| NFR-011 | Reliability | Spec version abstraction | Support ≥ 2 spec versions simultaneously | integration test: connect to 2024-11-05 and 2025-11-25 servers | P1 | R-002 |
| NFR-012 | Scalability | Traffic capture bounded memory | Default < 100MB, configurable | load test: sustained 1000 msg/sec for 30 min, measure RSS | P0 | R-010 |
| NFR-013 | Scalability | Pagination iteration cap | Default 100 pages, configurable | unit test: verify loop detection and termination | P0 | DI-019, ASM-014 |
| NFR-014 | Maintainability | rmcp interface boundary | Thin abstraction over rmcp for protocol operations | code review: verify rmcp usage confined to adapter layer | P0 | R-003 |
| NFR-015 | Accessibility | No color-only indicators in TUI | All status conveyed by shape/text + color | manual audit: verify every status badge/indicator | P0 | BC-3.08.005 |

## NFR Categories

| Category | Description | Validation Agent |
|----------|-------------|-----------------|
| Performance | Throughput, latency, memory, startup time | performance-engineer |
| Security | Heuristic precision, coverage, integrity | security-reviewer |
| Reliability | Cross-platform, binary, spec coverage | formal-verifier, devops-engineer |
| Scalability | Buffer limits, pagination, high-volume handling | performance-engineer |
| Maintainability | Abstraction boundaries, coupling | code-reviewer |
| Accessibility | Color-independent indicators, keyboard-only operation | accessibility-auditor |

## NFR-to-Module Mapping

| NFR ID | Affected Modules | Architectural Impact |
|--------|-----------------|---------------------|
| NFR-001 | CLI entry, clap parsing, daemon connection | Must minimize initialization path; lazy-load heavy subsystems |
| NFR-002 | TUI renderer, event loop, ratatui | Frame throttling, virtual scrolling, differential rendering |
| NFR-003 | CLI output formatters | Minimal JSON schema, no verbose wrappers |
| NFR-004 | Metric collector, traffic capture | Passive observation only; no synthetic probes |
| NFR-005 | Security rule engine | Rule corpus quality; confidence scoring calibration |
| NFR-006 | Security rule engine, report generator | Must map rules to AST10 categories at rule definition time |
| NFR-007 | Security auditor, connection manager | Hash tool metadata at first connect; compare on reconnect |
| NFR-008 | Build system, CI/CD | Cross-compilation targets; platform-specific integration tests |
| NFR-009 | Build system | LTO + strip in release profile; dependency size audit |
| NFR-010 | Conformance test suite | Method inventory tracking vs. spec |
| NFR-011 | Protocol adapter, capability negotiation | Version-tagged behavior; conditional method support |
| NFR-012 | Traffic capture buffer | Ring buffer or FIFO eviction; configurable limits |
| NFR-013 | Protocol adapter, list methods | Cursor dedup set; configurable page cap |
| NFR-014 | Protocol adapter layer | Thin trait boundary; rmcp types confined to adapter |
| NFR-015 | TUI widgets, style system | Every styled element must include non-color indicator |
