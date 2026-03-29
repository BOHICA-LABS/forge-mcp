---
document_type: domain-spec-index
level: L2
version: "1.1"
status: draft
producer: business-analyst
timestamp: 2026-03-29T11:05:00
phase: 1a
inputs: [product-brief.md, market-intel.md, domain-research]
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
  - reconciliation-notes.md
---

# L2 Domain Specification: Forge MCP

> **Sharded artifact (DF-021).** This index provides navigation and summary.
> Detail lives in per-section files listed below. Each section targets
> 800–1,200 tokens for optimal LLM consumption.

> **v1.1 — Post-domain-research reconciliation.** This version incorporates
> findings from MCP protocol research (spec details, rmcp SDK API surface,
> OWASP AST10 categories, MCP security research including BlueRock/Equixly
> CVE evidence, and competitive tool analysis). See `reconciliation-notes.md`
> for a summary of all changes.

## Domain Summary

Forge MCP is a Rust-based command-line tool that provides a unified binary for
discovering, inspecting, debugging, monitoring, and security-auditing MCP (Model
Context Protocol) servers. It combines a scriptable CLI, an interactive TUI
dashboard, and a protocol conformance suite — targeting AI platform engineers,
DevEx engineers, agent developers, security teams, and MCP server authors.

## Document Map

| Section | File | Est. Tokens | Primary Consumer | Purpose |
|---------|------|-------------|-----------------|---------|
| Domain Capabilities | capabilities.md | ~1200 | product-owner, architect, story-writer | CAP-NNN capability catalog (25 atomic capabilities) |
| Domain Entities | entities.md | ~1400 | architect, product-owner, ux-designer | Entity model with attributes and relationships (expanded with client/server capability distinction, sampling, elicitation, roots, tasks) |
| Domain Invariants | invariants.md | ~1200 | product-owner, architect | DI-NNN business rules (18 invariants, +3 for client capabilities) |
| Domain Events | events.md | ~1200 | architect | Event triggers, preconditions, outcomes (expanded with server-initiated events) |
| Edge Cases | edge-cases.md | ~1200 | story-writer, test-writer | DEC-NNN domain-level edge cases (19 cases, +4 for server-initiated methods) |
| Assumptions | assumptions.md | ~1200 | product-owner, test-writer | ASM-NNN with validation methods (13 assumptions, +1, 2 partially validated) |
| Risks | risks.md | ~1300 | product-owner, architect | R-NNN risk register (13 risks, +1 for client capability complexity) |
| Failure Modes | failure-modes.md | ~1200 | architect, test-writer | FM-NNN runtime failure catalog (18 modes, +3 for server-initiated methods) |
| Differentiators | differentiators.md | ~1000 | product-owner | Competitive differentiator → CAP-NNN mapping (refined with research evidence) |
| Reconciliation Notes | reconciliation-notes.md | ~800 | all | Summary of domain research reconciliation changes |

## Cross-References

| If you need... | Read these together |
|----------------|-------------------|
| BC creation input | capabilities.md + invariants.md + edge-cases.md + assumptions.md + risks.md + differentiators.md |
| Architecture design input | capabilities.md + entities.md + invariants.md + events.md + risks.md + failure-modes.md |
| Story decomposition input | capabilities.md + edge-cases.md |
| Holdout scenario generation | assumptions.md + risks.md + failure-modes.md |
| NFR derivation | risks.md + failure-modes.md |
| Client capability implementation | entities.md (Sampling Request, Elicitation Request, Root, Task) + invariants.md (DI-016, DI-017, DI-018) + edge-cases.md (DEC-016–DEC-019) + failure-modes.md (FM-016–FM-018) |
| Security rule development | entities.md (Security Finding) + edge-cases.md (DEC-018) + differentiators.md (Differentiator 1) + risks.md (R-004, R-011) |
| Full domain review (adversary/spec-reviewer) | ALL sections |

## ID Registry Summary

| ID Format | Count | Section |
|-----------|-------|---------|
| CAP-NNN | 25 | capabilities.md |
| DI-NNN | 18 | invariants.md |
| DEC-NNN | 19 | edge-cases.md |
| ASM-NNN | 13 | assumptions.md |
| R-NNN | 13 | risks.md |
| FM-NNN | 18 | failure-modes.md |

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

## Key Changes in v1.1 (Domain Research Reconciliation)

1. **Client vs. Server capability distinction** — MCP separates server capabilities (tools, resources, prompts, logging, completions, tasks) from client capabilities (roots, sampling, elicitation, tasks). Forge MCP must advertise and handle client capabilities.
2. **rmcp SDK confirmed** — `warpdotdev/rmcp` with `ClientCapabilitiesBuilder` and `ServerCapabilities` structs. Community-maintained (by Warp), not directly Anthropic.
3. **OWASP AST10 categories enumerated** — All 10 categories identified; ~6 are runtime-detectable.
4. **Security research corpus** — BlueRock/Equixly CVEs, 36.7% SSRF rate, known attack vectors documented.
5. **Spec cadence faster than assumed** — ~6 months between updates (not 12).
6. **Streamable HTTP replaces deprecated SSE** — Transport correction.
7. **New entities, invariants, edge cases, failure modes, assumptions, risks** added for server-initiated methods.
