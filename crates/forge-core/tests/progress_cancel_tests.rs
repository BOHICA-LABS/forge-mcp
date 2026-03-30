//! Integration tests for STORY-022: Progress Tracking, Cancellation & Error Distinction
//!
//! AC-001 — progress notifications emitted via broadcast channel
//! AC-002 — cancel_request() sends $/cancel notification
//! AC-003 — orphaned/cancelled progress silently dropped (E-PRO-006)
//! AC-004 — call_tool() isError:true → Ok(ToolResult { is_error: true })
//! AC-005 — ToolResult Display: ✓ success / ✗ error, no panic

#![allow(non_snake_case)]

use forge_core::events::{ProgressBus, ProgressEvent};
use forge_core::protocol::ToolResult;
use rmcp::model::{NumberOrString, ProgressToken};

// ── Helper: progress token constructor ───────────────────────────────────────

fn token(s: &str) -> ProgressToken {
    ProgressToken(NumberOrString::String(s.to_string().into()))
}

// ── AC-001: Progress notification received via event bus ─────────────────────

/// AC-001: When a `notifications/progress` is dispatched, subscribers receive
/// a `ProgressEvent` with the matching token, current, and total values.
#[tokio::test]
async fn test_BC_2_05_009_progress_notification_received() {
    let bus = ProgressBus::new();
    let mut rx = bus.subscribe();

    // Simulate a progress notification arriving from the server.
    let tok = token("req-001");
    bus.dispatch(ProgressEvent {
        token: tok.clone(),
        current: 3.0,
        total: Some(10.0),
    });

    // Subscriber must receive the event.
    let event = rx.try_recv().expect("should have received a progress event");
    assert_eq!(event.current, 3.0);
    assert_eq!(event.total, Some(10.0));
    // Token must match.
    let ProgressToken(ref inner) = event.token;
    assert_eq!(inner.to_string(), "req-001");
}

// ── AC-002: cancel_request() sends $/cancel notification ─────────────────────

/// AC-002: `ProgressBus::cancel_request(id)` records the ID in the cancelled
/// set so subsequent progress for that ID is silently dropped (AC-003).
///
/// Note: The actual `$/cancel` notification is sent over the wire by the
/// rmcp `Peer::notify_cancelled()` method in the connection layer.  This test
/// validates the observable contract: after cancellation, the bus knows the
/// request is cancelled and drops further progress for it.
#[tokio::test]
async fn test_BC_2_05_009_cancel_request() {
    let bus = ProgressBus::new();
    let mut rx = bus.subscribe();

    let tok = token("req-to-cancel");

    // Before cancel: dispatch reaches the subscriber.
    bus.dispatch(ProgressEvent {
        token: tok.clone(),
        current: 1.0,
        total: Some(5.0),
    });
    assert!(rx.try_recv().is_ok(), "first progress before cancel should arrive");

    // Cancel the request.
    bus.cancel_request(tok.clone());

    // After cancel: dispatch for this token is silently dropped (E-PRO-006).
    bus.dispatch(ProgressEvent {
        token: tok.clone(),
        current: 2.0,
        total: Some(5.0),
    });
    // The cancelled request's progress must NOT appear in the channel.
    assert!(
        rx.try_recv().is_err(),
        "progress after cancel must be silently dropped"
    );
}

// ── AC-003: Orphaned progress silently ignored (E-PRO-006) ───────────────────

/// AC-003: Progress notifications for unknown request IDs (no subscriber
/// registered) are silently dropped.  No panic, no error propagated to caller.
#[tokio::test]
async fn test_BC_2_05_009_orphaned_progress_ignored() {
    let bus = ProgressBus::new();
    // We subscribe so the broadcast channel is alive, but we never register
    // a specific token — the token is entirely unknown.
    let mut _rx = bus.subscribe();

    // Dispatching for an unknown token must not panic.
    bus.dispatch(ProgressEvent {
        token: token("totally-unknown-req"),
        current: 7.0,
        total: None,
    });

    // No assertions needed — if we reach here without a panic, the test passes.
    // The event IS broadcast to all subscribers (AC-001 semantics), but the
    // important invariant is: no error is propagated and no panic occurs.
}

// ── AC-004: call_tool() isError:true → Ok(ToolResult { is_error: true }) ─────

/// AC-004: When the server returns `isError: true`, `call_tool()` must return
/// `Ok(ToolResult { is_error: true })`, NOT `Err(...)`.
///
/// This test validates the pure conversion (ToolResult::from(CallToolResult)).
/// The integration-level test with a real server lives in protocol_tests.rs.
#[test]
fn test_BC_2_05_010_tool_error_is_ok_result() {
    use rmcp::model::CallToolResult;

    // isError: true → Ok(ToolResult { is_error: true })
    let raw = CallToolResult::error(vec![rmcp::model::Content::text("something went wrong")]);
    let result = ToolResult::from(raw);

    assert!(
        result.is_error,
        "isError: true in CallToolResult must produce ToolResult.is_error = true"
    );
    // Must not be an Err — callers must receive Ok.
    // (The static type guarantees this: `ToolResult::from` returns ToolResult, not Result<...>)
    // We additionally verify content is preserved.
    assert!(!result.content.is_empty(), "error result must carry content");
}

// ── AC-005: ToolResult Display — ✓ / ✗, no panic ────────────────────────────

/// AC-005: `ToolResult` has a `Display` impl that renders green ✓ for success
/// and red ✗ for errors.  Neither variant panics.
#[test]
fn test_BC_2_05_010_error_result_display_not_panic() {
    use rmcp::model::{CallToolResult, Content};

    // Success result.
    let success = ToolResult::from(CallToolResult::success(vec![Content::text("all good")]));
    let success_str = format!("{success}");
    // Must contain the check mark and not panic.
    assert!(
        success_str.contains('✓') || success_str.contains("ok") || success_str.contains("success"),
        "success display should contain ✓ or equivalent, got: {success_str}"
    );

    // Error result.
    let error = ToolResult::from(CallToolResult::error(vec![Content::text("it broke")]));
    let error_str = format!("{error}");
    // Must contain the cross and not panic.
    assert!(
        error_str.contains('✗') || error_str.contains("error") || error_str.contains("err"),
        "error display should contain ✗ or equivalent, got: {error_str}"
    );

    // The two formats must differ.
    assert_ne!(
        success_str, error_str,
        "success and error display strings must differ"
    );
}
