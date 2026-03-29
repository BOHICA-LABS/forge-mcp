---
document_type: story
story_id: STORY-034
epic_id: EPIC-06
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-033]
blocks: []
behavioral_contracts: [BC-6.13.002]
verification_properties: [VP-008]
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-034: Error Rate Trend Tracking

## Narrative
- **As an** AI Platform Engineer
- **I want to** see separate error rates for protocol errors and tool errors
- **So that** I can distinguish server bugs from tool-level failures

## Acceptance Criteria

### AC-001 (traces to BC-6.13.002 postcondition — error rate tracking)
`ErrorRateCollector` tracks two separate error rate windows: `protocol_error_rate` (E-PRO-001 class messages) and `tool_error_rate` (isError=true results). Both are expressed as errors/minute over a 5-minute sliding window.
- **Test:** `test_BC_6_13_002_separate_error_rates()`

### AC-002 (traces to BC-6.13.002 postcondition — trend direction)
`ErrorTrend { direction: Increasing | Stable | Decreasing, rate_delta: f64 }` is computed by comparing the current 1-minute window against the previous 5-minute average.
- **Test:** `test_BC_6_13_002_trend_direction_computed()`

### AC-003 (traces to BC-6.13.002 — zero baseline)
When no errors are observed, error rates are 0.0. No division-by-zero panics. Trend is Stable.
- **Test:** `test_BC_6_13_002_zero_baseline_no_panic()`

### AC-004 (traces to BC-6.13.002 — E-MON-003 stale metrics)
When no messages are received for > 60s, emits `E-MON-003: no traffic observed — metrics stale since <timestamp>`. Error rates are retained but marked stale.
- **Test:** `test_BC_6_13_002_stale_metrics_warning()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `ErrorRateCollector` | `forge-health/src/error_rate.rs` | Pure core |
| Stale detection | `forge-health/src/error_rate.rs` | Mixed (timer) |

## UX Screens
- SCR-005 (Health Metrics Panel)

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Only tool errors, no protocol errors | protocol_error_rate=0, tool_error_rate>0 |
| EC-002 | 100% error rate | rate = 1.0, no overflow |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| ErrorRateCollector | Pure core | Windowed counter math |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~700 |
| BC-6.13.002 | ~400 |
| **Total** | **~1,100** |
| Agent context window | 200K |
| **Budget usage** | **~0.6%** |

## Tasks

1. [ ] Write failing tests for all 4 ACs
2. [ ] Implement `ErrorRateCollector` with two separate windows
3. [ ] Implement trend direction computation
4. [ ] Implement stale metric detection + E-MON-003 warning
5. [ ] Verify Red Gate
6. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-033 | ThroughputCollector sliding window established | Reuse window pattern | Tool errors use isError flag from BC-2.05.010 |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Tool error vs protocol error distinction | BC-2.05.010 | Separate counters |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| (reuse hdrhistogram pattern from STORY-033) | | | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-health/src/error_rate.rs` | ErrorRateCollector | NO — this story creates it |
