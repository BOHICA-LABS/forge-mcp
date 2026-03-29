---
document_type: story
story_id: STORY-035
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
blocks: [STORY-036]
behavioral_contracts: [BC-6.14.001]
verification_properties: [VP-007]
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-035: Configurable Alerting Thresholds

## Narrative
- **As an** AI Platform Engineer
- **I want to** configure alerting thresholds for latency and error rates
- **So that** I'm notified when a server exceeds acceptable performance bounds

## Acceptance Criteria

### AC-001 (traces to BC-6.14.001 postcondition — threshold configuration)
`AlertConfig { latency_p99_threshold_ms: u64, error_rate_threshold_pct: f64, throughput_min_rps: f64 }` is configurable via config file or CLI flags (`--alert-latency`, `--alert-error-rate`, `--alert-throughput`). Defaults: latency=1000ms, error_rate=5%, throughput=0 (disabled).
- **Test:** `test_BC_6_14_001_threshold_config_parsed()`

### AC-002 (traces to BC-6.14.001 postcondition — threshold evaluation)
`evaluate_thresholds(snapshot, config)` returns `Vec<AlertEvent>` for each threshold breached. Returns empty Vec when all metrics within bounds.
- **Test:** `test_BC_6_14_001_threshold_evaluation()`

### AC-003 (traces to BC-6.14.001 — per-server config)
Each server can have its own `AlertConfig` overriding the global defaults. Config in server entry takes precedence.
- **Test:** `test_BC_6_14_001_per_server_threshold()`

### AC-004 (traces to BC-6.14.001 — disabled thresholds)
Setting a threshold to 0 or null disables that specific alert. No false positives when monitoring quiet servers.
- **Test:** `test_BC_6_14_001_disabled_threshold_no_alert()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `AlertConfig` | `forge-health/src/alert.rs` | Pure |
| `evaluate_thresholds()` | `forge-health/src/alert.rs` | Pure |

## UX Screens
- SCR-005 (Health Metrics) — `t` key opens threshold editor

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Both latency and error rate breached | Two AlertEvents in result |
| EC-002 | Threshold exactly at value | No breach (exclusive threshold) |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| evaluate_thresholds() | Pure | Takes snapshot + config, returns events |
| AlertConfig | Pure | Configuration struct |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~700 |
| BC-6.14.001 | ~400 |
| **Total** | **~1,100** |
| Agent context window | 200K |
| **Budget usage** | **~0.6%** |

## Tasks

1. [ ] Write failing tests for all 4 ACs
2. [ ] Define `AlertConfig` struct
3. [ ] Implement `evaluate_thresholds()` pure function
4. [ ] Implement CLI flags for threshold config
5. [ ] Verify Red Gate
6. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-033 | MetricSnapshot defined | evaluate_thresholds takes MetricSnapshot | |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Pure threshold evaluation | purity-boundary-map.md | No I/O in evaluate_thresholds |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| (no new deps) | | | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-health/src/alert.rs` | AlertConfig + evaluate_thresholds() | NO — this story creates it |
