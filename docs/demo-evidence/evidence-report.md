# Demo Evidence Report — STORY-028: Timing Analysis

Generated: 2026-03-31

## Summary

This report maps demo recording artifacts to acceptance criteria and edge cases
for STORY-028 (Timing Analysis — `TimingAnalyzer` in `forge-traffic`).

All recordings were produced via VHS tape scripts and captured as both `.webm`
(video) and `.gif` (animated image) for review.

---

## Acceptance Criteria Coverage

### AC-001 — Per-Message Latency (BC-4.09.002)

> For each request/response pair matched by ID, `forge-traffic` computes
> `latency_ms: f64` as the elapsed time between request `MessageCaptured` and
> response `MessageCaptured`. Result stored in `TimedMessage`.
>
> **Test:** `test_BC_4_09_002_per_message_latency()`

| Artifact | Type |
|----------|------|
| `AC-001-per-message-latency.tape` | VHS tape script |
| `AC-001-per-message-latency.webm` | Video recording |
| `AC-001-per-message-latency.gif` | Animated GIF |

---

### AC-002 — Throughput Calculation (BC-4.09.002)

> The traffic analyzer computes throughput as messages/second over a
> configurable sliding window (default 10s). Updated on each new
> `MessageCaptured` event.
>
> **Test:** `test_BC_4_09_002_throughput_calculation()`

| Artifact | Type |
|----------|------|
| `AC-002-throughput-calculation.tape` | VHS tape script |
| `AC-002-throughput-calculation.webm` | Video recording |
| `AC-002-throughput-calculation.gif` | Animated GIF |

---

### AC-003 — Unmatched Response Stored Without Latency (BC-4.09.002)

> Responses without a matching request ID (late responses, server-initiated)
> are stored without latency data (`latency_ms: None`).
>
> **Test:** `test_BC_4_09_002_unmatched_response_no_latency()`

| Artifact | Type |
|----------|------|
| `AC-003-unmatched-response.tape` | VHS tape script |
| `AC-003-unmatched-response.webm` | Video recording |
| `AC-003-unmatched-response.gif` | Animated GIF |

---

### AC-004 — Message Ordering Preserved (BC-4.09.002 / DI-006)

> Messages are stored in received-order. Reordered batch responses (E-PRO-009)
> are flagged in `TimedMessage.reordered: bool`.
>
> **Test:** `test_BC_4_09_002_message_ordering_preserved()`

| Artifact | Type |
|----------|------|
| `AC-004-message-ordering.tape` | VHS tape script |
| `AC-004-message-ordering.webm` | Video recording |
| `AC-004-message-ordering.gif` | Animated GIF |

---

## Edge Case Coverage

### EC-001 — Notification Without Request ID (No Pairing)

> Notifications (no request ID) produce no latency pairing; direction tracked
> only.

| Artifact | Type |
|----------|------|
| `EC-001-notification-no-pairing.tape` | VHS tape script |
| `EC-001-notification-no-pairing.webm` | Video recording |
| `EC-001-notification-no-pairing.gif` | Animated GIF |

---

### EC-002 — Duplicate Response ID Flagged

> When a second response arrives with the same ID as a previously matched
> response, the duplicate is flagged in `TimedMessage`.

| Artifact | Type |
|----------|------|
| `EC-002-duplicate-response.tape` | VHS tape script |
| `EC-002-duplicate-response.webm` | Video recording |
| `EC-002-duplicate-response.gif` | Animated GIF |

---

### EC-003 — Empty Analyzer Baseline

> Validates that a freshly constructed `TimingAnalyzer` with no messages
> produces zero-value metrics without panicking.

| Artifact | Type |
|----------|------|
| `EC-003-empty-analyzer.tape` | VHS tape script |

> Note: No `.webm`/`.gif` produced for EC-003 (output is trivially empty;
> tape script serves as sole artifact).

---

## Artifact Inventory

| File | AC/EC | Format |
|------|-------|--------|
| AC-001-per-message-latency.tape | AC-001 | VHS tape |
| AC-001-per-message-latency.webm | AC-001 | Video |
| AC-001-per-message-latency.gif | AC-001 | GIF |
| AC-002-throughput-calculation.tape | AC-002 | VHS tape |
| AC-002-throughput-calculation.webm | AC-002 | Video |
| AC-002-throughput-calculation.gif | AC-002 | GIF |
| AC-003-unmatched-response.tape | AC-003 | VHS tape |
| AC-003-unmatched-response.webm | AC-003 | Video |
| AC-003-unmatched-response.gif | AC-003 | GIF |
| AC-004-message-ordering.tape | AC-004 | VHS tape |
| AC-004-message-ordering.webm | AC-004 | Video |
| AC-004-message-ordering.gif | AC-004 | GIF |
| EC-001-notification-no-pairing.tape | EC-001 | VHS tape |
| EC-001-notification-no-pairing.webm | EC-001 | Video |
| EC-001-notification-no-pairing.gif | EC-001 | GIF |
| EC-002-duplicate-response.tape | EC-002 | VHS tape |
| EC-002-duplicate-response.webm | EC-002 | Video |
| EC-002-duplicate-response.gif | EC-002 | GIF |
| EC-003-empty-analyzer.tape | EC-003 | VHS tape |

**Total:** 19 artifacts across 4 ACs and 3 ECs.

---

## Traceability

| AC/EC | Behavioral Contract | Test | Evidence |
|-------|---------------------|------|---------|
| AC-001 | BC-4.09.002 (per-message latency) | `test_BC_4_09_002_per_message_latency` | ✅ webm + gif |
| AC-002 | BC-4.09.002 (throughput) | `test_BC_4_09_002_throughput_calculation` | ✅ webm + gif |
| AC-003 | BC-4.09.002 (unmatched) | `test_BC_4_09_002_unmatched_response_no_latency` | ✅ webm + gif |
| AC-004 | BC-4.09.002 / DI-006 | `test_BC_4_09_002_message_ordering_preserved` | ✅ webm + gif |
| EC-001 | — | — | ✅ tape |
| EC-002 | — | — | ✅ webm + gif |
| EC-003 | — | — | ✅ tape |
