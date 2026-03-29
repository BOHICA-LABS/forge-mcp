---
document_type: story
story_id: STORY-029
epic_id: EPIC-04
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-027]
blocks: [STORY-030, STORY-031, STORY-032]
behavioral_contracts: [BC-4.09.003]
verification_properties: [VP-005]
priority: P0
assumption_validations: []
risk_mitigations: [R-010]
---

# STORY-029: Capture Buffer Management with Bounded Memory

## Narrative
- **As an** AI Platform Engineer running long TUI sessions
- **I want to** have traffic capture bounded in memory
- **So that** Forge MCP doesn't consume unbounded RAM during extended monitoring

## Acceptance Criteria

### AC-001 (traces to BC-4.09.003 postcondition — ring buffer stores messages)
Captured `MessageCaptured` events are stored in a ring buffer with configurable capacity (default: enough to hold ~100MB of messages). Oldest messages are evicted when capacity is reached.
- **Test:** `test_BC_4_09_003_ring_buffer_stores_messages()`

### AC-002 (traces to BC-4.09.003 postcondition — FIFO eviction)
When the buffer is full, the oldest message is evicted first (FIFO). No random eviction, no priority eviction. (VP-005: ring buffer bounds + FIFO ordering.)
- **Test:** `test_BC_4_09_003_fifo_eviction()` (Kani proof for VP-005)

### AC-003 (traces to BC-4.09.003 — memory bound)
The ring buffer never consumes more than the configured limit (default 100MB, configurable via `--capture-limit`). Memory is bounded regardless of message frequency. (NFR-012.)
- **Test:** Load test: 1000 msg/sec for 30s, measure RSS

### AC-004 (traces to BC-4.09.003 — eviction warning)
When eviction occurs, emits `E-CAP-001` warning once per 10% capacity threshold crossed: "Capture buffer at <pct>% capacity".
- **Test:** `test_BC_4_09_003_eviction_warning_emitted()`

### AC-005 (traces to BC-4.09.003 — O(1) append)
Append operation is O(1). No shifting or copying on eviction. (Part of AD-006 ring buffer.)
- **Test:** `test_BC_4_09_003_append_is_constant_time()` — measure 1M appends, assert O(1)

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `RingBuffer<MessageCaptured>` | `forge-traffic/src/buffer.rs` | Pure |
| Memory accounting | `forge-traffic/src/buffer.rs` | Pure |
| Eviction warning emission | `forge-traffic/src/buffer.rs` | Mixed |

## UX Screens
- SCR-004 (Traffic Inspector) — shows message count in buffer

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Buffer capacity = 0 | All messages immediately evicted (valid config for headless mode) |
| EC-002 | Single massive message (>100MB) | Not stored, logged as oversized |
| EC-003 | Rapid fill then drain | No double-eviction issues |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| RingBuffer | Pure | Data structure: no I/O in push/pop |
| Memory accounting | Pure | Size counter math |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~900 |
| BC-4.09.003 | ~500 |
| AD-006 ring buffer | ~300 |
| **Total** | **~1,700** |
| Agent context window | 200K |
| **Budget usage** | **~0.9%** |

## Tasks

1. [ ] Write failing tests for all 5 ACs
2. [ ] Implement `RingBuffer<T>` generic data structure
3. [ ] Implement memory accounting (estimate message size)
4. [ ] Implement FIFO eviction with O(1) append
5. [ ] Implement eviction warning thresholds
6. [ ] Write Kani proof for VP-005 (bounds + FIFO)
7. [ ] Run load test for NFR-012
8. [ ] Verify Red Gate
9. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-027 | MessageCaptured is the message type | RingBuffer<MessageCaptured> | Message size estimation needed for memory bound |
| STORY-028 | TimedMessage wraps MessageCaptured | Buffer stores raw MessageCaptured | Disk spill deferred (not in v0.1.0) |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Ring buffer for bounded memory (AD-006, NFR-012) | ARCH-INDEX.md | Use ring buffer, not Vec |
| Capture bounded < 100MB (NFR-012) | nfr-catalog.md | Memory accounting enforced |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| kani | dev | VP-005 bounds proof | `#[kani::proof]` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-traffic/src/buffer.rs` | RingBuffer impl | NO — this story creates it |
| `crates/forge-traffic/proofs/ring_buffer.rs` | Kani VP-005 proof | NO |
| `tests/perf/capture_load_test.rs` | NFR-012 load test | NO |
