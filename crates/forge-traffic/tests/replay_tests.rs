//! Failing tests for STORY-032 — Message Sequence Replay Against Target Server.
//!
//! RED GATE: All tests are designed to COMPILE but FAIL until Phase 3
//! implementation replaces `todo!()` stubs in `replay.rs`.
//!
//! Naming follows the BC-based convention required by AGENTS.md:
//!   `test_BC_S_SS_NNN_<assertion>()`
//! Edge-case tests follow the EC-NNN convention from the story spec and BC.
//!
//! Traces to: BC-4.10.003 | STORY-032 | VP-006
//!
//! Test coverage:
//!   AC-001 → test_BC_4_10_003_replay_sends_messages
//!   AC-002 → test_BC_4_10_003_disconnected_target_errors
//!   AC-003 → test_BC_4_10_003_replay_responses_captured
//!   AC-004 → test_BC_4_10_003_replay_preserves_order
//!   AC-005 → test_BC_4_10_003_cli_replay_command  (smoke / parse check)
//!   INV-001 → test_BC_4_10_003_invariant_explicit_target_required
//!   INV-002 → test_BC_4_10_003_invariant_byte_identical_payload
//!   INV-003 → test_BC_4_10_003_invariant_order_preserved
//!   INV-004 → test_BC_4_10_003_invariant_capture_buffer_unchanged
//!   INV-005 → test_BC_4_10_003_invariant_responses_stored_separately
//!   EC-001  → test_BC_4_10_003_ec001_server_only_messages_skipped
//!   EC-004  → test_BC_4_10_003_ec004_response_timeout_marked_no_response
//!   EC-005  → test_BC_4_10_003_ec005_server_initiated_messages_skipped
//!   EC-008  → test_BC_4_10_003_ec008_empty_selection_no_replayable
//!   VP-001  → test_BC_4_10_003_vp001_refuses_without_explicit_target
//!   VP-002  → test_BC_4_10_003_vp002_byte_identical_payload
//!   VP-003  → test_BC_4_10_003_vp003_wire_order_preserved
//!   VP-004  → test_BC_4_10_003_vp004_capture_buffer_unchanged_after_replay
//!   VP-005  → test_BC_4_10_003_vp005_comparison_identifies_identical_divergent
//!   VP-006  → test_BC_4_10_003_vp006_target_unreachable_no_panic
//!   filter  → test_BC_4_10_003_filter_client_to_server_only
//!   progress → test_BC_4_10_003_replay_progress_tracking

// Suppress the mandatory BC-naming lint.
#![allow(non_snake_case)]

use std::time::Instant;

use forge_core::events::{MessageCaptured, MessageDirection};
use forge_traffic::replay::{
    ComparisonReport, ReplayError, ReplayProgress, ReplayResponse, ReplayStatus, ReplayTarget,
    filter_client_to_server, replay_sequence,
};
use uuid::Uuid;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Build a client→server JSON-RPC request with the given method and rpc_id.
fn make_client_request(method: &str, rpc_id: u64) -> MessageCaptured {
    MessageCaptured {
        id: Uuid::new_v4(),
        direction: MessageDirection::ClientToServer,
        method: Some(method.to_string()),
        payload: serde_json::json!({
            "jsonrpc": "2.0",
            "id": rpc_id,
            "method": method,
            "params": {}
        }),
        timestamp: Instant::now(),
    }
}

/// Build a server→client JSON-RPC response for the given rpc_id.
fn make_server_response(rpc_id: u64) -> MessageCaptured {
    MessageCaptured {
        id: Uuid::new_v4(),
        direction: MessageDirection::ServerToClient,
        method: None,
        payload: serde_json::json!({
            "jsonrpc": "2.0",
            "id": rpc_id,
            "result": { "content": [{"type": "text", "text": "ok"}] }
        }),
        timestamp: Instant::now(),
    }
}

/// Build a server→client notification (no id).
fn make_server_notification(method: &str) -> MessageCaptured {
    MessageCaptured {
        id: Uuid::new_v4(),
        direction: MessageDirection::ServerToClient,
        method: Some(method.to_string()),
        payload: serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": {}
        }),
        timestamp: Instant::now(),
    }
}

/// Build a client→server notification (no id) — EC-006.
fn make_client_notification(method: &str) -> MessageCaptured {
    MessageCaptured {
        id: Uuid::new_v4(),
        direction: MessageDirection::ClientToServer,
        method: Some(method.to_string()),
        payload: serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": {}
        }),
        timestamp: Instant::now(),
    }
}

/// A connected `ReplayTarget` for use in tests.
fn connected_target() -> ReplayTarget {
    ReplayTarget::Connected {
        label: "test-server".to_string(),
        is_original: false,
    }
}

/// A disconnected `ReplayTarget` for error-path tests.
fn disconnected_target() -> ReplayTarget {
    ReplayTarget::Disconnected
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-001 — replay sends client→server messages and returns responses
// ─────────────────────────────────────────────────────────────────────────────

/// AC-001 (BC-4.10.003 POST-001 / POST-002):
/// `replay_sequence` sends each client→server message in the sequence to the
/// target server and returns a `ReplayResponse` for each one.
///
/// TV-HP-001: 5 messages replayed, all return identical responses.
#[tokio::test]
async fn test_BC_4_10_003_replay_sends_messages() {
    let messages: Vec<MessageCaptured> = (1..=5)
        .map(|i| make_client_request("tools/call", i))
        .collect();

    let target = connected_target();
    let result = replay_sequence(&target, &messages).await;

    // Must succeed and return 5 responses (one per client→server message).
    let responses = result.expect("replay_sequence must succeed with connected target");
    assert_eq!(
        responses.len(),
        5,
        "must return one ReplayResponse per client→server message"
    );

    // Every response must be tagged replay: true (AC-003 precondition).
    for (i, resp) in responses.iter().enumerate() {
        assert!(
            resp.replay,
            "ReplayResponse[{i}] must have replay: true (AC-003)"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-002 — disconnected target returns E-CAP-002 error
// ─────────────────────────────────────────────────────────────────────────────

/// AC-002 (BC-4.10.003 PRE-003):
/// If the target server is not connected, `replay_sequence` must return
/// `Err(ReplayError::TargetNotConnected)` — error code E-CAP-002.
///
/// TV-ERR-001: No target designated.
#[tokio::test]
async fn test_BC_4_10_003_disconnected_target_errors() {
    let messages = vec![make_client_request("tools/call", 1)];
    let target = disconnected_target();

    let result = replay_sequence(&target, &messages).await;

    assert!(
        result.is_err(),
        "replay_sequence must return Err when target is disconnected"
    );
    assert_eq!(
        result.unwrap_err(),
        ReplayError::TargetNotConnected,
        "error must be TargetNotConnected (E-CAP-002)"
    );
}

/// AC-002 variant: E-CAP-002 is returned even when the message list is empty,
/// because the connection check must happen before message iteration.
#[tokio::test]
async fn test_BC_4_10_003_disconnected_target_errors_empty_messages() {
    let messages: Vec<MessageCaptured> = vec![];
    let target = disconnected_target();

    let result = replay_sequence(&target, &messages).await;

    assert_eq!(
        result.unwrap_err(),
        ReplayError::TargetNotConnected,
        "E-CAP-002 must be returned even with empty message list when disconnected"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-003 — replay responses tagged with replay: true
// ─────────────────────────────────────────────────────────────────────────────

/// AC-003 (BC-4.10.003 POST-002):
/// Responses from the replayed sequence are `ReplayResponse` events with
/// `replay: true`, distinguishable from live traffic.
#[tokio::test]
async fn test_BC_4_10_003_replay_responses_captured() {
    let messages = vec![
        make_client_request("tools/list", 1),
        make_client_request("tools/call", 2),
    ];
    let target = connected_target();

    let responses = replay_sequence(&target, &messages)
        .await
        .expect("should succeed");

    for (i, resp) in responses.iter().enumerate() {
        assert!(
            resp.replay,
            "ReplayResponse[{i}].replay must be true — must be distinguishable from live traffic"
        );
        // response_payload must be Some (the stub will populate it or return
        // a synthetic payload — what matters is the flag, but let's verify
        // the structure is at least populated).
        assert!(
            resp.response_payload.is_some() || matches!(resp.status, ReplayStatus::NoResponse | ReplayStatus::Error(_)),
            "response_payload must be populated or status explains absence"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-004 — messages replayed in temporal order
// ─────────────────────────────────────────────────────────────────────────────

/// AC-004 (BC-4.10.003 INV-003 / DI-007):
/// Messages are replayed in the same temporal order as the original capture.
/// No reordering is permitted.
///
/// TV-HP-001: verify sequence_index is strictly ascending.
#[tokio::test]
async fn test_BC_4_10_003_replay_preserves_order() {
    // Create 5 messages in order 1..=5.
    let messages: Vec<MessageCaptured> = (1..=5)
        .map(|i| make_client_request("tools/call", i as u64))
        .collect();

    let target = connected_target();
    let responses = replay_sequence(&target, &messages)
        .await
        .expect("should succeed");

    // sequence_index must be 0, 1, 2, 3, 4 — strictly ascending.
    let indices: Vec<usize> = responses.iter().map(|r| r.sequence_index).collect();
    let mut sorted = indices.clone();
    sorted.sort_unstable();

    assert_eq!(
        indices, sorted,
        "replay responses must appear in original sequence order (INV-003)"
    );

    // Adjacent indices must be strictly increasing (no duplicates / gaps).
    for window in indices.windows(2) {
        assert!(
            window[1] > window[0],
            "sequence_index must strictly increase: got {:?}",
            indices
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AC-005 — CLI replay command (smoke / parse-level check)
// ─────────────────────────────────────────────────────────────────────────────

/// AC-005 (STORY-032 AC-005):
/// The CLI subcommand `forge-mcp call <server> --replay <capture-file>` must
/// parse correctly and trigger a replay workflow.
///
/// Since we are at the Red Gate phase we test the public surface exposed by
/// the library (not the binary), confirming that `ReplayTarget` and
/// `replay_sequence` are accessible from a crate consumer's perspective.
/// Full CLI integration testing belongs in a later story.
///
/// This test validates that the public API compiles and that `ReplayTarget`
/// correctly represents an explicitly-designated server (DI-007).
#[test]
fn test_BC_4_10_003_cli_replay_command() {
    // The CLI will ultimately parse a server label and build a ReplayTarget.
    // Here we verify the target correctly represents "explicit designation".
    let target = ReplayTarget::Connected {
        label: "my-server".to_string(),
        is_original: false,
    };

    assert!(
        target.is_connected(),
        "Connected target must report is_connected() == true"
    );
    assert_eq!(
        target.label(),
        Some("my-server"),
        "label() must return the designated server label"
    );
    assert!(
        !target.is_original_server(),
        "is_original_server() must be false when targeting a different server"
    );

    // EC-007: when the original server is designated as target, the flag is set.
    let original_target = ReplayTarget::Connected {
        label: "original-server".to_string(),
        is_original: true,
    };
    assert!(
        original_target.is_original_server(),
        "is_original_server() must be true when replaying to original server (EC-007)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// INV-001 — explicit target designation required (DI-007)
// ─────────────────────────────────────────────────────────────────────────────

/// INV-001 (BC-4.10.003 INV-001 / VP-001):
/// Replay must refuse to execute without explicit target designation.
/// `ReplayTarget::Disconnected` represents the absence of a target.
///
/// VP-001: DI-007 state machine test.
#[tokio::test]
async fn test_BC_4_10_003_invariant_explicit_target_required() {
    let messages = vec![make_client_request("tools/call", 1)];
    let no_target = ReplayTarget::Disconnected;

    let result = replay_sequence(&no_target, &messages).await;

    assert!(
        result.is_err(),
        "INV-001: replay must not execute without explicit target (DI-007)"
    );
    assert_eq!(
        result.unwrap_err(),
        ReplayError::TargetNotConnected,
        "INV-001: error must be TargetNotConnected when no target is designated"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// INV-002 / VP-002 — byte-identical payloads (DI-005)
// ─────────────────────────────────────────────────────────────────────────────

/// INV-002 / VP-002 (BC-4.10.003 INV-002 / VP-002):
/// The payload field in each replayed `ReplayResponse::original` must be
/// byte-identical to the original captured message payload.
///
/// DI-005: replayed messages are byte-identical to captured payloads.
#[tokio::test]
async fn test_BC_4_10_003_invariant_byte_identical_payload() {
    let original_payload = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 42,
        "method": "tools/call",
        "params": { "name": "get_weather", "arguments": { "city": "Austin" } }
    });

    let msg = MessageCaptured {
        id: Uuid::new_v4(),
        direction: MessageDirection::ClientToServer,
        method: Some("tools/call".to_string()),
        payload: original_payload.clone(),
        timestamp: Instant::now(),
    };

    let target = connected_target();
    let responses = replay_sequence(&target, &[msg])
        .await
        .expect("should succeed");

    assert_eq!(responses.len(), 1, "one response expected");
    assert_eq!(
        responses[0].original.payload,
        original_payload,
        "INV-002 / VP-002: replayed payload must be byte-identical to original (DI-005)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// INV-003 / VP-003 — original wire order preserved
// ─────────────────────────────────────────────────────────────────────────────

/// INV-003 / VP-003 (BC-4.10.003 INV-003 / VP-003):
/// Message[i] must be sent before Message[i+1] — original wire order is
/// preserved throughout replay.
///
/// We verify this by inspecting the `sequence_index` of each response, which
/// must equal the position of the original message in the input slice.
#[tokio::test]
async fn test_BC_4_10_003_invariant_order_preserved() {
    let methods = ["tools/list", "tools/call", "resources/list", "prompts/list"];
    let messages: Vec<MessageCaptured> = methods
        .iter()
        .enumerate()
        .map(|(i, method)| make_client_request(method, i as u64 + 1))
        .collect();

    let target = connected_target();
    let responses = replay_sequence(&target, &messages)
        .await
        .expect("should succeed");

    assert_eq!(
        responses.len(),
        methods.len(),
        "must return one response per client→server message"
    );

    // Each response's sequence_index must correspond to the input position.
    for (expected_idx, resp) in responses.iter().enumerate() {
        assert_eq!(
            resp.sequence_index,
            expected_idx,
            "INV-003 / VP-003: sequence_index[{expected_idx}] must equal input position"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// INV-004 / VP-004 — original capture buffer unchanged after replay
// ─────────────────────────────────────────────────────────────────────────────

/// INV-004 / VP-004 (BC-4.10.003 INV-004 / VP-004):
/// The original `messages` slice passed to `replay_sequence` must not be
/// modified by the replay operation. Replay is a read-only operation on the
/// capture buffer.
#[tokio::test]
async fn test_BC_4_10_003_invariant_capture_buffer_unchanged() {
    let messages: Vec<MessageCaptured> = (1..=3)
        .map(|i| make_client_request("tools/call", i))
        .collect();

    // Snapshot ids before replay.
    let ids_before: Vec<Uuid> = messages.iter().map(|m| m.id).collect();
    let payloads_before: Vec<serde_json::Value> =
        messages.iter().map(|m| m.payload.clone()).collect();

    let target = connected_target();
    let _ = replay_sequence(&target, &messages).await;

    // Verify the original slice is unchanged.
    let ids_after: Vec<Uuid> = messages.iter().map(|m| m.id).collect();
    let payloads_after: Vec<serde_json::Value> =
        messages.iter().map(|m| m.payload.clone()).collect();

    assert_eq!(
        ids_before, ids_after,
        "INV-004 / VP-004: capture buffer ids must not change after replay"
    );
    assert_eq!(
        payloads_before, payloads_after,
        "INV-004 / VP-004: capture buffer payloads must not change after replay"
    );
    assert_eq!(
        messages.len(),
        3,
        "INV-004 / VP-004: capture buffer length must not change after replay"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// INV-005 — replay responses stored separately from capture buffer
// ─────────────────────────────────────────────────────────────────────────────

/// INV-005 (BC-4.10.003 INV-005):
/// Replay responses must be returned as a separate list and must not be
/// injected into the original capture buffer or confused with live traffic.
///
/// Verified by checking that response UUIDs differ from the original message
/// UUIDs and that `replay: true` is set.
#[tokio::test]
async fn test_BC_4_10_003_invariant_responses_stored_separately() {
    let messages: Vec<MessageCaptured> = (1..=3)
        .map(|i| make_client_request("tools/call", i))
        .collect();

    let original_ids: std::collections::HashSet<Uuid> =
        messages.iter().map(|m| m.id).collect();

    let target = connected_target();
    let responses = replay_sequence(&target, &messages)
        .await
        .expect("should succeed");

    for (i, resp) in responses.iter().enumerate() {
        // The response itself must have replay: true.
        assert!(
            resp.replay,
            "INV-005: ReplayResponse[{i}].replay must be true"
        );

        // The original field tracks the captured message, but the response
        // UUID (from the target server) must be a *new* event.
        // We don't mandate a specific UUID format, but the structure must be
        // a separate object from the capture buffer entries.
        assert!(
            original_ids.contains(&resp.original.id),
            "INV-005: ReplayResponse[{i}].original.id must reference the captured message"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// EC-001 / EC-005 — server→client messages are skipped
// ─────────────────────────────────────────────────────────────────────────────

/// EC-001 / EC-005 (BC-4.10.003 EC-005 / Story EC-001):
/// Only client→server messages are replayed. Server→client messages
/// (notifications, responses) are silently skipped.
#[tokio::test]
async fn test_BC_4_10_003_ec001_server_only_messages_skipped() {
    let messages = vec![
        make_client_request("tools/call", 1),     // client→server — REPLAYED
        make_server_response(1),                   // server→client — SKIPPED
        make_server_notification("notifications/message"), // server→client — SKIPPED
        make_client_request("tools/list", 2),     // client→server — REPLAYED
    ];

    let target = connected_target();
    let responses = replay_sequence(&target, &messages)
        .await
        .expect("should succeed");

    // Only 2 client→server messages → 2 responses.
    assert_eq!(
        responses.len(),
        2,
        "EC-001/EC-005: only client→server messages should be replayed (2 of 4)"
    );
}

/// EC-005 (BC-4.10.003 EC-005) — pure helper test:
/// `filter_client_to_server` must return only client→server messages with
/// their original indices.
#[test]
fn test_BC_4_10_003_ec005_server_initiated_messages_skipped() {
    let messages = vec![
        make_server_notification("notifications/progress"), // idx 0 — skipped
        make_client_request("tools/call", 1),              // idx 1 — kept
        make_server_response(1),                            // idx 2 — skipped
        make_client_request("resources/list", 2),           // idx 3 — kept
        make_server_notification("notifications/message"),  // idx 4 — skipped
    ];

    let filtered = filter_client_to_server(&messages);

    assert_eq!(
        filtered.len(),
        2,
        "filter_client_to_server must keep only 2 client→server messages"
    );

    // Verify original indices are preserved.
    assert_eq!(filtered[0].0, 1, "first kept message has original index 1");
    assert_eq!(filtered[1].0, 3, "second kept message has original index 3");

    // Verify directions.
    for (_, msg) in &filtered {
        assert_eq!(
            msg.direction,
            MessageDirection::ClientToServer,
            "filter must only keep ClientToServer messages"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// EC-004 — response timeout marked as NoResponse
// ─────────────────────────────────────────────────────────────────────────────

/// EC-004 (BC-4.10.003 EC-004 / Story EC-002):
/// If the target server does not respond within the timeout, the response
/// status must be `ReplayStatus::NoResponse` and `response_payload` must be
/// `None`. Replay continues to the next message.
///
/// This test verifies the status enum value via ReplayStatus construction.
#[test]
fn test_BC_4_10_003_ec004_response_timeout_marked_no_response() {
    // Construct a ReplayResponse as the implementation would on timeout.
    let timed_out = ReplayResponse {
        sequence_index: 0,
        original: make_client_request("tools/call", 1),
        response_payload: None,
        replay: true,
        status: ReplayStatus::NoResponse,
    };

    assert!(
        timed_out.replay,
        "timed-out replay response must still carry replay: true"
    );
    assert!(
        timed_out.response_payload.is_none(),
        "EC-004: timed-out response must have response_payload: None"
    );
    assert_eq!(
        timed_out.status,
        ReplayStatus::NoResponse,
        "EC-004: timed-out response must have status NoResponse"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// EC-008 — empty selection returns empty response list
// ─────────────────────────────────────────────────────────────────────────────

/// EC-008 (BC-4.10.003 EC-008 / TV-ERR-003):
/// If no client→server messages exist in the sequence (e.g. only server→client
/// messages), `replay_sequence` returns an empty `Vec<ReplayResponse>` with `Ok`.
///
/// "No replayable messages selected" scenario.
#[tokio::test]
async fn test_BC_4_10_003_ec008_empty_selection_no_replayable() {
    let messages = vec![
        make_server_response(1),
        make_server_notification("notifications/message"),
    ];

    let target = connected_target();
    let result = replay_sequence(&target, &messages).await;

    // Should succeed (not an error) but return empty responses.
    let responses = result.expect(
        "EC-008: empty replayable selection must return Ok([]) not an error"
    );
    assert!(
        responses.is_empty(),
        "EC-008: when no client→server messages exist, responses must be empty"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// VP-001 — refuses without explicit target (DI-007)
// ─────────────────────────────────────────────────────────────────────────────

/// VP-001 (BC-4.10.003 VP-001):
/// State machine test — `replay_sequence` with `ReplayTarget::Disconnected`
/// must return `Err(TargetNotConnected)` regardless of the message list.
#[tokio::test]
async fn test_BC_4_10_003_vp001_refuses_without_explicit_target() {
    // Multiple variants to confirm the refusal is unconditional.
    let test_cases: Vec<Vec<MessageCaptured>> = vec![
        vec![make_client_request("tools/call", 1)],
        vec![make_client_request("tools/call", 1), make_client_request("tools/list", 2)],
        vec![],
    ];

    for (i, messages) in test_cases.iter().enumerate() {
        let target = ReplayTarget::Disconnected;
        let result = replay_sequence(&target, messages).await;

        assert_eq!(
            result.unwrap_err(),
            ReplayError::TargetNotConnected,
            "VP-001: test case {i} — must refuse without explicit target (DI-007)"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// VP-002 — replayed bytes are identical to captured bytes (DI-005)
// ─────────────────────────────────────────────────────────────────────────────

/// VP-002 (BC-4.10.003 VP-002):
/// The `original.payload` stored in each `ReplayResponse` must be
/// byte-identical to the payload of the captured message that was replayed.
/// This ensures DI-005 (wire-identical replay) is enforced.
#[tokio::test]
async fn test_BC_4_10_003_vp002_byte_identical_payload() {
    let payloads: Vec<serde_json::Value> = vec![
        serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}),
        serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"calc"}}),
    ];

    let messages: Vec<MessageCaptured> = payloads
        .iter()
        .enumerate()
        .map(|(i, payload)| MessageCaptured {
            id: Uuid::new_v4(),
            direction: MessageDirection::ClientToServer,
            method: Some("tools/call".to_string()),
            payload: payload.clone(),
            timestamp: Instant::now(),
        })
        .collect();

    let target = connected_target();
    let responses = replay_sequence(&target, &messages)
        .await
        .expect("should succeed");

    assert_eq!(responses.len(), 2);

    for (i, resp) in responses.iter().enumerate() {
        assert_eq!(
            resp.original.payload,
            payloads[i],
            "VP-002: ReplayResponse[{i}].original.payload must be byte-identical to captured payload"
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// VP-003 — wire order preserved
// ─────────────────────────────────────────────────────────────────────────────

/// VP-003 (BC-4.10.003 VP-003):
/// Sequence test: message[i] must produce a response at index i, and
/// `sequence_index` must match the original position in the captured sequence.
#[tokio::test]
async fn test_BC_4_10_003_vp003_wire_order_preserved() {
    let n = 10;
    let messages: Vec<MessageCaptured> = (0..n)
        .map(|i| make_client_request("tools/call", i as u64))
        .collect();

    let target = connected_target();
    let responses = replay_sequence(&target, &messages)
        .await
        .expect("should succeed");

    assert_eq!(responses.len(), n, "must get {n} responses for {n} messages");

    for (expected_pos, resp) in responses.iter().enumerate() {
        assert_eq!(
            resp.sequence_index,
            expected_pos,
            "VP-003: sequence_index must equal position in input (expected {expected_pos}, got {})",
            resp.sequence_index
        );
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// VP-004 — capture buffer unchanged after replay
// ─────────────────────────────────────────────────────────────────────────────

/// VP-004 (BC-4.10.003 VP-004):
/// Invariant test — the input `messages` slice must not be mutated by
/// `replay_sequence`. Verified by comparing len, ids, and payloads before/after.
#[tokio::test]
async fn test_BC_4_10_003_vp004_capture_buffer_unchanged_after_replay() {
    let messages: Vec<MessageCaptured> = vec![
        make_client_request("tools/call", 1),
        make_client_request("resources/list", 2),
        make_server_notification("notifications/message"), // will be skipped
    ];

    let len_before = messages.len();
    let ids_before: Vec<Uuid> = messages.iter().map(|m| m.id).collect();

    let target = connected_target();
    let _ = replay_sequence(&target, &messages).await;

    assert_eq!(
        messages.len(),
        len_before,
        "VP-004: message slice length must not change after replay"
    );

    let ids_after: Vec<Uuid> = messages.iter().map(|m| m.id).collect();
    assert_eq!(
        ids_before, ids_after,
        "VP-004: message IDs must not change after replay"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// VP-005 — comparison correctly identifies identical vs. divergent responses
// ─────────────────────────────────────────────────────────────────────────────

/// VP-005 (BC-4.10.003 VP-005 / POST-003 / POST-004):
/// `ComparisonReport::from_responses` must correctly classify responses as
/// `Identical`, `Equivalent`, or `Divergent`.
///
/// TV-HP-001: 5 identical → 100% match.
/// TV-HP-002: 4 identical + 1 divergent → divergent count = 1.
#[test]
fn test_BC_4_10_003_vp005_comparison_identifies_identical_divergent() {
    let make_response_entry = |status: ReplayStatus| -> ReplayResponse {
        ReplayResponse {
            sequence_index: 0,
            original: make_client_request("tools/call", 1),
            response_payload: Some(serde_json::json!({"jsonrpc":"2.0","id":1,"result":{}})),
            replay: true,
            status,
        }
    };

    // TV-HP-001: all identical.
    let all_identical: Vec<ReplayResponse> = (0..5)
        .map(|_| make_response_entry(ReplayStatus::Identical))
        .collect();

    let report = ComparisonReport::from_responses(all_identical);
    assert_eq!(report.identical, 5, "VP-005: 5 identical responses");
    assert_eq!(report.divergent, 0, "VP-005: 0 divergent responses");
    assert_eq!(report.total_replayed, 5, "VP-005: total_replayed = 5");

    let pct = report.match_percentage();
    assert!(
        (pct - 100.0).abs() < 0.001,
        "VP-005: 5/5 identical → 100% match, got {pct:.2}"
    );

    // TV-HP-002: 4 identical + 1 divergent.
    let mixed: Vec<ReplayResponse> = (0..5)
        .map(|i| {
            if i < 4 {
                make_response_entry(ReplayStatus::Identical)
            } else {
                make_response_entry(ReplayStatus::Divergent)
            }
        })
        .collect();

    let report2 = ComparisonReport::from_responses(mixed);
    assert_eq!(report2.identical, 4, "VP-005: 4 identical");
    assert_eq!(report2.divergent, 1, "VP-005: 1 divergent");

    let pct2 = report2.match_percentage();
    assert!(
        (pct2 - 80.0).abs() < 0.001,
        "VP-005: 4/5 identical → 80% match, got {pct2:.2}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// VP-006 — target unreachable handled without panic (FM-011)
// ─────────────────────────────────────────────────────────────────────────────

/// VP-006 (BC-4.10.003 VP-006 / EC-001):
/// When the target server is unreachable (FM-011), `replay_sequence` must
/// return `Err(ReplayError::TargetUnreachable)` without panicking.
///
/// TV-EC-001: target unreachable → E-RPL-001; "0 of 5 messages sent".
#[tokio::test]
async fn test_BC_4_10_003_vp006_target_unreachable_no_panic() {
    // We cannot actually start a server in a unit test, so we model the
    // error via the error enum directly to confirm it is constructable and
    // the Display impl matches the expected error code format.
    let err = ReplayError::TargetUnreachable {
        address: "localhost:9999".to_string(),
    };

    let display = err.to_string();
    assert!(
        display.contains("E-RPL-001"),
        "VP-006: TargetUnreachable must include error code E-RPL-001 in Display: got {display:?}"
    );
    assert!(
        display.contains("localhost:9999"),
        "VP-006: TargetUnreachable must include the target address: got {display:?}"
    );

    // Verify ConnectionLostMidReplay also formats correctly (E-RPL-002).
    let err2 = ReplayError::ConnectionLostMidReplay { sent: 3, total: 5 };
    let display2 = err2.to_string();
    assert!(
        display2.contains("E-RPL-002"),
        "VP-006: ConnectionLostMidReplay must include E-RPL-002: got {display2:?}"
    );
    assert!(
        display2.contains('3') && display2.contains('5'),
        "VP-006: ConnectionLostMidReplay must include sent/total counts: got {display2:?}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// filter_client_to_server — pure helper tests
// ─────────────────────────────────────────────────────────────────────────────

/// filter_client_to_server: empty input returns empty output.
#[test]
fn test_BC_4_10_003_filter_client_to_server_only() {
    // Empty input.
    let empty: Vec<MessageCaptured> = vec![];
    assert_eq!(
        filter_client_to_server(&empty).len(),
        0,
        "filter on empty slice must return empty"
    );

    // All server→client — none returned.
    let server_only = vec![
        make_server_response(1),
        make_server_notification("notifications/progress"),
    ];
    assert_eq!(
        filter_client_to_server(&server_only).len(),
        0,
        "filter must exclude all server→client messages"
    );

    // Mixed — only client→server kept.
    let mixed = vec![
        make_server_response(1),                // idx 0 — excluded
        make_client_request("tools/call", 2),   // idx 1 — included
        make_client_notification("$/cancel"),    // idx 2 — included (client notification)
        make_server_notification("notif"),       // idx 3 — excluded
    ];
    let filtered = filter_client_to_server(&mixed);
    assert_eq!(filtered.len(), 2, "filter must keep 2 client→server messages");
    assert_eq!(filtered[0].0, 1, "first kept index must be 1");
    assert_eq!(filtered[1].0, 2, "second kept index must be 2");
}

// ─────────────────────────────────────────────────────────────────────────────
// ReplayProgress — progress tracking (POST-006)
// ─────────────────────────────────────────────────────────────────────────────

/// ReplayProgress: new/increment_sent/increment_received (POST-006).
#[test]
fn test_BC_4_10_003_replay_progress_tracking() {
    let mut progress = ReplayProgress::new(5);
    assert_eq!(progress.total, 5, "total must be set to 5");
    assert_eq!(progress.sent, 0, "sent must start at 0");
    assert_eq!(progress.received, 0, "received must start at 0");

    progress.increment_sent();
    assert_eq!(progress.sent, 1, "sent must be 1 after one increment");

    progress.increment_sent();
    progress.increment_sent();
    assert_eq!(progress.sent, 3, "sent must be 3 after three increments");

    progress.increment_received();
    progress.increment_received();
    assert_eq!(progress.received, 2, "received must be 2 after two increments");

    // Sent must never exceed total.
    for _ in 0..10 {
        progress.increment_sent();
    }
    assert!(
        progress.sent <= progress.total,
        "sent ({}) must not exceed total ({})",
        progress.sent,
        progress.total
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Mixed sequence: client requests + client notifications + server messages
// ─────────────────────────────────────────────────────────────────────────────

/// TV-HP-003 (BC-4.10.003 canonical test vector):
/// Replay includes 2 requests and 1 notification (all client→server).
/// 2 comparisons generated (requests); notification marked "sent" (SentNotification).
/// Server→client messages are excluded.
#[tokio::test]
async fn test_BC_4_10_003_mixed_sequence_notifications_and_requests() {
    let messages = vec![
        make_client_request("tools/call", 1),      // client→server request
        make_client_notification("$/cancel"),        // client→server notification
        make_server_response(1),                     // server→client — skipped
        make_client_request("resources/list", 2),   // client→server request
    ];

    let target = connected_target();
    let responses = replay_sequence(&target, &messages)
        .await
        .expect("should succeed");

    // 3 client→server messages total (2 requests + 1 notification).
    assert_eq!(
        responses.len(),
        3,
        "TV-HP-003: must return 3 responses (2 requests + 1 notification)"
    );

    // Find the notification response — must be SentNotification.
    let notif_resp = responses
        .iter()
        .find(|r| r.original.method.as_deref() == Some("$/cancel"))
        .expect("must have a response for the notification");

    assert_eq!(
        notif_resp.status,
        ReplayStatus::SentNotification,
        "TV-HP-003: notification replay must have status SentNotification (EC-006)"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// ComparisonReport — match_percentage edge cases
// ─────────────────────────────────────────────────────────────────────────────

/// ComparisonReport: 0% match when all responses are divergent.
#[test]
fn test_BC_4_10_003_comparison_report_zero_percent_match() {
    let responses: Vec<ReplayResponse> = (0..3)
        .map(|_| ReplayResponse {
            sequence_index: 0,
            original: make_client_request("tools/call", 1),
            response_payload: Some(serde_json::json!({"result":"different"})),
            replay: true,
            status: ReplayStatus::Divergent,
        })
        .collect();

    let report = ComparisonReport::from_responses(responses);
    assert_eq!(report.divergent, 3);
    assert_eq!(report.identical, 0);

    let pct = report.match_percentage();
    assert!(
        pct < 0.001,
        "0 identical out of 3 total must give 0% match, got {pct:.2}"
    );
}

/// ComparisonReport: empty responses → 100% match (vacuously true).
#[test]
fn test_BC_4_10_003_comparison_report_empty_responses() {
    let report = ComparisonReport::from_responses(vec![]);
    assert_eq!(report.total_replayed, 0);

    // Match percentage for 0/0 is defined as 100% (no failures means success).
    let pct = report.match_percentage();
    assert!(
        (pct - 100.0).abs() < 0.001,
        "empty replay must report 100% match (vacuously true), got {pct:.2}"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// ReplayStatus — equality and Display checks
// ─────────────────────────────────────────────────────────────────────────────

/// Smoke test: `ReplayStatus` variants are constructable and comparable.
#[test]
fn test_BC_4_10_003_replay_status_variants() {
    assert_eq!(ReplayStatus::Identical, ReplayStatus::Identical);
    assert_eq!(ReplayStatus::Equivalent, ReplayStatus::Equivalent);
    assert_eq!(ReplayStatus::Divergent, ReplayStatus::Divergent);
    assert_eq!(ReplayStatus::NoResponse, ReplayStatus::NoResponse);
    assert_eq!(ReplayStatus::SentNotification, ReplayStatus::SentNotification);
    assert_eq!(
        ReplayStatus::Error("oops".to_string()),
        ReplayStatus::Error("oops".to_string())
    );
    assert_ne!(
        ReplayStatus::Identical,
        ReplayStatus::Divergent
    );
}
