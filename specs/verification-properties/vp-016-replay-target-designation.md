---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-03-29T14:39:00
phase: 1d
inputs: [specs/prd.md, specs/architecture/ARCH-INDEX.md]
input-hash: ""
traces_to: specs/prd.md
source_bc: [BC-4.10.003]
module: forge-traffic
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

# VP-016: Replay Requires Explicit Target Designation

## Property Statement

The replay state machine in forge-traffic **cannot transition to the `Executing` state** from any state unless a target server has been explicitly designated. Specifically:

1. **No implicit target:** The `ReplayStateMachine` has no default target. The `target` field is `None` at construction and after reset.
2. **Transition guard:** The `start_replay()` transition is only valid when `target.is_some()` AND `target` was set by an explicit `set_target(server_name)` call.
3. **No original-server fallback:** There is no code path that infers the original capture source as the replay target. The original server ID is not stored in the replay state machine at all.
4. **Idle→Executing impossible without target:** From the `Idle` state, calling `start_replay()` with `target == None` returns an error and the state remains `Idle`.

This is a **state machine safety** property: it prevents accidental production mutation via replay.

## Source Contract

- **BC-4.10.003** — Message Sequence Replay Against Target Server. INV-001 (DI-007): "Replay requires explicit target designation — never implicit or default to original."
- **DI-007** — Replay requires explicit target designation.

## Proof Method

| Aspect | Detail |
|--------|--------|
| Method | Model checking (Kani) |
| Tool | `kani` crate |
| Bounds | All reachable states of the replay state machine (Idle, TargetSet, Executing, Paused, Complete, Error) |
| Unwind | 6 (one per state) |

## Harness Skeleton

```rust
#[cfg(kani)]
mod replay_safety_proofs {
    use super::*;

    /// VP-016-P1: Replay cannot execute without explicit target
    #[kani::proof]
    fn replay_never_executes_without_target() {
        let mut sm = ReplayStateMachine::new();

        // State machine starts with no target
        kani::assert(sm.target().is_none(), "Initial state must have no target");
        kani::assert(sm.state() == ReplayState::Idle, "Initial state must be Idle");

        // Attempt to start replay without setting target
        let result = sm.start_replay();
        kani::assert(result.is_err(), "start_replay without target must fail");
        kani::assert(sm.state() == ReplayState::Idle, "State must remain Idle on failed start");
    }

    /// VP-016-P2: Only explicit set_target populates the target field
    #[kani::proof]
    fn target_only_set_by_explicit_call() {
        let mut sm = ReplayStateMachine::new();

        // Non-deterministic server name
        let server: [u8; 8] = kani::any();
        let server_name = String::from_utf8_lossy(&server).to_string();

        // Target is None before set_target
        kani::assert(sm.target().is_none(), "Target must be None before set_target");

        sm.set_target(server_name.clone());
        kani::assert(
            sm.target() == Some(&server_name),
            "Target must equal the explicitly set value"
        );
    }

    /// VP-016-P3: Reset clears the target
    #[kani::proof]
    fn reset_clears_target() {
        let mut sm = ReplayStateMachine::new();

        let server: [u8; 8] = kani::any();
        let server_name = String::from_utf8_lossy(&server).to_string();

        sm.set_target(server_name);
        kani::assert(sm.target().is_some(), "Target should be set");

        sm.reset();
        kani::assert(sm.target().is_none(), "Reset must clear the target");
        kani::assert(sm.state() == ReplayState::Idle, "Reset must return to Idle");
    }

    /// VP-016-P4: All state transitions that reach Executing require target
    #[kani::proof]
    #[kani::unwind(6)]
    fn executing_state_implies_target_set() {
        let mut sm = ReplayStateMachine::new();

        // Apply a non-deterministic sequence of operations
        for _ in 0..5 {
            let action: u8 = kani::any();
            match action % 4 {
                0 => { let _ = sm.start_replay(); },
                1 => { sm.set_target("test-server".to_string()); },
                2 => { sm.reset(); },
                3 => { let _ = sm.pause(); },
                _ => unreachable!(),
            }

            // Invariant: if we're in Executing state, target must be Some
            if sm.state() == ReplayState::Executing {
                kani::assert(
                    sm.target().is_some(),
                    "INVARIANT VIOLATION: Executing state reached without target"
                );
            }
        }
    }
}
```

## State Machine Definition (for proof context)

```rust
#[derive(Debug, Clone, PartialEq)]
enum ReplayState {
    Idle,        // No replay in progress, no target set
    TargetSet,   // Target designated, ready to start
    Executing,   // Replay in progress
    Paused,      // Replay paused mid-sequence
    Complete,    // All messages replayed
    Error,       // Replay failed (target unreachable, etc.)
}
```

Valid transitions:
- `Idle` → `TargetSet` (via `set_target()`)
- `TargetSet` → `Executing` (via `start_replay()`)
- `Executing` → `Paused` / `Complete` / `Error`
- `Paused` → `Executing` (via `resume()`)
- `Any` → `Idle` (via `reset()`)
- `Idle` → `Executing` is **impossible** (the safety property)

## Feasibility Assessment

| Factor | Assessment |
|--------|-----------|
| State space | 6 states × 4 actions — small, fully enumerable |
| Complexity | Low — enum state machine with simple transition guards |
| Tool support | Excellent — Kani handles enum state machines natively |
| Time | Seconds (bounded state space) |
| Verdict | **FEASIBLE** |

## Composition

- Composes with VP-005 (ring buffer bounds) and VP-014 (filter ordering): replay reads from the capture buffer, so buffer integrity (VP-005) and filter ordering (VP-014) are prerequisites for correct replay input. VP-016 guarantees the replay *execution* is gated on explicit target.
- Composes with VP-006 (capture content integrity): replayed bytes must be identical to captured bytes (DI-005), and VP-016 ensures they go to the right target.

## Lifecycle

| Field | Value |
|-------|-------|
| Introduced | v0.1.0 (ADV-P1-011 resolution) |
| Modified | — |
| Deprecated | — |
| Retired | — |
