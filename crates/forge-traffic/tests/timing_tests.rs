//! Failing tests for STORY-028 — Per-Message Timing & Throughput Analysis.
//!
//! All tests in this file are designed to COMPILE but FAIL (Red Gate) until the
//! implementation in `timing.rs` and `throughput.rs` replaces the `todo!()`
//! stubs.
//!
//! Naming follows the BC-based convention required by AGENTS.md:
//!   `test_BC_S_SS_NNN_<assertion>()`
//! Edge-case tests follow the EC-NNN convention from the story edge-case catalog.
//!
//! Traces to: BC-4.09.002, VP-006

use std::time::{Duration, Instant};

use forge_core::events::{MessageCaptured, MessageDirection};
use forge_traffic::{TimedMessage, TimingAnalyzer, ThroughputWindow};
use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Build a minimal `MessageCaptured` whose payload is an MCP JSON-RPC *request*
/// with the given numeric `rpc_id`.
fn make_request(direction: MessageDirection, rpc_id: u64, timestamp: Instant) -> MessageCaptured {
    MessageCaptured {
        id: Uuid::new_v4(),
        direction,
        method: Some("tools/call".to_string()),
        payload: serde_json::json!({
            "jsonrpc": "2.0",
            "id": rpc_id,
            "method": "tools/call",
            "params": {}
        }),
        timestamp,
    }
}

/// Build a minimal `MessageCaptured` whose payload is an MCP JSON-RPC *response*
/// matching the given `rpc_id`.
fn make_response(direction: MessageDirection, rpc_id: u64, timestamp: Instant) -> MessageCaptured {
    MessageCaptured {
        id: Uuid::new_v4(),
        direction,
        method: None,
        payload: serde_json::json!({
            "jsonrpc": "2.0",
            "id": rpc_id,
            "result": {}
        }),
        timestamp,
    }
}

/// Build a notification (no `id` field) — EC-001.
fn make_notification(direction: MessageDirection, timestamp: Instant) -> MessageCaptured {
    MessageCaptured {
        id: Uuid::new_v4(),
        direction,
        method: Some("notifications/progress".to_string()),
        payload: serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/progress",
            "params": { "progressToken": 1, "progress": 0.5 }
        }),
        timestamp,
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-001 — Per-message latency
// ─────────────────────────────────────────────────────────────────────────────

/// AC-001: A request/response pair matched by JSON-RPC `id` should have
/// `latency_ms` equal to the elapsed time between the two `MessageCaptured`
/// events (within ±5 ms tolerance to absorb minor floating-point noise).
///
/// Traces to: BC-4.09.002 postcondition — per-message latency.
#[test]
fn test_BC_4_09_002_per_message_latency() {
    let t0 = Instant::now();
    let t1 = t0 + Duration::from_millis(100);

    let request = make_request(MessageDirection::ClientToServer, 1, t0);
    let response = make_response(MessageDirection::ServerToClient, 1, t1);

    let mut analyzer = TimingAnalyzer::new();

    // Feed the request — no latency available yet.
    let req_timed = analyzer.process_message(&request);
    // Feeding a request may return Some (without latency) or None; either is
    // acceptable as long as the response carries the latency.

    // Feed the matching response — latency should now be ~100 ms.
    let resp_timed = analyzer
        .process_message(&response)
        .expect("response should produce a TimedMessage");

    let latency = resp_timed
        .latency_ms
        .expect("matched response must have latency_ms");

    assert!(
        (latency - 100.0).abs() < 5.0,
        "expected latency ~100 ms, got {latency:.2} ms"
    );

    // The request TimedMessage (if produced) must NOT have latency set —
    // latency is only meaningful on the response side.
    if let Some(rt) = req_timed {
        assert!(
            rt.latency_ms.is_none(),
            "request TimedMessage must not carry latency_ms"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-002 — Throughput calculation
// ─────────────────────────────────────────────────────────────────────────────

/// AC-002: A `ThroughputWindow(10)` fed 100 messages spread uniformly across
/// ~1 second of simulated timestamps should report `messages_per_second() ≈ 10`
/// (100 messages / 10-second window).
///
/// We use *simulated* `Instant` offsets to avoid flakiness from real wall-clock
/// variation. The window is 10 s; we inject 100 messages with timestamps in the
/// range [now, now + 1 s] — all within the window — and expect 100/10 = 10 msg/s.
///
/// Traces to: BC-4.09.002 postcondition — throughput.
#[test]
fn test_BC_4_09_002_throughput_calculation() {
    let mut window = ThroughputWindow::new(10);
    let base = Instant::now();

    // Spread 100 messages across 1 second (10 ms apart), all well within the
    // 10-second window.
    for i in 0..100u64 {
        window.record(base + Duration::from_millis(i * 10));
    }

    let mps = window.messages_per_second();

    // Expected: 100 messages / 10 s window = 10.0 msg/s (±1.0 tolerance).
    assert!(
        (mps - 10.0).abs() < 1.0,
        "expected throughput ~10 msg/s, got {mps:.2}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-003 — Unmatched response has no latency
// ─────────────────────────────────────────────────────────────────────────────

/// AC-003: A response `MessageCaptured` whose JSON-RPC `id` has no prior
/// matching request must produce a `TimedMessage` with `latency_ms: None`.
///
/// Traces to: BC-4.09.002 — unmatched responses.
#[test]
fn test_BC_4_09_002_unmatched_response_no_latency() {
    let mut analyzer = TimingAnalyzer::new();

    // Feed a response with id=99 without any prior request with id=99.
    let response = make_response(MessageDirection::ServerToClient, 99, Instant::now());
    let timed = analyzer
        .process_message(&response)
        .expect("unmatched response must still produce a TimedMessage");

    assert!(
        timed.latency_ms.is_none(),
        "unmatched response must have latency_ms: None, got {:?}",
        timed.latency_ms
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-004 — Message ordering detection
// ─────────────────────────────────────────────────────────────────────────────

/// AC-004: When a *response* arrives before its matching *request* (out-of-order
/// delivery, E-PRO-009 batch reordering), the `TimedMessage` for the response
/// should have `reordered: true`.
///
/// Traces to: BC-4.09.002 — DI-006 ordering.
#[test]
fn test_BC_4_09_002_message_ordering_preserved() {
    let t0 = Instant::now();
    let t1 = t0 + Duration::from_millis(50);

    // Intentionally feed the response FIRST (out of order).
    let response = make_response(MessageDirection::ServerToClient, 42, t0);
    let request = make_request(MessageDirection::ClientToServer, 42, t1);

    let mut analyzer = TimingAnalyzer::new();

    let resp_timed = analyzer
        .process_message(&response)
        .expect("out-of-order response must produce a TimedMessage");

    // May or may not produce a TimedMessage for the late request — implementation
    // decides — but the earlier response must be flagged as reordered.
    let _req_timed = analyzer.process_message(&request);

    assert!(
        resp_timed.reordered,
        "response arriving before its request must have reordered: true"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// EC-001 — Notification gets no latency pairing
// ─────────────────────────────────────────────────────────────────────────────

/// EC-001: JSON-RPC notifications (no `id` field) are direction-only events.
/// They must not be paired for latency calculation; `latency_ms` must be `None`.
#[test]
fn test_notification_no_latency_pairing() {
    let mut analyzer = TimingAnalyzer::new();

    let notification = make_notification(MessageDirection::ServerToClient, Instant::now());
    let timed = analyzer
        .process_message(&notification)
        .expect("notification must produce a TimedMessage");

    assert!(
        timed.latency_ms.is_none(),
        "notification must have latency_ms: None, got {:?}",
        timed.latency_ms
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// EC-002 — Duplicate response ID flagged
// ─────────────────────────────────────────────────────────────────────────────

/// EC-002: If a second response arrives with the same JSON-RPC `id` as one
/// already processed, `process_message` must return `None` (drop the duplicate)
/// rather than re-using the same pending request slot.
#[test]
fn test_duplicate_response_id() {
    let t0 = Instant::now();
    let t1 = t0 + Duration::from_millis(50);
    let t2 = t1 + Duration::from_millis(50);

    let request = make_request(MessageDirection::ClientToServer, 7, t0);
    let response1 = make_response(MessageDirection::ServerToClient, 7, t1);
    let response2 = make_response(MessageDirection::ServerToClient, 7, t2);

    let mut analyzer = TimingAnalyzer::new();

    analyzer.process_message(&request);

    let first = analyzer.process_message(&response1);
    assert!(
        first.is_some(),
        "first response must produce a TimedMessage"
    );

    let second = analyzer.process_message(&response2);
    assert!(
        second.is_none(),
        "duplicate response with same id must be dropped (None)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Clean-state test — empty analyzer
// ─────────────────────────────────────────────────────────────────────────────

/// Sanity check: a freshly-constructed `TimingAnalyzer` and `ThroughputWindow`
/// have clean initial state with no panics.
#[test]
fn test_empty_analyzer() {
    // TimingAnalyzer — no messages processed, just verify construction.
    let _analyzer = TimingAnalyzer::new();

    // ThroughputWindow — no messages recorded; should return 0.0 without panic.
    let window = ThroughputWindow::new(10);
    let mps = window.messages_per_second();
    assert_eq!(mps, 0.0, "empty window must report 0.0 msg/s");
}
