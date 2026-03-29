---
document_type: story
story_id: STORY-028
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
blocks: []
behavioral_contracts: [BC-4.09.002]
verification_properties: [VP-006]
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-028: Per-Message Timing & Throughput Analysis

## Narrative
- **As an** AI Platform Engineer inspecting traffic
- **I want to** see per-message latency and throughput statistics
- **So that** I can identify slow tools and performance regressions

## Acceptance Criteria

### AC-001 (traces to BC-4.09.002 postcondition — per-message latency)
For each request/response pair matched by ID, `forge-traffic` computes `latency_ms: f64` as the elapsed time between request `MessageCaptured` and response `MessageCaptured`. Result stored in `TimedMessage`.
- **Test:** `test_BC_4_09_002_per_message_latency()`

### AC-002 (traces to BC-4.09.002 postcondition — throughput)
The traffic analyzer computes throughput as messages/second over a configurable sliding window (default 10s). Updated on each new `MessageCaptured` event.
- **Test:** `test_BC_4_09_002_throughput_calculation()`

### AC-003 (traces to BC-4.09.002 — unmatched responses)
Responses without a matching request ID (late responses, server-initiated) are stored without latency data (`latency_ms: None`).
- **Test:** `test_BC_4_09_002_unmatched_response_no_latency()`

### AC-004 (traces to BC-4.09.002 — DI-006 ordering)
Messages are stored in received-order. Reordered batch responses (E-PRO-009) are flagged in `TimedMessage.reordered: bool`.
- **Test:** `test_BC_4_09_002_message_ordering_preserved()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `TimingAnalyzer` | `forge-traffic/src/timing.rs` | Pure |
| `TimedMessage` | `forge-traffic/src/types.rs` | Pure |
| Throughput window | `forge-traffic/src/throughput.rs` | Pure |

## UX Screens
- SCR-004 (Traffic Inspector) — shows latency column per message

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Notification (no ID) | No latency pairing, direction-only |
| EC-002 | Duplicate response ID | Second match flagged as duplicate |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| TimingAnalyzer | Pure | Pure computation on MessageCaptured events |
| Throughput window | Pure | Sliding window math, no I/O |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~700 |
| BC-4.09.002 | ~400 |
| **Total** | **~1,100** |
| Agent context window | 200K |
| **Budget usage** | **~0.6%** |

## Tasks

1. [ ] Write failing tests for all 4 ACs
2. [ ] Implement `TimedMessage` struct
3. [ ] Implement request-response matching by ID
4. [ ] Implement latency computation
5. [ ] Implement throughput sliding window
6. [ ] Implement ordering validation (DI-006)
7. [ ] Verify Red Gate
8. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-027 | MessageCaptured has timestamp | Use existing timestamps | Notifications don't have IDs |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| forge-traffic L1 (depends only on forge-core) | dependency-graph.md | Import MessageCaptured from forge-core |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| (no new deps) | | | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-traffic/src/timing.rs` | TimingAnalyzer | NO — this story creates it |
| `crates/forge-traffic/src/types.rs` | TimedMessage | NO |
| `crates/forge-traffic/src/throughput.rs` | Throughput window | NO |
