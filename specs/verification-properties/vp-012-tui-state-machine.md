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
source_bc: [BC-3.07.001]
module: forge-tui
proof_method: proptest
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

# VP-012: TUI State Machine No Invalid States

## Property Statement

For any sequence of keyboard/mouse events, the TUI state machine (panel focus, navigation mode, input mode) MUST NOT enter an invalid state. Additionally:

1. All states MUST be reachable from the initial state.
2. The quit action MUST be reachable from any state.

This is a **state machine safety** property: the TUI MUST NOT get "stuck" or enter a corrupted state regardless of user input.

## Source Contract

- **BC-3.07.001** — Multi-panel TUI with keyboard-driven navigation, panel focus management, and input mode switching.

## Proof Method

| Aspect | Detail |
|--------|--------|
| Method | Property-based testing (proptest) |
| Tool | `proptest` crate |
| Input space | Arbitrary event sequences (0..200 events) |
| Event types | Keyboard keys, mouse clicks, resize events |
| Iterations | 256+ cases |
| Shrinking | Automatic — proptest shrinks failing cases to minimal reproduction |

## Harness Skeleton

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn tui_state_always_valid(
        events in prop::collection::vec(arb_tui_event(), 0..200)
    ) {
        let mut state = TuiState::initial();
        for event in events {
            state = state.handle_event(event);
            prop_assert!(
                state.is_valid(),
                "Invalid state after event: {:?}",
                event
            );
        }
    }
}
```

### Strategy: `arb_tui_event`

Generates arbitrary TUI events covering all input types:

```rust
fn arb_tui_event() -> impl Strategy<Value = TuiEvent> {
    prop_oneof![
        // Navigation keys
        Just(TuiEvent::Key(KeyCode::Tab)),
        Just(TuiEvent::Key(KeyCode::BackTab)),
        Just(TuiEvent::Key(KeyCode::Up)),
        Just(TuiEvent::Key(KeyCode::Down)),
        Just(TuiEvent::Key(KeyCode::Left)),
        Just(TuiEvent::Key(KeyCode::Right)),
        Just(TuiEvent::Key(KeyCode::Enter)),
        Just(TuiEvent::Key(KeyCode::Esc)),
        // Character input
        any::<char>().prop_map(|c| TuiEvent::Key(KeyCode::Char(c))),
        // Mouse events
        (0u16..200, 0u16..50).prop_map(|(x, y)| TuiEvent::Mouse(x, y)),
        // Resize
        (10u16..300, 5u16..100).prop_map(|(w, h)| TuiEvent::Resize(w, h)),
    ]
}
```

### Validity Predicate: `is_valid`

The `TuiState::is_valid()` method checks:

```rust
impl TuiState {
    fn is_valid(&self) -> bool {
        // Panel focus is within valid range
        self.focused_panel < self.panel_count
        // Navigation mode and input mode are not simultaneously active
        && !(self.nav_mode && self.input_mode)
        // At least one panel exists
        && self.panel_count > 0
        // Scroll positions are within bounds
        && self.scroll_offset <= self.content_height
    }
}
```

### Reachability Sub-Property

A separate test verifies all states are reachable:

```rust
proptest! {
    #[test]
    fn quit_always_reachable(
        events in prop::collection::vec(arb_tui_event(), 0..100)
    ) {
        let mut state = TuiState::initial();
        for event in &events {
            state = state.handle_event(event.clone());
        }
        // From any state, pressing 'q' in nav mode should reach quit
        let state = state.handle_event(TuiEvent::Key(KeyCode::Esc)); // ensure nav mode
        let state = state.handle_event(TuiEvent::Key(KeyCode::Char('q')));
        prop_assert!(state.is_quit_requested());
    }
}
```

## Feasibility Assessment

| Factor | Assessment |
|--------|-----------|
| Input space | Bounded event sequences (0..200 events), finite event types |
| Complexity | Medium — depends on TUI state machine complexity |
| Tool support | Excellent — proptest with enum strategies |
| Precondition | TUI state machine MUST use bounded enum states (not arbitrary values) |
| Expected time | Milliseconds per case |
| Verdict | **FEASIBLE** |

## Lifecycle

| Field | Value |
|-------|-------|
| Introduced | v0.1.0 |
| Modified | — |
| Deprecated | — |
| Retired | — |
