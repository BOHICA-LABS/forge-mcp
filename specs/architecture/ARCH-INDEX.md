---
document_type: architecture-index
level: L3
version: "1.0"
status: draft
producer: architect
timestamp: 2026-03-29T14:05:00
phase: 1b
inputs: [domain-spec/L2-INDEX.md, specs/prd.md]
traces_to: specs/prd.md
deployment_topology: single-service
---

# Architecture Index: Forge MCP

This is a single-service CLI/TUI tool. Single Rust binary. Single repo. `deployment_topology: single-service`.

## Document Map

| Section | File | Tokens | Primary Consumer | Purpose |
|---------|------|--------|-----------------|---------|
| System Overview | system-overview.md | ~900 | orchestrator, all agents | High-level architecture, ASCII diagram, deployment |
| Module Decomposition | module-decomposition.md | ~1000 | story-writer, implementer | Module catalog with responsibilities and boundaries |
| Dependency Graph | dependency-graph.md | ~700 | story-writer, consistency-validator | Module dependency DAG with direction rules |
| API Surface | api-surface.md | ~1000 | test-writer, implementer | Key traits, types, inter-module contracts |
| Purity Boundary Map | purity-boundary-map.md | ~700 | implementer, formal-verifier | Pure core vs effectful shell classification |
| Verification Architecture | verification-architecture.md | ~1000 | formal-verifier, architect | Provable properties catalog, proof strategies |
| Tooling Selection | tooling-selection.md | ~600 | formal-verifier, devops-engineer | Verification toolchain per module |
| Test Vectors | test-vectors.md | ~800 | test-writer, implementer | Sample inputs/outputs per subsystem |
| Verification Coverage | verification-coverage-matrix.md | ~600 | consistency-validator | VP-to-module mapping |

## Cross-References

| If you need... | Read these together |
|----------------|-------------------|
| Implementation plan for a module | module-decomposition.md + dependency-graph.md + api-surface.md |
| Verification plan for a module | verification-architecture.md + purity-boundary-map.md + tooling-selection.md |
| Full module picture | module-decomposition.md + purity-boundary-map.md + verification-coverage-matrix.md |
| Story decomposition input | module-decomposition.md + dependency-graph.md |
| Security review scope | module-criticality.md + purity-boundary-map.md |

## Architecture Decisions

| ID | Decision | Rationale |
|----|----------|-----------|
| AD-001 | Single Cargo workspace, one output binary | CLI tool with tight coupling between subsystems; single binary is a key differentiator (KD-006) |
| AD-002 | rmcp as exclusive transport/protocol layer | DI-004 hard constraint; custom protocol code is the #1 risk |
| AD-003 | Pure core / effectful shell separation per module | Enables formal verification of business logic (Kani/proptest) without I/O mocking |
| AD-004 | Event-driven internal architecture | MessageCaptured events flow to metrics, security, TUI consumers via channels |
| AD-005 | Tokio async runtime | Required by rmcp; also powers TUI event loop and daemon |
| AD-006 | Ring buffer for traffic capture | Bounded memory (NFR-012); O(1) append, FIFO eviction |
| AD-007 | Crossterm backend for ratatui | Best Windows support; ratatui default |
