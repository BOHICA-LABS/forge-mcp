---
document_type: architecture-section
level: L3
section: verification-coverage-matrix
version: "1.0"
status: draft
producer: architect
timestamp: 2026-03-29T14:05:00
phase: 1b
inputs: [domain-spec/L2-INDEX.md, specs/prd.md]
traces_to: ARCH-INDEX.md
---

# Verification Coverage Matrix

Maps each VP to its target module/function and tracks coverage status.

## VP-to-Module Mapping

| VP ID | Module | Target Function/Area | Proof Method | Status | BC Source |
|-------|--------|---------------------|-------------|--------|-----------|
| VP-001 | forge-core | JSON-RPC message parsing | fuzz | draft | BC-2.04.001 |
| VP-002 | forge-core | Error classification (tool vs protocol) | kani | draft | BC-2.05.010 |
| VP-003 | forge-core | Pagination state machine termination | kani | draft | BC-2.05.001 |
| VP-004 | forge-discovery | Config parsing no-panic | fuzz+kani | draft | BC-1.01.002 |
| VP-005 | forge-traffic | Ring buffer bounds and FIFO ordering | kani | draft | BC-4.09.003 |
| VP-006 | forge-traffic | Message capture preserves content | proptest | draft | BC-4.09.001 |
| VP-007 | forge-health | Alert state machine transitions | kani | draft | BC-6.14.002 |
| VP-008 | forge-health | Latency histogram correctness | proptest | draft | BC-6.13.001 |
| VP-009 | forge-security | IP classification correctness | kani | draft | BC-7.16.002 |
| VP-010 | forge-security | Confidence score bounds | kani | draft | BC-7.16.004 |
| VP-011 | forge-security | Schema drift detection completeness | proptest | draft | BC-7.16.003 |
| VP-012 | forge-tui | TUI state machine no invalid states | proptest | draft | BC-3.07.001 |
| VP-013 | forge-core | Connection state machine validity | kani | draft | BC-1.02.003 |
| VP-014 | forge-traffic | Filter preserves message ordering | proptest | draft | BC-4.10.001 |

## Coverage Summary

| Module | VP Count | Kani | proptest | fuzz | Total |
|--------|----------|------|---------|------|-------|
| forge-core | 4 | 2 | 0 | 1 | 3+1 |
| forge-discovery | 1 | 1 | 0 | 1 | 1+1 |
| forge-traffic | 3 | 1 | 2 | 0 | 3 |
| forge-health | 2 | 1 | 1 | 0 | 2 |
| forge-security | 3 | 2 | 1 | 0 | 3 |
| forge-tui | 1 | 0 | 1 | 0 | 1 |
| **Total** | **14** | **7** | **5** | **2** | **14** |

## Coverage Gaps

Modules without VPs (by design — effectful-heavy or test-sufficient):

- **forge-daemon:** Effectful-heavy module. Verified through integration tests only.
- **forge-conformance:** Pure core covered by unit tests; shell requires mock server integration tests.
- **forge-config:** Fully pure but low criticality. Covered by proptest in diff correctness (tracked in tooling-selection, not as a VP due to low risk).
- **forge-mcp (binary):** Output formatting covered by proptest; no formal VP warranted for CLI layer.

## Traceability Notes

- Every VP traces back to a specific BC postcondition or domain invariant (DI-NNN).
- VP status lifecycle: `draft` → `harness_written` → `proven` → `verified`.
- VPs are individually tracked in `verification-properties/vp-NNN-*.md` files during Phase 3+.
