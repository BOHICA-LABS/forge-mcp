---
document_type: holdout-scenario-index
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-03-29T11:25:00
phase: 1a
traces_to: ../specs/prd.md
---

# Holdout Scenario Index — Forge MCP

## Purpose

These holdout scenarios test real-world usage patterns that the builder and
test-writer **must not see** during implementation. They are used for
post-implementation validation (Phase 3.5) to ensure the product handles
scenarios that were not explicitly coded for.

## Information Asymmetry Wall

- **Builder sees:** PRD, BCs, architecture, stories, test vectors
- **Builder does NOT see:** These holdout scenarios
- **Holdout evaluator sees:** These scenarios + the built product
- **Violation:** If a builder references HS-NNN in code or tests, the holdout is contaminated

## Scenario Registry

| ID | Title | Category | Source | Priority |
|----|-------|----------|--------|----------|
| [HS-001](HS-001.md) | Multi-server workflow: discover, inspect, compare | Integration | ASM-002 | P0 |
| [HS-002](HS-002.md) | Server crash during TUI traffic inspection | Resilience | FM-002 | P0 |
| [HS-003](HS-003.md) | Security audit discovers actual SSRF vulnerability | Security | R-004 | P0 |
| [HS-004](HS-004.md) | Config drift across 3+ editors with dual schema | Config | ASM-002 | P0 |
| [HS-005](HS-005.md) | Performance under high message volume (1000+ msg/sec) | Performance | R-005 | P1 |
| [HS-006](HS-006.md) | Rug pull attack: tool schema changes between connections | Security | R-014 | P0 |
| [HS-007](HS-007.md) | Conformance test against incomplete server | Conformance | DEC-003 | P1 |
| [HS-008](HS-008.md) | Agent workflow: discover → inspect → call in < 500 tokens | Agent UX | NFR-003 | P1 |
| [HS-009](HS-009.md) | Pagination with misbehaving server (cursor loop) | Resilience | ASM-014 | P1 |
| [HS-010](HS-010.md) | Sampling proxy failure with graceful degradation | Resilience | ASM-013 | P1 |
| [HS-011](HS-011.md) | Elicitation request in non-interactive CLI mode | Edge Case | DEC-017 | P1 |
| [HS-012](HS-012.md) | Cross-platform daemon lifecycle | Daemon | ASM-005 | P0 |
| [HS-013](HS-013.md) | Mixed protocol version environment | Protocol | DEC-015 | P1 |
| [HS-014](HS-014.md) | Security finding suppression and re-classification | Security | DEC-013 | P1 |
| [HS-015](HS-015.md) | Known-good corpus: filesystem server | Corpus (FP) | Real-world | P0 |
| [HS-016](HS-016.md) | Known-problematic corpus: server with known CVEs | Corpus (FN) | Real-world | P0 |
| [HS-017](HS-017.md) | TUI with minimal terminal (80×24, 16-color, no Unicode) | Accessibility | FM-013 | P1 |
| [HS-018](HS-018.md) | Concurrent TUI and CLI access to same server | Concurrency | DI-003 | P1 |

## Category Distribution

- **Security:** HS-003, HS-006, HS-014 (3 scenarios)
- **Resilience:** HS-002, HS-009, HS-010 (3 scenarios)
- **Integration/Config:** HS-001, HS-004 (2 scenarios)
- **Performance:** HS-005 (1 scenario)
- **Conformance:** HS-007 (1 scenario)
- **Agent UX:** HS-008 (1 scenario)
- **Edge Case:** HS-011 (1 scenario)
- **Daemon:** HS-012 (1 scenario)
- **Protocol:** HS-013 (1 scenario)
- **Corpus (Real-world):** HS-015, HS-016 (2 scenarios)
- **Accessibility:** HS-017 (1 scenario)
- **Concurrency:** HS-018 (1 scenario)

## Scoring Summary

Each scenario is scored on a 0–3 rubric per criterion:
- **3** — Fully satisfied, no defects
- **2** — Mostly satisfied, minor issues
- **1** — Partially satisfied, significant gaps
- **0** — Not satisfied or broken

**Pass threshold:** Every must-pass criterion scores ≥ 2. Overall scenario score ≥ 70%.
