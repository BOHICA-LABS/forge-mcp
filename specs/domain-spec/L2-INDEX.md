---
document_type: domain-spec-index
level: L2
version: "1.2"
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

> **v1.2 — Full domain research reconciliation.** This version incorporates
> comprehensive findings from MCP protocol research including: complete JSON-RPC
> method enumeration (~25 methods), dual config schema support (`mcpServers` vs
> `servers`), cursor-based pagination, batch JSON-RPC, tool error semantics
> (`isError` vs JSON-RPC errors), resource subscriptions, notification inventory,
> progress/cancellation protocol, rug pull attack patterns, Streamable HTTP
> session management, and per-editor config file paths. See
> `reconciliation-notes.md` for a summary of all changes.

## Domain Summary

Forge MCP is a Rust-based command-line tool that provides a unified binary for
discovering, inspecting, debugging, monitoring, and security-auditing MCP (Model
Context Protocol) servers. It combines a scriptable CLI, an interactive TUI
dashboard, and a protocol conformance suite — targeting AI platform engineers,
DevEx engineers, agent developers, security teams, and MCP server authors.

## Document Map

| Section | File | Est. Tokens | Primary Consumer | Purpose |
|---------|------|-------------|-----------------|---------|
| Domain Capabilities | capabilities.md | ~1400 | product-owner, architect, story-writer | CAP-NNN capability catalog (25 atomic capabilities, refined with full MCP method list, pagination, notifications) |
| Domain Entities | entities.md | ~1800 | architect, product-owner, ux-designer | Entity model with attributes and relationships (expanded with Resource Subscription, Paginated List Response, Transport detail, Config Source paths, batch JSON-RPC, tool error semantics) |
| Domain Invariants | invariants.md | ~1400 | product-owner, architect | DI-NNN business rules (20 invariants, +5 for client capabilities, pagination, tool errors) |
| Domain Events | events.md | ~1400 | architect | Event triggers, preconditions, outcomes (expanded with server-initiated events, schema drift detection) |
| Edge Cases | edge-cases.md | ~1500 | story-writer, test-writer | DEC-NNN domain-level edge cases (24 cases, +9 for server-initiated methods, pagination, batch JSON-RPC, progress/cancellation) |
| Assumptions | assumptions.md | ~1400 | product-owner, test-writer | ASM-NNN with validation methods (15 assumptions, 2 partially validated, +2 for pagination and rmcp handler extensibility) |
| Risks | risks.md | ~1500 | product-owner, architect | R-NNN risk register (15 risks, +2 for rug pull attacks and HTTP session complexity) |
| Failure Modes | failure-modes.md | ~1500 | architect, test-writer | FM-NNN runtime failure catalog (21 modes, +6 for server-initiated methods, pagination, rug pull, HTTP sessions) |
| Differentiators | differentiators.md | ~1100 | product-owner | Competitive differentiator → CAP-NNN mapping (refined with schema drift detection, research evidence) |
| Reconciliation Notes | reconciliation-notes.md | ~1000 | all | Summary of domain research reconciliation changes |

## Cross-References

| If you need... | Read these together |
|----------------|-------------------|
| BC creation input | capabilities.md + invariants.md + edge-cases.md + assumptions.md + risks.md + differentiators.md |
| Architecture design input | capabilities.md + entities.md + invariants.md + events.md + risks.md + failure-modes.md |
| Story decomposition input | capabilities.md + edge-cases.md |
| Holdout scenario generation | assumptions.md + risks.md + failure-modes.md |
| NFR derivation | risks.md + failure-modes.md |
| Client capability implementation | entities.md (Sampling Request, Elicitation Request, Root, Task) + invariants.md (DI-016, DI-017, DI-018) + edge-cases.md (DEC-016–DEC-019) + failure-modes.md (FM-016–FM-018) |
| Config parsing implementation | entities.md (Config Source with OS paths) + capabilities.md (CAP-001 dual schema) + edge-cases.md (DEC-007–DEC-010) + failure-modes.md (FM-006–FM-009) |
| Pagination handling | entities.md (Paginated List Response) + invariants.md (DI-019) + edge-cases.md (DEC-020, DEC-021) + failure-modes.md (FM-019) + assumptions.md (ASM-014) |
| Security rule development | entities.md (Security Finding) + edge-cases.md (DEC-018, DEC-023) + differentiators.md (Differentiator 1) + risks.md (R-004, R-011, R-014) + events.md (SchemaDriftDetected) + failure-modes.md (FM-020) |
| Traffic capture and protocol correctness | entities.md (JSON-RPC Message, Resource Subscription) + invariants.md (DI-005, DI-006, DI-020) + edge-cases.md (DEC-022, DEC-023, DEC-024) + assumptions.md (ASM-015) |
| Full domain review (adversary/spec-reviewer) | ALL sections |

## ID Registry Summary

| ID Format | Count | Section |
|-----------|-------|---------|
| CAP-NNN | 25 | capabilities.md |
| DI-NNN | 20 | invariants.md |
| DEC-NNN | 24 | edge-cases.md |
| ASM-NNN | 15 | assumptions.md |
| R-NNN | 15 | risks.md |
| FM-NNN | 21 | failure-modes.md |

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

## Key Changes in v1.2 (Full Domain Research Reconciliation)

### Carried from v1.1
1. **Client vs. Server capability distinction** — MCP separates server capabilities (tools, resources, prompts, logging, completions, tasks) from client capabilities (roots, sampling, elicitation, tasks). Forge MCP must advertise and handle client capabilities.
2. **rmcp SDK confirmed** — `warpdotdev/rmcp` with `ClientCapabilitiesBuilder` and `ServerCapabilities` structs. Community-maintained (by Warp), not directly Anthropic.
3. **OWASP AST10 categories enumerated** — All 10 categories identified; ~6 are runtime-detectable.
4. **Security research corpus** — BlueRock/Equixly CVEs, 36.7% SSRF rate, known attack vectors documented.
5. **Spec cadence faster than assumed** — ~6 months between updates (not 12).
6. **Streamable HTTP replaces deprecated SSE** — Transport correction.
7. **New entities, invariants, edge cases, failure modes, assumptions, risks** added for server-initiated methods.

### New in v1.2
8. **VS Code config correction** — VS Code uses `mcp.json` (not `settings.json`), top-level key `"servers"` (not `"mcpServers"`), with explicit `"type"` field. CAP-001 now specifies dual schema parsing.
9. **Complete MCP method inventory** — CAP-005 now enumerates all ~25 distinct method names including notifications, `ping`, `resources/subscribe`/`unsubscribe`, and exact method names (`completion/complete` not `completions/complete`).
10. **Cursor-based pagination** — New entity (Paginated List Response), new invariant (DI-019), new edge cases (DEC-020, DEC-021), new failure mode (FM-019), new assumption (ASM-014).
11. **Batch JSON-RPC support** — JSON-RPC Message entity refined with batch type, new edge case (DEC-022) for response ordering.
12. **Tool error semantics** — New invariant (DI-020) distinguishing `result.isError` from JSON-RPC errors. New edge case (DEC-023).
13. **Progress and cancellation protocol** — New edge case (DEC-024) for post-cancellation progress notifications.
14. **Rug pull attack pattern** — New failure mode (FM-020), new risk (R-014), new event (SchemaDriftDetected). Schema drift detection elevated to security-critical.
15. **Streamable HTTP session management** — New failure mode (FM-021), new risk (R-015) for `Mcp-Session-Id` and session recovery.
16. **Resource subscriptions** — New entity (Resource Subscription) for `resources/subscribe`/`unsubscribe`.
17. **Per-editor config file paths** — Config Source entity now includes OS-specific paths for all 4 editors.
18. **Transport detail expanded** — Transport Connection entity now includes stdio message format (UTF-8, newline-delimited), Streamable HTTP response modes (JSON vs SSE), session headers.
19. **rmcp handler extensibility assumption** — New assumption (ASM-015) for traffic capture via handler interception.
