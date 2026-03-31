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
use forge_traffic::{ThroughputWindow, TimedMessage, TimingAnalyzer};
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
/// delivery, E-PRO-009 batch reordering), the pair is flagged as reordered.
///
/// Per PRF-001 fix: the `reordered` flag is placed on the **request**
/// `TimedMessage` (emitted when the late request arrives), NOT on the
/// response.  The response is emitted with `reordered: false` because at
/// emission time we cannot distinguish out-of-order from truly unmatched.
/// The request then arrives and sees its response was already consumed —
/// it is emitted with `reordered: true`.
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

    // The response arrives first — it is emitted with reordered: false because
    // we cannot yet know whether it is truly unmatched or out-of-order.
    assert!(
        !resp_timed.reordered,
        "response emitted before its request arrives must have reordered: false \
         (cannot distinguish out-of-order from unmatched at emission time)"
    );

    // When the late request arrives, it discovers its response was already
    // consumed — the REQUEST TimedMessage carries reordered: true.
    let req_timed = analyzer
        .process_message(&request)
        .expect("late request must still produce a TimedMessage");

    assert!(
        req_timed.reordered,
        "late request (whose response already arrived) must have reordered: true"
    );
}

/// AC-004 extended: test the full batch reordering scenario.
///
/// Two requests are in flight (id=10, id=20). Responses arrive in reverse
/// order: response for id=20 arrives before response for id=10.  This is
/// the canonical E-PRO-009 batch reordering case — both responses are matched
/// (latency computed), but the one that arrived first for id=20 is out-of-order
/// relative to the request submission order.
///
/// For this scenario responses arrive AFTER both requests, so neither is an
/// orphan — `reordered` is `false` for both responses.
/// (Ordering relative to sibling responses is tracked separately if needed.)
#[test]
fn test_timing_analyzer_out_of_order() {
    let t0 = Instant::now();
    let t1 = t0 + Duration::from_millis(10);
    let t2 = t1 + Duration::from_millis(50); // response for id=20 arrives first
    let t3 = t2 + Duration::from_millis(30); // response for id=10 arrives second

    let req10 = make_request(MessageDirection::ClientToServer, 10, t0);
    let req20 = make_request(MessageDirection::ClientToServer, 20, t1);
    let resp20 = make_response(MessageDirection::ServerToClient, 20, t2); // out-of-order
    let resp10 = make_response(MessageDirection::ServerToClient, 10, t3); // matches req10

    let mut analyzer = TimingAnalyzer::new();

    // Feed both requests first (normal batch submission).
    let req10_timed = analyzer
        .process_message(&req10)
        .expect("request must produce TimedMessage");
    let req20_timed = analyzer
        .process_message(&req20)
        .expect("request must produce TimedMessage");

    // Neither request is reordered on submission.
    assert!(!req10_timed.reordered, "req10 should not be reordered");
    assert!(!req20_timed.reordered, "req20 should not be reordered");

    // Response for id=20 arrives before response for id=10 — but BOTH requests
    // are already pending, so neither response is an orphan.
    let resp20_timed = analyzer
        .process_message(&resp20)
        .expect("resp20 must produce TimedMessage");
    let resp10_timed = analyzer
        .process_message(&resp10)
        .expect("resp10 must produce TimedMessage");

    // Both responses are matched — latency must be populated.
    assert!(
        resp20_timed.latency_ms.is_some(),
        "resp20 must have latency (matched)"
    );
    assert!(
        resp10_timed.latency_ms.is_some(),
        "resp10 must have latency (matched)"
    );

    // Neither response is flagged reordered (both requests were already pending).
    assert!(!resp20_timed.reordered, "resp20 should not be reordered");
    assert!(!resp10_timed.reordered, "resp10 should not be reordered");

    // Verify approximate latency values.
    let latency20 = resp20_timed.latency_ms.unwrap();
    let latency10 = resp10_timed.latency_ms.unwrap();
    // resp20 latency = t2 - t1 ≈ 50 ms
    assert!(
        (latency20 - 50.0).abs() < 5.0,
        "resp20 latency expected ~50 ms, got {latency20:.2}"
    );
    // resp10 latency = t3 - t0 ≈ 90 ms
    assert!(
        (latency10 - 90.0).abs() < 5.0,
        "resp10 latency expected ~90 ms, got {latency10:.2}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// PRF-001 — Unmatched responses must NOT be flagged reordered
// ─────────────────────────────────────────────────────────────────────────────

/// PRF-001(b): A response whose JSON-RPC `id` was never requested must have
/// `reordered: false`.  This distinguishes it from the AC-004 out-of-order
/// case (where the request arrives late) and correctly represents that the
/// response is genuinely unmatched rather than reordered.
#[test]
fn test_unmatched_response_not_flagged_reordered() {
    let mut analyzer = TimingAnalyzer::new();

    // Feed a response with id=99 without any prior request.
    // No request will ever arrive for id=99 in this test.
    let response = make_response(MessageDirection::ServerToClient, 99, Instant::now());
    let timed = analyzer
        .process_message(&response)
        .expect("unmatched response must still produce a TimedMessage");

    assert!(
        timed.latency_ms.is_none(),
        "unmatched response must have latency_ms: None"
    );
    assert!(
        !timed.reordered,
        "unmatched response must have reordered: false (PRF-001)"
    );
}

/// PRF-001(a/b) combined: verify that out-of-order and unmatched responses
/// are correctly distinguished by the `reordered` flag.
///
/// - id=55: response arrives before request → REQUEST gets `reordered: true`
/// - id=66: response arrives, request never comes → response keeps `reordered: false`
#[test]
fn test_reordered_vs_unmatched_distinction() {
    let t0 = Instant::now();
    let t1 = t0 + Duration::from_millis(20);

    let mut analyzer = TimingAnalyzer::new();

    // id=55: out-of-order — response before request.
    let resp55 = make_response(MessageDirection::ServerToClient, 55, t0);
    let req55 = make_request(MessageDirection::ClientToServer, 55, t1);

    // id=66: truly unmatched — response with no request ever.
    let resp66 = make_response(MessageDirection::ServerToClient, 66, t0);

    let resp55_timed = analyzer
        .process_message(&resp55)
        .expect("resp55 must produce TimedMessage");
    let resp66_timed = analyzer
        .process_message(&resp66)
        .expect("resp66 must produce TimedMessage");

    // Both responses are emitted with reordered: false (we don't know yet).
    assert!(
        !resp55_timed.reordered,
        "resp55 initially emitted with reordered: false"
    );
    assert!(
        !resp66_timed.reordered,
        "resp66 emitted with reordered: false (truly unmatched)"
    );

    // Late request for id=55 arrives — it should carry reordered: true.
    let req55_timed = analyzer
        .process_message(&req55)
        .expect("late req55 must produce TimedMessage");
    assert!(
        req55_timed.reordered,
        "req55 arriving after its response must have reordered: true"
    );

    // No request for id=66 ever arrives. resp66_timed.reordered was already
    // correctly false — verify the analyzer state is consistent (no panic).
    // (Nothing else to assert here; the correctness is in resp66_timed above.)
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

/// EC-002 extended (PRF-001 interaction): orphan response followed by a
/// duplicate orphan response.  The first response (no pending request) must
/// be returned; the second (same id) must be dropped even though no request
/// was ever seen.
#[test]
fn test_duplicate_orphan_response_dropped() {
    let t0 = Instant::now();
    let t1 = t0 + Duration::from_millis(30);

    let resp1 = make_response(MessageDirection::ServerToClient, 88, t0);
    let resp2 = make_response(MessageDirection::ServerToClient, 88, t1);

    let mut analyzer = TimingAnalyzer::new();

    // No request for id=88 — both responses are orphans.
    let first = analyzer
        .process_message(&resp1)
        .expect("first orphan response must produce TimedMessage");
    assert!(
        !first.reordered,
        "orphan response must not be flagged reordered"
    );
    assert!(
        first.latency_ms.is_none(),
        "orphan response must have no latency"
    );

    let second = analyzer.process_message(&resp2);
    assert!(
        second.is_none(),
        "duplicate orphan response with same id must be dropped (None)"
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
