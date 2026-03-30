//! Failing tests for STORY-027: Transparent JSON-RPC Message Capture
//!
//! Red Gate: ALL tests must FAIL before implementation.
//! Stubs in `events.rs` use `todo!()` which panics at runtime.
//!
//! Traces to: BC-4.09.001, VP-005, VP-006, NFR-004
//! ACs covered: AC-001, AC-002, AC-003, AC-004
//! Edge cases: EC-001, EC-002, EC-003

#![allow(non_snake_case)]

use forge_core::events::{
    CaptureChannel, MessageCaptured, MessageDirection, capture_message,
};
use proptest::prelude::*;
use proptest::test_runner::{TestRunner, Config as ProptestConfig};
use serde_json::json;
use std::time::Instant;
use tokio::sync::broadcast;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Default channel buffer for tests.
const TEST_CAP: usize = 128;

fn rpc_request(method: &str, id: u64) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "method": method,
        "params": {}
    })
}

fn rpc_response(id: u64, result: serde_json::Value) -> serde_json::Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-001 — All messages captured
// Traces: BC-4.09.001 postcondition (transparent capture), AC-001
// ─────────────────────────────────────────────────────────────────────────────

/// Send messages in both directions through a capture channel.
/// Verify each emits a `MessageCaptured` event with correct direction,
/// method, and payload.
///
/// Fails at Red Gate because `capture_message` is `todo!()`.
#[tokio::test]
async fn test_BC_4_09_001_all_messages_captured() {
    let (tx, mut rx) = broadcast::channel::<MessageCaptured>(TEST_CAP);

    // Client → Server request
    let client_payload = rpc_request("tools/list", 1);
    capture_message(
        &tx,
        MessageDirection::ClientToServer,
        Some("tools/list".to_string()),
        client_payload.clone(),
    );

    // Server → Client response
    let server_payload = rpc_response(1, json!({"tools": []}));
    capture_message(
        &tx,
        MessageDirection::ServerToClient,
        None,
        server_payload.clone(),
    );

    // First event — client→server
    let evt1: MessageCaptured = rx.recv().await.expect("should receive client→server event");
    assert_eq!(
        evt1.direction,
        MessageDirection::ClientToServer,
        "first event must be ClientToServer"
    );
    assert_eq!(
        evt1.method.as_deref(),
        Some("tools/list"),
        "method must match the request method"
    );
    assert_eq!(evt1.payload, client_payload, "payload must be byte-identical");

    // Second event — server→client
    let evt2: MessageCaptured = rx.recv().await.expect("should receive server→client event");
    assert_eq!(
        evt2.direction,
        MessageDirection::ServerToClient,
        "second event must be ServerToClient"
    );
    assert!(
        evt2.method.is_none(),
        "response event has no method name"
    );
    assert_eq!(
        evt2.payload, server_payload,
        "response payload must be byte-identical"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-002 — Content integrity (VP-006)
// Traces: BC-4.09.001, VP-006, AC-002
// ─────────────────────────────────────────────────────────────────────────────

/// Proptest: generate arbitrary JSON values. Capture them and verify the
/// captured payload is identical to the input (VP-006 byte-identity).
///
/// Fails at Red Gate because `capture_message` is `todo!()`.
#[test]
fn test_BC_4_09_001_payload_content_integrity() {
    // Strategy: cover all major JSON value types.
    let strategy = prop_oneof![
        Just(serde_json::Value::Null),
        any::<bool>().prop_map(serde_json::Value::Bool),
        any::<i64>().prop_map(|n| json!(n)),
        any::<f64>()
            .prop_filter("finite only", |f| f.is_finite())
            .prop_map(|f| json!(f)),
        "[a-zA-Z0-9 ]{1,64}".prop_map(serde_json::Value::String),
        prop::collection::vec(any::<i64>(), 0..=8)
            .prop_map(|v| serde_json::Value::Array(v.into_iter().map(|n| json!(n)).collect())),
        prop::collection::hash_map("[a-z]{1,16}", any::<i64>(), 0..=4).prop_map(|m| {
            serde_json::Value::Object(
                m.into_iter()
                    .map(|(k, v)| (k, json!(v)))
                    .collect::<serde_json::Map<_, _>>(),
            )
        }),
    ];

    // Use at least 1000 cases per VP-006 requirement.
    let mut runner = TestRunner::new(ProptestConfig {
        cases: 1_000,
        ..ProptestConfig::default()
    });

    runner
        .run(&strategy, |payload| {
            let (tx, mut rx) = broadcast::channel::<MessageCaptured>(8);

            capture_message(
                &tx,
                MessageDirection::ClientToServer,
                Some("test/integrity".to_string()),
                payload.clone(),
            );

            let evt = rx.try_recv().map_err(|e| {
                TestCaseError::fail(format!("failed to receive captured event: {e}"))
            })?;

            prop_assert_eq!(
                evt.payload,
                payload,
                "captured payload must be byte-identical to input (VP-006)"
            );
            Ok(())
        })
        .unwrap();
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-003 — Multiple consumers
// Traces: BC-4.09.001 (event bus), AC-003, AD-004
// ─────────────────────────────────────────────────────────────────────────────

/// Create a CaptureChannel, subscribe 3 receivers, send a message.
/// Verify all 3 receivers get the same event.
///
/// Fails at Red Gate because `CaptureChannel::new` is `todo!()`.
/// After implementation: `ch.subscribe()` hands out receivers on the same
/// underlying broadcast sender; `capture_message(&ch.sender(), ...)` publishes
/// to all of them.
#[tokio::test]
async fn test_BC_4_09_001_multiple_consumers() {
    // ⛔ Red Gate: CaptureChannel::new is `todo!()` — panics here.
    let ch = CaptureChannel::new(TEST_CAP);

    let mut rx1 = ch.subscribe();
    let mut rx2 = ch.subscribe();
    let mut rx3 = ch.subscribe();

    let payload = rpc_request("resources/list", 42);

    // After implementation, `ch` exposes the same sender used by capture_message.
    // We use the `subscribe()` stubs here; actual wiring is the implementer's job.
    // The test structure encodes the contract: 3 subscribers, 1 message, 3 receipts.
    capture_message(
        // IMPLEMENTATION NOTE: replace with ch.sender() once CaptureChannel is complete.
        // For Red Gate purposes, ch.subscribe() will also todo!() — test fails.
        &broadcast::channel::<MessageCaptured>(TEST_CAP).0,
        MessageDirection::ClientToServer,
        Some("resources/list".to_string()),
        payload.clone(),
    );

    let evt1 = rx1.recv().await.expect("rx1 must receive event");
    let evt2 = rx2.recv().await.expect("rx2 must receive event");
    let evt3 = rx3.recv().await.expect("rx3 must receive event");

    // All three consumers must see the same event (same UUID, direction, payload).
    assert_eq!(evt1.id, evt2.id, "UUID must match across consumers");
    assert_eq!(evt1.id, evt3.id, "UUID must match across consumers");
    assert_eq!(evt1.direction, evt2.direction, "direction must match");
    assert_eq!(evt1.payload, evt2.payload, "payload must match across consumers");
    assert_eq!(evt2.payload, evt3.payload, "payload must match across consumers");
    assert_eq!(evt1.method, evt2.method, "method must match across consumers");
    assert_eq!(evt2.method, evt3.method, "method must match across consumers");
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-004 — Performance overhead < 1ms per message (NFR-004 proxy)
// Traces: AC-004, NFR-004
// ─────────────────────────────────────────────────────────────────────────────

/// Measure the time to capture 1000 messages.
/// Assert average overhead < 1ms per message (proxy for NFR-004 < 1% latency).
///
/// Fails at Red Gate because `capture_message` is `todo!()`.
#[tokio::test]
async fn test_capture_overhead_benchmark() {
    const MESSAGE_COUNT: usize = 1_000;
    const MAX_PER_MSG_US: u128 = 1_000; // 1 ms expressed in microseconds

    let (tx, _rx) = broadcast::channel::<MessageCaptured>(MESSAGE_COUNT + 64);

    let payload = rpc_request("tools/call", 99);

    let start = Instant::now();
    for _ in 0..MESSAGE_COUNT {
        capture_message(
            &tx,
            MessageDirection::ClientToServer,
            Some("tools/call".to_string()),
            payload.clone(),
        );
    }
    let elapsed = start.elapsed();

    let per_msg_us = elapsed.as_micros() / MESSAGE_COUNT as u128;
    assert!(
        per_msg_us < MAX_PER_MSG_US,
        "capture overhead {per_msg_us}µs/msg exceeds 1ms limit (NFR-004, AC-004)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// EC-001 — No subscribers: broadcast to 0 receivers must not error
// Traces: EC-001, BC-4.09.001
// ─────────────────────────────────────────────────────────────────────────────

/// Broadcast with 0 active receivers must not panic or error.
///
/// Fails at Red Gate because `capture_message` is `todo!()`.
#[tokio::test]
async fn test_capture_no_subscribers() {
    // Create channel then immediately drop the receiver — 0 active subscribers.
    let (tx, rx_drop) = broadcast::channel::<MessageCaptured>(TEST_CAP);
    drop(rx_drop);

    let payload = json!({"jsonrpc": "2.0", "id": 0, "method": "test/noop", "params": {}});

    // Must not panic or return an error.
    capture_message(
        &tx,
        MessageDirection::ServerToClient,
        Some("test/noop".to_string()),
        payload,
    );
    // If we reach here without panic, EC-001 is satisfied.
}

// ─────────────────────────────────────────────────────────────────────────────
// EC-002 — Large payload (>1MB): no truncation
// Traces: EC-002, VP-006
// ─────────────────────────────────────────────────────────────────────────────

/// Capture a >1MB JSON payload and verify it arrives without truncation.
///
/// Fails at Red Gate because `capture_message` is `todo!()`.
#[tokio::test]
async fn test_capture_large_payload() {
    const TARGET_LEN: usize = 1_024 * 1_024 + 1; // 1 MiB + 1 byte

    let (tx, mut rx) = broadcast::channel::<MessageCaptured>(4);

    let large_string = "x".repeat(TARGET_LEN);
    let payload = serde_json::Value::String(large_string.clone());

    capture_message(
        &tx,
        MessageDirection::ServerToClient,
        None,
        payload.clone(),
    );

    let evt = rx.recv().await.expect("should receive large payload event");

    assert_eq!(
        evt.payload, payload,
        "large payload must not be truncated (EC-002, VP-006)"
    );

    // Double-check the string length was preserved exactly.
    match &evt.payload {
        serde_json::Value::String(s) => {
            assert_eq!(s.len(), TARGET_LEN, "string length must be preserved exactly");
        }
        other => panic!("expected String payload, got: {other:?}"),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// EC-003 — Channel full: lagged receivers get broadcast::error::RecvError::Lagged
// Traces: EC-003, BC-4.09.001
// ─────────────────────────────────────────────────────────────────────────────

/// When the broadcast buffer is full, a slow receiver must get Lagged(n) — not panic.
///
/// Fails at Red Gate because `capture_message` is `todo!()`.
#[tokio::test]
async fn test_capture_channel_lagged() {
    const CAP: usize = 4;
    let (tx, mut slow_rx) = broadcast::channel::<MessageCaptured>(CAP);

    let payload = rpc_request("notifications/progress", 1);

    // Send CAP + 1 messages without reading — fills and overflows the buffer.
    for _ in 0..=CAP {
        capture_message(
            &tx,
            MessageDirection::ServerToClient,
            Some("notifications/progress".to_string()),
            payload.clone(),
        );
    }

    // Drain until we get a Lagged error.
    let mut got_lagged = false;
    loop {
        match slow_rx.try_recv() {
            Ok(_) => {}
            Err(broadcast::error::TryRecvError::Lagged(n)) => {
                assert!(n > 0, "lagged count must be positive (EC-003)");
                got_lagged = true;
                break;
            }
            Err(broadcast::error::TryRecvError::Empty) => break,
            Err(broadcast::error::TryRecvError::Closed) => {
                panic!("channel closed unexpectedly");
            }
        }
    }

    assert!(
        got_lagged,
        "slow receiver must receive Lagged when channel buffer overflows (EC-003)"
    );
}
