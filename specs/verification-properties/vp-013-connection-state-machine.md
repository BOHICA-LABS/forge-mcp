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
source_bc: [BC-1.02.003]
module: forge-core
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

# VP-013: Connection State Machine Validity

## Property Statement

The connection state machine (`Disconnected → Connecting → Connected → Error`) MUST only make **valid transitions**:

- `Disconnected → Connecting` (initiate connection)
- `Connecting → Connected` (connection established)
- `Connecting → Error` (connection failed)
- `Connected → Error` (connection lost)
- `Connected → Disconnected` (graceful disconnect)
- `Error → Connecting` (reconnect attempt)
- `Error → Disconnected` (give up)

No other transitions are permitted. Self-transitions (remaining in the same state) are permitted as no-ops. `Connected` MUST only be reachable after passing through `Connecting`.

This is a **state machine safety** property: the connection lifecycle MUST only traverse defined transitions and MUST NOT reach invalid states.

## Source Contract

- **BC-1.02.003** — Connection lifecycle management with reconnection support and graceful shutdown.

## Proof Method

| Aspect | Detail |
|--------|--------|
| Method | Model checking (Kani) |
| Tool | Kani Rust Verifier |
| Input space | 4 states × arbitrary event sequences (max 10 events) |
| Bound | `#[kani::unwind(11)]` |
| Expected time | Seconds |

## Harness Skeleton

```rust
#[kani::proof]
#[kani::unwind(11)]
fn verify_connection_state_machine() {
    let mut state = ConnectionState::Disconnected;
    let num_events: u8 = kani::any();
    kani::assume(num_events <= 10);

    for _ in 0..num_events {
        let event: ConnectionEvent = kani::any();
        let new_state = state.transition(event);

        match (&state, &new_state) {
            (ConnectionState::Disconnected, ConnectionState::Connecting) => {}
            (ConnectionState::Connecting, ConnectionState::Connected) => {}
            (ConnectionState::Connecting, ConnectionState::Error) => {}
            (ConnectionState::Connected, ConnectionState::Error) => {}
            (ConnectionState::Connected, ConnectionState::Disconnected) => {} // graceful
            (ConnectionState::Error, ConnectionState::Connecting) => {} // reconnect
            (ConnectionState::Error, ConnectionState::Disconnected) => {} // give up
            (s, ns) if std::mem::discriminant(s) == std::mem::discriminant(ns) => {} // no-op
            _ => panic!("Invalid transition: {:?} -> {:?}", state, new_state),
        }
        state = new_state;
    }
}
```

### State Machine Diagram

```
                    initiate
Disconnected ──────────────────► Connecting
     ▲                            │      │
     │                   success  │      │ failure
     │                            ▼      ▼
     │ graceful              Connected   Error
     │ disconnect                │        │  │
     └───────────────────────────┘        │  │
     │                                    │  │
     │              give up               │  │
     └────────────────────────────────────┘  │
                                             │
                    reconnect                │
               Connecting ◄──────────────────┘
```

### Reachability Sub-Property

`Connected` is only reachable via `Connecting`:

```rust
#[kani::proof]
fn verify_connected_requires_connecting() {
    let mut state = ConnectionState::Disconnected;
    let mut was_connecting = false;

    let num_events: u8 = kani::any();
    kani::assume(num_events <= 10);

    for _ in 0..num_events {
        let event: ConnectionEvent = kani::any();
        let new_state = state.transition(event);

        if matches!(state, ConnectionState::Connecting) {
            was_connecting = true;
        }
        if matches!(new_state, ConnectionState::Connected) {
            assert!(was_connecting, "Reached Connected without Connecting");
        }
        state = new_state;
    }
}
```

## Feasibility Assessment

| Factor | Assessment |
|--------|-----------|
| Input space | 4 states, bounded event sequence (10 events) |
| Complexity | Low — well-defined state machine with few states |
| Tool support | Kani excels at enum-based state machine verification |
| Expected time | Seconds |
| Verdict | **FEASIBLE** |

## Lifecycle

| Field | Value |
|-------|-------|
| Introduced | v0.1.0 |
| Modified | — |
| Deprecated | — |
| Retired | — |
