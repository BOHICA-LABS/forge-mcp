---
document_type: story
story_id: STORY-036
epic_id: EPIC-06
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-035]
blocks: [STORY-043, STORY-046]
behavioral_contracts: [BC-6.14.002]
verification_properties: [VP-007]
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-036: Alert State Machine (Normal → Breached → Recovered)

## Narrative
- **As an** AI Platform Engineer
- **I want to** see alerts transition through well-defined states with proper notifications
- **So that** I don't get alert storms and can see exactly when issues start and resolve

## Acceptance Criteria

### AC-001 (traces to BC-6.14.002 postcondition — normal to breached)
When `evaluate_thresholds()` returns a breach event and current state is `Normal`, state transitions to `Breached`. `E-MON-001` warning emitted: "Alert breached: <metric> exceeded <threshold>".
- **Test:** `test_BC_6_14_002_normal_to_breached()` (Kani proof for VP-007)

### AC-002 (traces to BC-6.14.002 postcondition — breached to recovered)
When `evaluate_thresholds()` returns no breach AND current state is `Breached`, state transitions to `Recovered` and immediately to `Normal`. `E-MON-002` info emitted: "Alert recovered: <metric> returned to normal".
- **Test:** `test_BC_6_14_002_breached_to_recovered()`

### AC-003 (traces to BC-6.14.002 invariant — no spurious transitions)
State machine only transitions on threshold evaluation events, not on timer ticks alone. No spurious Normal→Breached without an actual breach event.
- **Test:** Kani proof: ∀ states, transition only on AlertEvent, not time passing

### AC-004 (traces to BC-6.14.002 — consecutive breach events)
Multiple consecutive breach events while in `Breached` state produce no repeated E-MON-001 warnings (no alert storm). State remains `Breached` until recovery.
- **Test:** `test_BC_6_14_002_no_alert_storm()`

### AC-005 (traces to BC-6.14.002 — DI-009)
Alert state is per-server, per-metric (latency, error_rate, throughput independently tracked). One server can have latency Breached and error_rate Normal simultaneously.
- **Test:** `test_BC_6_14_002_per_metric_state()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `AlertStateMachine` | `forge-health/src/alert_sm.rs` | Pure |
| Alert event emission | `forge-health/src/alert_sm.rs` | Mixed |

## UX Screens
- SCR-005 (Health Metrics) — shows alert state badge

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | No metrics collected yet | State = Normal, no events |
| EC-002 | Threshold changes while Breached | Re-evaluate; may transition to Normal if new threshold accommodates |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| AlertStateMachine | Pure | Transition function takes event, returns new state |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~800 |
| BC-6.14.002 | ~500 |
| VP-007 proof | ~400 |
| **Total** | **~1,700** |
| Agent context window | 200K |
| **Budget usage** | **~0.9%** |

## Tasks

1. [ ] Write failing tests for all 5 ACs
2. [ ] Implement `AlertState` enum (Normal, Breached, Recovered)
3. [ ] Implement `AlertStateMachine` pure transition function
4. [ ] Implement per-metric state tracking (DI-009)
5. [ ] Implement anti-storm logic (deduplicate consecutive breach events)
6. [ ] Write Kani proof for VP-007 (valid transitions only)
7. [ ] Verify Red Gate
8. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-035 | evaluate_thresholds returns AlertEvent | AlertStateMachine consumes AlertEvent | Three separate state machines: latency, error_rate, throughput |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Pure state machine | purity-boundary-map.md | transition() takes (state, event) → state |
| No alert storm (DI-009) | module-decomposition.md | State-based dedup |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| kani | dev | VP-007 proof | `#[kani::proof]` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-health/src/alert_sm.rs` | AlertStateMachine | NO — this story creates it |
| `crates/forge-health/proofs/alert_sm.rs` | Kani VP-007 proof | NO |
