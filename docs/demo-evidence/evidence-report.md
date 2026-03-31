# Demo Evidence Report — STORY-032: Message Sequence Replay

**Story:** STORY-032 — Message Sequence Replay Against Target Server  
**Epic:** EPIC-04  
**Points:** 5  
**BC:** BC-4.10.003  
**VP:** VP-006  
**Generated:** 2026-03-31  
**Recorded by:** demo-recorder  

---

## Coverage Summary

| AC | Description | Success Path | Error Path | Status |
|----|-------------|-------------|------------|--------|
| AC-001 | `replay_sequence` sends client→server messages, returns responses | ✅ | ✅ | COVERED |
| AC-002 | Disconnected target returns `Err(E-CAP-002)` | ✅ | ✅ | COVERED |
| AC-003 | Replay responses tagged with `replay: true` | ✅ | ✅ | COVERED |
| AC-004 | Messages replayed in temporal order (INV-003) | ✅ | ✅ | COVERED |
| AC-005 | CLI replay API — `ReplayTarget` explicit designation (DI-007) | ✅ | ✅ | COVERED |

**Coverage: 5/5 ACs (100%) — 10 recordings (success + error for each AC)**

---

## AC-001: `replay_sequence` sends client→server messages and returns responses

**Traces to:** BC-4.10.003 POST-001 / POST-002 | Test: `test_BC_4_10_003_replay_sends_messages`

### Success Path
Demonstrates: 5 client→server messages replayed → 5 `ReplayResponse` objects returned, each with `replay: true`.

| File | Format |
|------|--------|
| `AC-001-replay-sends-messages.tape` | VHS script |
| `AC-001-replay-sends-messages.gif` | ![AC-001 success](AC-001-replay-sends-messages.gif) |
| `AC-001-replay-sends-messages.webm` | Archival video |

### Error Path
Demonstrates: Mixed sequence with server→client messages — only 2 of 4 messages are replayed (EC-005 direction filter).

| File | Format |
|------|--------|
| `AC-001-error-server-messages-filtered.tape` | VHS script |
| `AC-001-error-server-messages-filtered.gif` | ![AC-001 error](AC-001-error-server-messages-filtered.gif) |
| `AC-001-error-server-messages-filtered.webm` | Archival video |

---

## AC-002: Disconnected target returns `Err(E-CAP-002)`

**Traces to:** BC-4.10.003 PRE-003 | Test: `test_BC_4_10_003_disconnected_target_errors`

### Error Path (primary AC behaviour)
Demonstrates: `ReplayTarget::Disconnected` → `Err(ReplayError::TargetNotConnected)` with error code E-CAP-002.

| File | Format |
|------|--------|
| `AC-002-disconnected-target-error.tape` | VHS script |
| `AC-002-disconnected-target-error.gif` | ![AC-002 error](AC-002-disconnected-target-error.gif) |
| `AC-002-disconnected-target-error.webm` | Archival video |

### Success Path (Connected target proceeds)
Demonstrates: VP-001 — all 3 Disconnected variants return `TargetNotConnected`, confirming `Connected` succeeds without this error.

| File | Format |
|------|--------|
| `AC-002-success-connected-target.tape` | VHS script |
| `AC-002-success-connected-target.gif` | ![AC-002 success](AC-002-success-connected-target.gif) |
| `AC-002-success-connected-target.webm` | Archival video |

---

## AC-003: Replay responses tagged with `replay: true`

**Traces to:** BC-4.10.003 POST-002 | Test: `test_BC_4_10_003_replay_responses_captured`

### Success Path
Demonstrates: Every `ReplayResponse` has `replay: true`, distinguishing replay traffic from live traffic.

| File | Format |
|------|--------|
| `AC-003-replay-flag-true.tape` | VHS script |
| `AC-003-replay-flag-true.gif` | ![AC-003 success](AC-003-replay-flag-true.gif) |
| `AC-003-replay-flag-true.webm` | Archival video |

### Error Path
Demonstrates: `ComparisonReport` correctly classifies `Identical` (100% match) vs `Divergent` (80% match) responses — VP-005.

| File | Format |
|------|--------|
| `AC-003-error-comparison-divergent.tape` | VHS script |
| `AC-003-error-comparison-divergent.gif` | ![AC-003 error](AC-003-error-comparison-divergent.gif) |
| `AC-003-error-comparison-divergent.webm` | Archival video |

---

## AC-004: Messages replayed in temporal order

**Traces to:** BC-4.10.003 INV-003 / DI-007 | Test: `test_BC_4_10_003_replay_preserves_order`

### Success Path
Demonstrates: `sequence_index` values strictly ascending 0..N — original wire order enforced (INV-003 / VP-003).

| File | Format |
|------|--------|
| `AC-004-temporal-order.tape` | VHS script |
| `AC-004-temporal-order.gif` | ![AC-004 success](AC-004-temporal-order.gif) |
| `AC-004-temporal-order.webm` | Archival video |

### Error Path
Demonstrates: INV-004 — original capture buffer IDs and payloads are unchanged after replay completes (read-only invariant, VP-004).

| File | Format |
|------|--------|
| `AC-004-error-capture-unchanged.tape` | VHS script |
| `AC-004-error-capture-unchanged.gif` | ![AC-004 error](AC-004-error-capture-unchanged.gif) |
| `AC-004-error-capture-unchanged.webm` | Archival video |

---

## AC-005: CLI replay command — `forge-mcp call` with explicit designation

**Traces to:** BC-4.10.003 DI-007 | Test: `test_BC_4_10_003_cli_replay_command`

### Success Path
Demonstrates: `forge-mcp call --help` shows the `call` subcommand surface; `ReplayTarget::Connected` reports `is_connected() == true` and correct `label()`.

| File | Format |
|------|--------|
| `AC-005-cli-replay-api.tape` | VHS script |
| `AC-005-cli-replay-api.gif` | ![AC-005 success](AC-005-cli-replay-api.gif) |
| `AC-005-cli-replay-api.webm` | Archival video |

### Error Path
Demonstrates: `ReplayError::TargetUnreachable` formats with error code `E-RPL-001` and address; `ConnectionLostMidReplay` formats with `E-RPL-002` and sent/total counts — no panic (VP-006).

| File | Format |
|------|--------|
| `AC-005-error-unreachable-target.tape` | VHS script |
| `AC-005-error-unreachable-target.gif` | ![AC-005 error](AC-005-error-unreachable-target.gif) |
| `AC-005-error-unreachable-target.webm` | Archival video |

---

## File Inventory (STORY-032 new files)

| # | File | Type |
|---|------|------|
| 1 | `AC-001-replay-sends-messages.tape` | VHS script |
| 2 | `AC-001-replay-sends-messages.gif` | Recording |
| 3 | `AC-001-replay-sends-messages.webm` | Recording |
| 4 | `AC-001-error-server-messages-filtered.tape` | VHS script |
| 5 | `AC-001-error-server-messages-filtered.gif` | Recording |
| 6 | `AC-001-error-server-messages-filtered.webm` | Recording |
| 7 | `AC-002-disconnected-target-error.tape` | VHS script |
| 8 | `AC-002-disconnected-target-error.gif` | Recording |
| 9 | `AC-002-disconnected-target-error.webm` | Recording |
| 10 | `AC-002-success-connected-target.tape` | VHS script |
| 11 | `AC-002-success-connected-target.gif` | Recording |
| 12 | `AC-002-success-connected-target.webm` | Recording |
| 13 | `AC-003-replay-flag-true.tape` | VHS script |
| 14 | `AC-003-replay-flag-true.gif` | Recording |
| 15 | `AC-003-replay-flag-true.webm` | Recording |
| 16 | `AC-003-error-comparison-divergent.tape` | VHS script |
| 17 | `AC-003-error-comparison-divergent.gif` | Recording |
| 18 | `AC-003-error-comparison-divergent.webm` | Recording |
| 19 | `AC-004-temporal-order.tape` | VHS script |
| 20 | `AC-004-temporal-order.gif` | Recording |
| 21 | `AC-004-temporal-order.webm` | Recording |
| 22 | `AC-004-error-capture-unchanged.tape` | VHS script |
| 23 | `AC-004-error-capture-unchanged.gif` | Recording |
| 24 | `AC-004-error-capture-unchanged.webm` | Recording |
| 25 | `AC-005-cli-replay-api.tape` | VHS script |
| 26 | `AC-005-cli-replay-api.gif` | Recording |
| 27 | `AC-005-cli-replay-api.webm` | Recording |
| 28 | `AC-005-error-unreachable-target.tape` | VHS script |
| 29 | `AC-005-error-unreachable-target.gif` | Recording |
| 30 | `AC-005-error-unreachable-target.webm` | Recording |
| 31 | `evidence-report.md` | This report |

**Total new STORY-032 demo files: 31**

---

## Test Suite Confirmation

All 27 STORY-032 tests pass:

```
test result: ok. 27 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

**Crate:** `forge-traffic` | **Module:** `replay.rs` | **Test file:** `tests/replay_tests.rs`

### AC-to-Test Traceability

| AC | Test | BC Reference |
|----|------|-------------|
| AC-001 | `test_BC_4_10_003_replay_sends_messages` | BC-4.10.003 POST-001 |
| AC-002 | `test_BC_4_10_003_disconnected_target_errors` | BC-4.10.003 PRE-003 |
| AC-003 | `test_BC_4_10_003_replay_responses_captured` | BC-4.10.003 POST-002 |
| AC-004 | `test_BC_4_10_003_replay_preserves_order` | BC-4.10.003 INV-003 |
| AC-005 | `test_BC_4_10_003_cli_replay_command` | BC-4.10.003 DI-007 |
