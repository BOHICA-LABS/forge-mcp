---
document_type: story
story_id: STORY-030
epic_id: EPIC-04
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-029]
blocks: [STORY-026]
behavioral_contracts: [BC-4.10.001]
verification_properties: [VP-014]
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-030: Traffic Filtering by Method, Direction, Time & Content

## Narrative
- **As an** AI Platform Engineer
- **I want to** filter the traffic capture by method name, direction, time range, and content
- **So that** I can focus on specific message types during debugging

## Acceptance Criteria

### AC-001 (traces to BC-4.10.001 postcondition — method filter)
`filter_messages(messages, FilterSpec { method: Some("tools/call"), ..}) ` returns only messages where `method == "tools/call"`. Supports wildcard: `"tools/*"` matches all tools methods.
- **Test:** `test_BC_4_10_001_filter_by_method()`

### AC-002 (traces to BC-4.10.001 postcondition — direction filter)
`FilterSpec { direction: ClientToServer }` returns only client-initiated messages.
- **Test:** `test_BC_4_10_001_filter_by_direction()`

### AC-003 (traces to BC-4.10.001 postcondition — time range filter)
`FilterSpec { after: Some(t1), before: Some(t2) }` returns only messages with `timestamp` in `[t1, t2]`.
- **Test:** `test_BC_4_10_001_filter_by_time_range()`

### AC-004 (traces to BC-4.10.001 postcondition — content filter)
`FilterSpec { content: Some("\"isError\":true") }` returns messages whose serialized payload contains the substring.
- **Test:** `test_BC_4_10_001_filter_by_content()`

### AC-005 (traces to BC-4.10.001 — filter preserves ordering)
The filtered result maintains the same temporal ordering as the input buffer. No reordering of matching messages. (VP-014.)
- **Test:** `test_BC_4_10_001_filter_preserves_order()` (proptest for VP-014)

### AC-006 (traces to BC-4.10.001 — combined filter)
Multiple filter dimensions are ANDed. `{ method: "tools/call", direction: ClientToServer }` matches only client-originated tool calls.
- **Test:** `test_BC_4_10_001_combined_filter()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `filter_messages()` | `forge-traffic/src/filter.rs` | Pure |
| `FilterSpec` | `forge-traffic/src/filter.rs` | Pure |

## UX Screens
- SCR-004 (Traffic Inspector) — `/` opens filter dialog using this logic

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | No messages match filter | Empty Vec returned, no error |
| EC-002 | Empty filter spec | All messages returned |
| EC-003 | Wildcard method pattern | Matches all methods with that prefix |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| filter_messages() | Pure | Takes Vec, returns Vec, no I/O |
| FilterSpec | Pure | Data structure |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~900 |
| BC-4.10.001 | ~500 |
| VP-014 | ~300 |
| **Total** | **~1,700** |
| Agent context window | 200K |
| **Budget usage** | **~0.9%** |

## Tasks

1. [ ] Write failing tests for all 6 ACs
2. [ ] Define `FilterSpec` struct
3. [ ] Implement `filter_messages()` pure function
4. [ ] Implement method wildcard matching
5. [ ] Add proptest for VP-014 (ordering preserved)
6. [ ] Verify Red Gate
7. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-029 | Buffer is RingBuffer<MessageCaptured> | Filter takes snapshot as Vec | Wildcard matching needs simple glob, not regex |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Filter is pure function | purity-boundary-map.md | No I/O in filter_messages |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| glob | >= 0.3 | Method wildcard matching | `glob::Pattern::new` |
| proptest | dev | VP-014 ordering | `proptest::proptest!` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-traffic/src/filter.rs` | FilterSpec + filter_messages() | NO — this story creates it |
| `crates/forge-traffic/proofs/filter_ordering.rs` | VP-014 proptest | NO |
