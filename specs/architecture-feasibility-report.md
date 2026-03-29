---
document_type: architecture-feasibility-report
level: ops
version: "1.0"
status: approved
producer: architect
timestamp: 2026-03-29T14:05:00
phase: 1b
inputs: [specs/prd.md]
input-hash: ""
traces_to: specs/prd.md
prd_version: "1.0"
---

# Architecture Feasibility Report: Forge MCP

## Executive Summary

The PRD is feasible as-is. All 65 BCs are technically achievable with Rust + rmcp + ratatui. The 10-subsystem decomposition maps cleanly to implementation modules. No restructuring needed.

## Key Feasibility Notes

- All 65 BCs are feasible with Rust + rmcp + ratatui
- The 10-subsystem decomposition from L2 aligns well with module boundaries
- rmcp handles transport layer (DI-004), reducing implementation risk
- Server-initiated methods (sampling proxy, elicitation) add complexity but are well-defined in rmcp's ClientHandler trait
- The dual config schema (mcpServers vs servers) is straightforward JSON parsing
- NFR-001 (<50ms cold start) is achievable with Rust's zero-cost abstractions and lazy subsystem loading
- NFR-002 (60fps TUI at 100 events/sec) needs frame throttling and virtual scrolling but is feasible with ratatui
- NFR-012 (bounded capture <100MB) is a ring buffer implementation — straightforward
- No BCs are infeasible or need revision
- Missing concern: The PRD doesn't explicitly address graceful TUI cleanup on panic — needs architecture-level panic handler

## Constraint Mapping

| PRD Section | BC-S.SS.NNN | Requirement | Architecture Constraint | Feasibility | Resolution |
|-------------|-------------|-------------|------------------------|-------------|-----------|
| 2.1 | BC-1.01.002 | Dual-schema config parsing | Two JSON schemas (mcpServers, servers) | feasible | Union type with schema detection |
| 2.2 | BC-2.05.004 | Sampling proxy to external LLM | Requires HTTP client + LLM API integration | feasible | reqwest + configurable LLM endpoint |
| 2.2 | BC-2.05.005 | Elicitation request handling | TUI interaction flow for forms | feasible | Modal dialog widget in ratatui |
| 2.3 | BC-3.06.001 | Multi-pane adaptive layout 80×24 to ultra-wide | ratatui Layout constraints | feasible | Responsive layout with breakpoints |
| 2.4 | BC-4.09.001 | Transparent message capture | Must not modify wire content (DI-005) | feasible | Observer pattern on transport events |
| 2.7 | BC-7.16.002 | SSRF detection | Pattern matching on IP addresses in payloads | feasible | Deterministic rules + heuristic layer |
| 2.8 | BC-8.19.001 | Conformance validation | Need reference behavior per spec | feasible | Spec-derived assertion library |
| 2.10 | BC-10.25.001 | Response behavior delta | Parallel requests to 2 servers | feasible | Async concurrent connections |

## Subsystem Grouping Assessment

All 10 subsystems valid. All NFR profiles coherent. No subsystem has conflicting NFR assignments. The L2 domain decomposition maps 1:1 to implementation modules with clean boundaries.

## Subsystem-to-Module Preliminary Mapping

| L2 Subsystem | Proposed Modules | Pure/Effectful |
|-------------|-----------------|---------------|
| S1: Discovery & Connection | forge_discovery (pure parsing), forge_daemon (effectful transport) | mixed |
| S2: Protocol Operations | forge_core — pure message types + effectful rmcp wrapper | mixed |
| S3: TUI Dashboard | forge_tui — effectful rendering + pure state machine | mixed |
| S4: Traffic Inspection | forge_traffic — pure filtering/search + effectful I/O | mixed |
| S5: CLI Mode | forge_mcp (binary) — effectful I/O + pure formatting | mixed |
| S6: Health Monitoring | forge_health — pure calculations + pure state machine | pure-heavy |
| S7: Security Auditing | forge_security — pure analysis + pure formatting | pure |
| S8: Conformance Testing | forge_conformance — pure assertions + effectful test runner | mixed |
| S9: Config Drift | forge_config — pure comparison | pure |
| S10: Server Comparison | forge_config — pure diffing (shared crate with S9) | pure |

## Decision Log

| Decision | Alternatives Considered | Chosen | Rationale |
|----------|------------------------|--------|-----------|
| Single Cargo workspace with library crates | Single binary monolith, multiple binaries | Workspace with lib crates | Enables unit testing of pure cores independently, single binary output via re-exports |
| rmcp as sole transport layer | Custom transport, dual rmcp+custom | rmcp only (DI-004) | Hard constraint from brief; custom transport is the #1 risk in existing tools |
| Crossterm backend for ratatui | Termion, Termwiz | Crossterm | Best cross-platform support including Windows; default for ratatui |

## Approval

**Architect approves.** The PRD subsystem grouping is feasible. Proceed to L3 architecture design.
