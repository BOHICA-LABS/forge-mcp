---
document_type: domain-spec-index
level: L2
version: "1.0"
status: draft
producer: business-analyst
timestamp: 2026-03-29T10:55:00
phase: 1a
inputs: [product-brief.md, market-intel.md]
input-hash: ""
traces_to: .factory/planning/product-brief.md
sections:
  - capabilities.md
  - entities.md
  - invariants.md
  - events.md
  - edge-cases.md
  - assumptions.md
  - risks.md
  - failure-modes.md
  - differentiators.md
---

# L2 Domain Specification: Forge MCP

> **Sharded artifact (DF-021).** This index provides navigation and summary.
> Detail lives in per-section files listed below. Each section targets
> 800–1,200 tokens for optimal LLM consumption.

> **Note:** Domain research (`.factory/planning/domain-research.md`) was not yet
> available when this spec was produced. Primary inputs were the approved product
> brief (v1.1) and market intelligence assessment. Domain research findings should
> be reconciled when available.

## Domain Summary

Forge MCP is a Rust-based command-line tool that provides a unified binary for
discovering, inspecting, debugging, monitoring, and security-auditing MCP (Model
Context Protocol) servers. It combines a scriptable CLI, an interactive TUI
dashboard, and a protocol conformance suite — targeting AI platform engineers,
DevEx engineers, agent developers, security teams, and MCP server authors.

## Document Map

| Section | File | Est. Tokens | Primary Consumer | Purpose |
|---------|------|-------------|-----------------|---------|
| Domain Capabilities | capabilities.md | ~1100 | product-owner, architect, story-writer | CAP-NNN capability catalog (25 atomic capabilities) |
| Domain Entities | entities.md | ~1000 | architect, product-owner, ux-designer | Entity model with attributes and relationships |
| Domain Invariants | invariants.md | ~1000 | product-owner, architect | DI-NNN business rules (15 invariants) |
| Domain Events | events.md | ~950 | architect | Event triggers, preconditions, outcomes |
| Edge Cases | edge-cases.md | ~1000 | story-writer, test-writer | DEC-NNN domain-level edge cases (15 cases) |
| Assumptions | assumptions.md | ~1000 | product-owner, test-writer | ASM-NNN with validation methods (12 assumptions) |
| Risks | risks.md | ~1100 | product-owner, architect | R-NNN risk register (12 risks) |
| Failure Modes | failure-modes.md | ~1000 | architect, test-writer | FM-NNN runtime failure catalog (15 modes) |
| Differentiators | differentiators.md | ~900 | product-owner | Competitive differentiator → CAP-NNN mapping |

## Cross-References

| If you need... | Read these together |
|----------------|-------------------|
| BC creation input | capabilities.md + invariants.md + edge-cases.md + assumptions.md + risks.md + differentiators.md |
| Architecture design input | capabilities.md + entities.md + invariants.md + events.md + risks.md + failure-modes.md |
| Story decomposition input | capabilities.md + edge-cases.md |
| Holdout scenario generation | assumptions.md + risks.md + failure-modes.md |
| NFR derivation | risks.md + failure-modes.md |
| Full domain review (adversary/spec-reviewer) | ALL sections |

## ID Registry Summary

| ID Format | Count | Section |
|-----------|-------|---------|
| CAP-NNN | 25 | capabilities.md |
| DI-NNN | 15 | invariants.md |
| DEC-NNN | 15 | edge-cases.md |
| ASM-NNN | 12 | assumptions.md |
| R-NNN | 12 | risks.md |
| FM-NNN | 15 | failure-modes.md |

## Priority Distribution

| Priority | Count | Items |
|----------|-------|-------|
| P0 (must-have) | 15 | CAP-001 through CAP-005, CAP-006 through CAP-010, CAP-011 through CAP-015 |
| P1 (should-have) | 6 | CAP-016 through CAP-021 |
| P2 (nice-to-have) | 4 | CAP-022 through CAP-025 |

## Brief → Capability Traceability

| Brief Capability # | Brief Description | Domain CAPs |
|---|---|---|
| 1 | Server discovery and connection management | CAP-001, CAP-002, CAP-003 |
| 2 | Full MCP 2025-11-25 protocol coverage | CAP-004, CAP-005 |
| 3 | Interactive TUI dashboard | CAP-006, CAP-007, CAP-008 |
| 4 | Protocol-level traffic inspection | CAP-009, CAP-010 |
| 5 | Non-interactive CLI mode | CAP-011, CAP-012 |
| 6 | Server health monitoring | CAP-013, CAP-014, CAP-015 |
| 7 | Runtime security auditing | CAP-016, CAP-017, CAP-018 |
| 8 | Protocol conformance testing | CAP-019, CAP-020 |
| 9 | Config sync and drift detection | CAP-021, CAP-022 |
| 10 | Server comparison and diff | CAP-023, CAP-024, CAP-025 |
