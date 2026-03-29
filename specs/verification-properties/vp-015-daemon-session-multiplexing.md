---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-03-29T14:57:00
phase: 1b
inputs: [specs/prd.md, specs/architecture/ARCH-INDEX.md, specs/behavioral-contracts/BC-1.03.001.md]
input-hash: ""
traces_to: specs/prd.md
source_bc: [BC-1.03.001]
module: forge-daemon
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

# VP-015: Daemon Session-Pool Multiplexing Correctness

## Property Statement

When multiple clients share a pooled MCP server connection through the daemon, three isolation guarantees MUST hold:

1. **Session Isolation (no cross-talk):** A response from server S MUST be delivered *only* to the client whose session originated the corresponding request. A message from server A MUST NOT leak to a client subscribed exclusively to server B.

2. **Server-Initiated Callback Routing:** Server-initiated requests (sampling, elicitation, progress notifications, `list_changed` events) arriving on a pooled connection MUST be routed to the correct client session based on the session's registered capabilities. Specifically:
   - A sampling request MUST target only the client that advertised sampling capability for that server connection.
   - An elicitation request MUST target only the client that advertised elicitation capability.
   - A `list_changed` notification MUST be delivered to *all* clients sharing that server connection (broadcast semantic).
   - A progress notification with a known `progressToken` MUST route to the client that initiated the originating request.

3. **Connection Lifecycle Independence:** Disconnecting client C₁ from the daemon MUST NOT corrupt, drop, or misroute in-flight messages for any other client C₂…Cₙ sharing the same pooled server connection. The pooled server connection itself MUST remain live as long as at least one client references it.

This is a **session isolation** property: the daemon's multiplexing layer MUST maintain strict request/response correlation and capability-based routing across all concurrent clients.

## Source Contract

- **BC-1.03.001** — Daemon Lazy Start and Session Pooling
  - POST-003: MCP server connections are pooled; new client reuses existing connection.
  - POST-004: Each client gets a unique session ID for request routing.
  - POST-005: Daemon tracks active client count.
  - EC-002: Two CLI clients connect simultaneously to the same server; responses route to correct client.
  - DI-003: Session ID uniqueness.
- **HS-018** — Holdout scenario requiring no cross-talk between concurrent CLI calls.

## Proof Method

| Aspect | Detail |
|--------|--------|
| Method | Property-based testing (proptest) + targeted integration tests |
| Tool | proptest (Rust), tokio test runtime |
| Rationale | `forge-daemon` is effectful-heavy (socket I/O, process lifecycle), so model checking of the full daemon is not feasible. However, the session-routing logic — the request/response correlation table and callback dispatch map — can be extracted as a **pure core component** and verified with proptest. Integration tests cover the effectful wiring. |

### Decomposition Strategy

The daemon's multiplexing correctness depends on a **SessionRouter** (or equivalent) that:
- Maintains a `HashMap<RequestId, ClientSessionId>` for request/response correlation
- Maintains a `HashMap<ServerConnectionId, Vec<ClientSessionId>>` for broadcast dispatch
- Maintains per-client capability registrations for targeted server-initiated routing

This routing table is pure state — no I/O. It can be verified exhaustively with proptest.

## Harness Skeleton

### Property 1: Session Isolation (No Cross-Talk)

```rust
use proptest::prelude::*;

/// A request/response pair preserves client identity through the router.
#[test]
fn session_isolation_no_crosstalk() {
    proptest!(|(
        num_clients in 2u8..=20,
        num_requests in 1u16..=200,
        seed: u64,
    )| {
        let mut router = SessionRouter::new();
        let mut rng = StdRng::seed_from_u64(seed);

        // Register N clients, all sharing one pooled server connection
        let server_id = ServerConnectionId::new("test-server");
        let client_ids: Vec<ClientSessionId> = (0..num_clients)
            .map(|i| ClientSessionId::new(format!("client-{i}")))
            .collect();
        for cid in &client_ids {
            router.register_client(cid.clone(), server_id.clone());
        }

        // Each client sends interleaved requests
        let mut expected: HashMap<RequestId, ClientSessionId> = HashMap::new();
        for _ in 0..num_requests {
            let client = &client_ids[rng.gen_range(0..client_ids.len())];
            let req_id = router.send_request(client.clone(), server_id.clone());
            expected.insert(req_id, client.clone());
        }

        // Responses arrive in arbitrary order — verify routing
        let response_order: Vec<RequestId> = {
            let mut ids: Vec<_> = expected.keys().cloned().collect();
            ids.shuffle(&mut rng);
            ids
        };
        for req_id in response_order {
            let routed_to = router.route_response(&req_id)
                .expect("Response must route to a client");
            prop_assert_eq!(
                &routed_to, expected.get(&req_id).unwrap(),
                "Response for {:?} routed to wrong client", req_id
            );
        }
    });
}
```

### Property 2: Server-Initiated Callback Routing

```rust
#[test]
fn server_callbacks_route_correctly() {
    proptest!(|(
        num_clients in 2u8..=10,
        num_callbacks in 1u16..=50,
        seed: u64,
    )| {
        let mut router = SessionRouter::new();
        let mut rng = StdRng::seed_from_u64(seed);
        let server_id = ServerConnectionId::new("test-server");

        let client_ids: Vec<ClientSessionId> = (0..num_clients)
            .map(|i| ClientSessionId::new(format!("client-{i}")))
            .collect();

        // Only one client registers sampling capability per server
        let sampling_client = &client_ids[0];
        router.register_client_capability(
            sampling_client.clone(), server_id.clone(), Capability::Sampling
        );

        // Only one client registers elicitation capability
        let elicitation_client = &client_ids[1 % client_ids.len()];
        router.register_client_capability(
            elicitation_client.clone(), server_id.clone(), Capability::Elicitation
        );

        // All clients are subscribed to list_changed (broadcast)
        for cid in &client_ids {
            router.register_client(cid.clone(), server_id.clone());
        }

        // Sampling request from server → routes to sampling_client only
        let targets = router.route_server_request(
            server_id.clone(), ServerRequest::Sampling { .. }
        );
        prop_assert_eq!(targets.len(), 1);
        prop_assert_eq!(&targets[0], sampling_client);

        // Elicitation request → routes to elicitation_client only
        let targets = router.route_server_request(
            server_id.clone(), ServerRequest::Elicitation { .. }
        );
        prop_assert_eq!(targets.len(), 1);
        prop_assert_eq!(&targets[0], elicitation_client);

        // list_changed → broadcast to ALL clients on this server
        let targets = router.route_server_notification(
            server_id.clone(), ServerNotification::ListChanged
        );
        prop_assert_eq!(targets.len(), client_ids.len());

        // Progress with known token → routes to originating client
        let originator = &client_ids[rng.gen_range(0..client_ids.len())];
        let req_id = router.send_request(originator.clone(), server_id.clone());
        let progress_token = router.get_progress_token(&req_id).unwrap();
        let targets = router.route_server_notification(
            server_id.clone(),
            ServerNotification::Progress { token: progress_token }
        );
        prop_assert_eq!(targets.len(), 1);
        prop_assert_eq!(&targets[0], originator);
    });
}
```

### Property 3: Connection Lifecycle Independence

```rust
#[test]
fn client_disconnect_does_not_affect_others() {
    proptest!(|(
        num_clients in 3u8..=15,
        disconnect_index in 0u8..=14,
        num_requests_after in 1u16..=100,
        seed: u64,
    )| {
        let mut router = SessionRouter::new();
        let mut rng = StdRng::seed_from_u64(seed);
        let server_id = ServerConnectionId::new("test-server");

        let client_ids: Vec<ClientSessionId> = (0..num_clients)
            .map(|i| ClientSessionId::new(format!("client-{i}")))
            .collect();
        for cid in &client_ids {
            router.register_client(cid.clone(), server_id.clone());
        }

        // Each client sends some in-flight requests
        let mut inflight: HashMap<RequestId, ClientSessionId> = HashMap::new();
        for cid in &client_ids {
            let req_id = router.send_request(cid.clone(), server_id.clone());
            inflight.insert(req_id, cid.clone());
        }

        // Disconnect one client
        let disconnect_idx = (disconnect_index as usize) % client_ids.len();
        let disconnected = &client_ids[disconnect_idx];
        router.disconnect_client(disconnected);

        // Server connection is still live
        prop_assert!(router.is_server_connected(&server_id));

        // Remaining clients' in-flight requests still route correctly
        for (req_id, expected_client) in &inflight {
            if expected_client == disconnected {
                // Disconnected client's requests are cleaned up
                prop_assert!(router.route_response(req_id).is_none());
            } else {
                let routed = router.route_response(req_id)
                    .expect("Surviving client's response must still route");
                prop_assert_eq!(&routed, expected_client);
            }
        }

        // New requests from surviving clients work normally
        let surviving: Vec<_> = client_ids.iter()
            .filter(|c| *c != disconnected)
            .collect();
        for _ in 0..num_requests_after {
            let client = surviving[rng.gen_range(0..surviving.len())];
            let req_id = router.send_request(client.clone(), server_id.clone());
            let routed = router.route_response(&req_id).unwrap();
            prop_assert_eq!(&routed, client);
        }
    });
}
```

### Sub-Property: Pool Drain Triggers Server Disconnect

```rust
#[test]
fn all_clients_disconnect_releases_server() {
    proptest!(|(num_clients in 1u8..=10)| {
        let mut router = SessionRouter::new();
        let server_id = ServerConnectionId::new("test-server");
        let client_ids: Vec<ClientSessionId> = (0..num_clients)
            .map(|i| ClientSessionId::new(format!("client-{i}")))
            .collect();

        for cid in &client_ids {
            router.register_client(cid.clone(), server_id.clone());
        }

        // Disconnect all clients
        for cid in &client_ids {
            router.disconnect_client(cid);
        }

        // Server connection should be marked for release
        prop_assert!(!router.is_server_connected(&server_id));
        prop_assert_eq!(router.active_client_count(&server_id), 0);
    });
}
```

## Feasibility Assessment

| Factor | Assessment |
|--------|-----------|
| Input space | 2–20 clients × 1–200 interleaved requests × arbitrary ordering |
| Complexity | Medium — routing table is a pure HashMap-based structure; callbacks add capability-based dispatch |
| Tool support | proptest handles HashMap+enum state well; tokio test runtime supports async integration tests |
| Decomposition | Session routing logic can be extracted as a pure `SessionRouter` struct separate from socket I/O |
| Expected time | Seconds (proptest), minutes (integration tests with mock server) |
| Risk | Architecture MUST enforce the pure/effectful split — routing logic in pure core, socket handling in effectful shell. If routing is entangled with I/O, refactor required. |
| Verdict | **FEASIBLE** |

## Integration Test Companion

The proptest harnesses verify the pure routing logic. Additionally, integration tests (not formally verified) validate the effectful wiring:

| Test | Validates |
|------|-----------|
| `test_two_cli_clients_interleaved` | Two CLI processes send requests to same server via daemon; responses arrive at correct client (EC-002) |
| `test_sampling_callback_routes_to_correct_client` | Server sends sampling request; only the capability-registered client receives it |
| `test_client_crash_does_not_disrupt_others` | SIGKILL one client; verify other client's in-flight request completes (EC-004 + VP-015 P3) |
| `test_100_concurrent_clients` | 100 clients sharing one server connection; no cross-talk under load (EC-006) |

## Traceability

| Trace | Target |
|-------|--------|
| BC | BC-1.03.001 (POST-003, POST-004, EC-002, DI-003) |
| Holdout | HS-018 (no cross-talk between concurrent CLI calls) |
| Domain Invariant | DI-003 (session ID uniqueness) |
| Module | forge-daemon (effectful shell) + SessionRouter (pure core, to be extracted) |
| Risk | R-NNN (shared state corruption under concurrency) |

## Lifecycle

| Field | Value |
|-------|-------|
| Introduced | v0.1.0 |
| Modified | — |
| Deprecated | — |
| Retired | — |

---

<!-- ADV-P2-001 resolution note:
  Created VP-015 to close the verification gap identified in ADV-P2-001 (Pass 2).
  The adversary correctly identified that forge-daemon was listed under "Modules without VPs
  (by design)" in verification-coverage-matrix.md, despite BC-1.03.001 requiring isolation
  guarantees (POST-003, POST-004, EC-002) and HS-018 stress-testing cross-talk.

  Resolution approach: Rather than attempting to formally verify the full effectful daemon,
  this VP decomposes the problem into a pure SessionRouter core (verifiable with proptest)
  and effectful integration tests. The SessionRouter must be extracted during implementation
  as a pure struct with no I/O dependencies — this is a hard architectural constraint.

  Three sub-properties cover the full isolation surface:
    P1: Request/response routing isolation (no cross-talk)
    P2: Server-initiated callback dispatch (sampling, elicitation, list_changed, progress)
    P3: Connection lifecycle independence (client disconnect doesn't affect others)

  VP-INDEX.md and verification-coverage-matrix.md updated in the same commit.
-->
