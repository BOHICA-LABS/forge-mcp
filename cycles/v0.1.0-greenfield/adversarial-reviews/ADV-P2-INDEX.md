---
document_type: adversarial-review
pass: 2
phase: 1d
previous_review: ADV-P1-INDEX.md
---

# ADV-P2 Index

## Finding Summary

| ID | Severity | Category | Title |
|----|----------|----------|-------|
| ADV-P2-001 | HIGH | verification-gap | Daemon multiplexing isolation is still unverified despite shared-session requirement |
| ADV-P2-002 | HIGH | contradiction | HTTP transport contract still references error codes that now mean different failures |
| ADV-P2-003 | MEDIUM | contradiction | Schema-drift coverage still disagrees on whether annotations are part of the contract |
| ADV-P2-004 | MEDIUM | ux-inconsistency | Traffic replay UX still omits the mandatory explicit target-selection step |

## Category Groups

### Verification Gaps
- ADV-P2-001

### Contradictions
- ADV-P2-002
- ADV-P2-003

### UX Inconsistencies
- ADV-P2-004

## Fix Verification Notes

- **ADV-P1-003 / ADV-P1-006 (suppression overlay):** Pass 1 fix appears correctly applied. `BC-7.18.002`, `SCR-007`, `FLOW-005`, and new `VP-015` now align on overlay semantics (`suppressed: true`, not deleted, export preserved).
- **ADV-P1-001 (deprecated SSE example):** Pass 1 fix appears correctly applied. `BC-1.02.002` now states Streamable HTTP single-endpoint semantics and explicitly forbids deprecated SSE transport.
- **ADV-P1-009 (replay target UX):** Not actually closed. Replay contract still requires explicit target designation, but `FLOW-003` / `SCR-004` still model replay as immediate `R` action.
- **ADV-P1-010 / ADV-P1-016 (shared-session routing + daemon VP):** Not actually closed. Holdout HS-018 still has no matching L4 property, and architecture still marks `forge-daemon` as a no-VP module.

## Severity Totals
- HIGH: 2
- MEDIUM: 2
- LOW: 0
- CRITICAL: 0

## Convergence Signal

Not converged. Remaining issues are structural, not cosmetic: pooled-session correctness is still orphaned at L4, error-code semantics are still internally contradictory, and replay UX still violates the safety contract it is supposed to enforce.
