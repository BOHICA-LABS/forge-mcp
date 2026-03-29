---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-03-29T11:25:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: ""
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: "Traffic Inspection"
capability: "CAP-010"
lifecycle_status: active
introduced: v0.1.0
---

# BC-4.10.003 — Message Sequence Replay Against Target Server

## Summary

The traffic inspection subsystem can replay a sequence of captured messages against an explicitly designated target server. Replay requires explicit server designation (DI-007) — the system never replays to the original server without deliberate user intent. After replay, responses from the target are compared with the original captured responses to identify behavioral differences. This enables regression testing and server compatibility validation.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | Capture buffer contains at least one client→server message to replay |
| PRE-002 | A target server is explicitly designated by the user (DI-007) |
| PRE-003 | Target server is reachable and accepting connections |
| PRE-004 | User has confirmed the replay action (replay is a deliberate, non-automatic operation) |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | All selected client→server messages are sent to the target server in their original wire order |
| POST-002 | Responses from the target server are captured |
| POST-003 | A comparison report is generated: original response vs. replay response for each matched request |
| POST-004 | Differences are categorized: identical, semantically equivalent (e.g., different timestamps but same structure), or divergent |
| POST-005 | Replay never sends to the original server unless the user explicitly designates it as the target |
| POST-006 | Replay progress is shown in the TUI (messages sent: N/M, responses received: K/M) |

## Invariants

| ID | Invariant |
|----|-----------|
| INV-001 | DI-007: Replay requires explicit target designation — never implicit or default to original |
| INV-002 | Replayed messages are byte-identical to original captured payloads (DI-005 applies to replay too) |
| INV-003 | Replay preserves original wire order (messages sent in capture sequence) |
| INV-004 | Original capture buffer is not modified by replay operations |
| INV-005 | Replay responses are stored separately from original capture buffer |

## Edge Cases

| ID | Scenario | Expected Behavior | Source |
|----|----------|-------------------|--------|
| EC-001 | Target server unreachable | Abort replay; report E-RPL-001 "Target server unreachable: {address}" with number of unsent messages | FM-011 |
| EC-002 | Target server disconnects mid-replay | Stop replay; report partial results for messages sent so far; E-RPL-002 "Connection lost after {N} of {M} messages" | FM-011 |
| EC-003 | Target server responds with different JSON-RPC id | Map by sequence position (1st request → 1st response); note id mismatch in comparison report | — |
| EC-004 | Target server does not respond to a request within timeout (30s) | Mark as "no response" in comparison; continue with next message | — |
| EC-005 | Replay of server→client messages (not applicable) | Only client→server messages are replayable; server→client messages are skipped with info message | — |
| EC-006 | Replay includes notifications (no id, no expected response) | Send notification; do not expect response; mark as "sent (notification)" in progress | — |
| EC-007 | User designates original server as replay target | Allowed (explicit intent verified); show warning "Replaying to original server — responses may differ due to state changes" | — |
| EC-008 | Empty selection (no client→server messages in filtered view) | Display "No replayable messages selected (only client→server messages can be replayed)" | — |
| EC-009 | Replay 10,000+ messages | Replay proceeds with throttling (configurable rate limit, default: 100 msg/sec); progress bar shown | — |

## Comparison Report Structure

| Field | Description |
|-------|-------------|
| Request # | Sequential index of the replayed request |
| Method | JSON-RPC method name |
| Original Response | Summary of original captured response |
| Replay Response | Summary of target server's response |
| Status | `identical`, `equivalent`, `divergent`, `no-response`, `error` |
| Diff | Structural diff for divergent responses |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-HP-001 | Replay 5 messages to target server; all return identical responses | Comparison: 5/5 identical; report summary "100% match" |
| TV-HP-002 | Replay 5 messages; 1 response differs (different result value) | Comparison: 4 identical, 1 divergent; diff shows specific field difference |
| TV-HP-003 | Replay includes 2 requests and 1 notification | 2 comparisons generated (requests); notification marked "sent"; progress: 3/3 sent |
| TV-HP-004 | Target is explicitly designated via `:replay target-server` | Replay proceeds to target-server; confirmation prompt shown first |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-EC-001 | Target server unreachable | E-RPL-001; "0 of 5 messages sent" | FM-011 |
| TV-EC-002 | Target disconnects after 3 of 5 messages | Partial report: 3 comparisons; E-RPL-002; "2 messages not sent" |
| TV-EC-003 | User selects original server as target | Warning displayed; replay proceeds after user confirmation |
| TV-EC-004 | Response has different timestamp but identical structure | Status: "equivalent" (timestamp fields excluded from structural comparison) |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-ERR-001 | No target server designated | Error: "No replay target specified. Use :replay <server-name>" |
| TV-ERR-002 | Target server name not found in registry | Error: "Server not found: {name}. Available servers: {list}" |
| TV-ERR-003 | Replay attempted with zero replayable messages | Info: "No replayable messages selected" |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | DI-007: replay function refuses to execute without explicit target designation | State machine test |
| VP-002 | Replayed bytes are identical to captured bytes (DI-005) | Byte comparison test |
| VP-003 | Replay preserves wire order: message[i] is sent before message[i+1] | Sequence test |
| VP-004 | Original capture buffer is unchanged after replay operation | Invariant test |
| VP-005 | Comparison correctly identifies identical vs. divergent responses | Property-based test |
| VP-006 | Target unreachable is handled without panic (FM-011) | Error handling test |

## Traceability

| Item | Reference |
|------|-----------|
| L2 Capability | CAP-010 (Traffic Filtering & Replay) |
| Domain Invariant | DI-005 (byte-identical replay), DI-007 (explicit target designation) |
| Failure Mode | FM-011 (target unreachable) |
| Error Code | E-RPL-001 (target unreachable), E-RPL-002 (connection lost mid-replay) |
| Related BCs | BC-4.09.001 (capture — replay source), BC-4.10.001 (filtering — selects messages for replay), BC-4.09.002 (timing — replay can generate new timing data) |
