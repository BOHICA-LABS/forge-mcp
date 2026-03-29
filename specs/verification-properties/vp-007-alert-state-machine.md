---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-03-29T14:05:00
phase: 1b
inputs: [specs/prd.md, specs/architecture/ARCH-INDEX.md]
input-hash: ""
traces_to: specs/prd.md
source_bc: [BC-6.14.002]
module: forge-health
proof_method: kani
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: v0.1.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-007: Alert State Machine Transitions

## Property Statement

The alert state machine MUST have exactly 3 states: **Normal**, **Breached**, **Recovered**. Valid transitions are:

- `Normal → Breached` (metric exceeds threshold)
- `Breached → Recovered` (metric returns below threshold)
- `Recovered → Breached` (metric exceeds threshold again)

No other transitions are permitted. Self-transitions (remaining in the same state) are allowed. Duplicate breach alerts MUST NOT be emitted for the same threshold crossing.

This is a **state machine safety** property: the alert system only makes valid transitions and never produces spurious alerts.

## Source Contract

- **BC-6.14.002** — Alert system with configurable thresholds and state-based alerting.
- **DI-009** — Alert state machine invariant: only valid transitions are permitted; no duplicate alerts.

## Proof Method

| Aspect | Detail |
|--------|--------|
| Method | Model checking (Kani) |
| Tool | Kani Rust Verifier |
| Input space | Bounded metric value sequence (max 10 events), threshold = 100.0 |
| Bound | `#[kani::unwind(11)]` |
| Expected time | Seconds |

## Harness Skeleton

```rust
#[kani::proof]
#[kani::unwind(11)]
fn verify_alert_state_machine() {
    let threshold: f64 = 100.0;
    let mut state = AlertState::Normal;

    let num_events: u8 = kani::any();
    kani::assume(num_events <= 10);
    let mut breach_count = 0u32;
    let mut last_breach_value: Option<f64> = None;

    for _ in 0..num_events {
        let value: f64 = kani::any();
        kani::assume(value >= 0.0 && value <= 1000.0);

        let (new_state, alert) = state.evaluate(value, threshold);

        match (&state, &new_state) {
            (AlertState::Normal, AlertState::Breached) => {
                breach_count += 1;
            }
            (AlertState::Breached, AlertState::Recovered) => {}
            (AlertState::Recovered, AlertState::Breached) => {
                breach_count += 1;
            }
            (s, ns) if std::mem::discriminant(s) == std::mem::discriminant(ns) => {}
            _ => panic!("Invalid transition"),
        }
        state = new_state;
    }
}
```

### State Machine Diagram

```
         exceeds threshold
Normal ─────────────────────► Breached
                                  │
                   below threshold │
                                  ▼
                              Recovered
                                  │
                  exceeds threshold│
                                  ▼
                              Breached
                                (cycle)
```

### No-Duplicate-Alert Sub-Property

A supplementary harness verifies that consecutive evaluations in the `Breached` state do not emit additional alerts:

```rust
#[kani::proof]
fn verify_no_duplicate_breach_alert() {
    let threshold: f64 = 100.0;
    let mut state = AlertState::Normal;

    // First breach
    let (state, alert1) = state.evaluate(150.0, threshold);
    assert!(alert1.is_some()); // alert emitted

    // Stay breached — no new alert
    let (state, alert2) = state.evaluate(200.0, threshold);
    assert!(alert2.is_none()); // no duplicate
}
```

## Feasibility Assessment

| Factor | Assessment |
|--------|-----------|
| Input space | 3 states, bounded events (10), bounded metric values |
| Complexity | Low — simple state machine with 3 states |
| Tool support | Kani handles enum-based state machines naturally |
| Expected time | Seconds |
| Verdict | **FEASIBLE** |

## Lifecycle

| Field | Value |
|-------|-------|
| Introduced | v0.1.0 |
| Modified | — |
| Deprecated | — |
| Retired | — |
