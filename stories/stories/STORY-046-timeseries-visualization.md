---
document_type: story
story_id: STORY-046
epic_id: EPIC-06
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-043, STORY-033]
blocks: []
behavioral_contracts: [BC-6.15.001]
verification_properties: []
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-046: Time-Series Metric Visualization in TUI

## Narrative
- **As an** AI Platform Engineer
- **I want to** see a full health metrics panel with real-time updating charts
- **So that** I can monitor server performance trends over time

## Acceptance Criteria

### AC-001 (traces to BC-6.15.001 postcondition — health panel)
SCR-005 (Health Metrics Panel) renders: latency sparkline (last 60s), throughput sparkline (last 60s), error rate sparkline, and a summary row showing current p50/p95/p99 latency values.
- **Test:** `test_BC_6_15_001_health_panel_components()`

### AC-002 (traces to BC-6.15.001 — alert threshold indicators)
When alert state is `Breached`, the relevant metric's sparkline turns red and shows `[ALERT]` text label. Normal state shows green (below threshold) or yellow (approaching). Text label required (BC-3.08.005).
- **Test:** `test_BC_6_15_001_alert_threshold_indicators()`

### AC-003 (traces to BC-6.15.001 — per-server metrics)
Health panel shows metrics for the currently selected server (from server sidebar). Switching servers updates all sparklines to the newly selected server's data.
- **Test:** `test_BC_6_15_001_per_server_metrics()`

### AC-004 (traces to BC-6.15.001 — 1Hz refresh)
All sparklines update at 1Hz. No flicker or full redraw needed; only changed data points cause re-render.
- **Test:** `test_BC_6_15_001_1hz_refresh_rate()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `HealthMetricsPanel` widget | `forge-tui/src/widgets/health_panel.rs` | Pure |
| 1Hz tick source | `forge-tui/src/app.rs` | Effectful |

## UX Screens
- SCR-005 (Health Metrics Panel)
- FLOW-004 (Health Alert)

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Narrow terminal (health panel as tab) | All sparklines in single column |
| EC-002 | No data yet (just connected) | Empty sparklines, "No data" label |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| HealthMetricsPanel | Pure | Renders from MetricSnapshot history |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~700 |
| BC-6.15.001 | ~400 |
| **Total** | **~1,100** |
| Agent context window | 200K |
| **Budget usage** | **~0.6%** |

## Tasks

1. [ ] Write failing tests for all 4 ACs
2. [ ] Implement HealthMetricsPanel widget
3. [ ] Wire to MetricSnapshot time-series from forge-health
4. [ ] Implement per-server switching
5. [ ] Verify accessibility (ALERT text label)
6. [ ] Verify Red Gate
7. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-043 | SparklineWidget established | Compose multiple sparklines | forge-health MetricSnapshot needs ring-buffer history |
| STORY-036 | AlertStateMachine state available | Feed alert state to sparkline coloring | |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Accessibility: text labels (NFR-015) | nfr-catalog.md | [ALERT] required |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| ratatui | >= 0.28 | Layout composition | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-tui/src/widgets/health_panel.rs` | HealthMetricsPanel | NO — this story creates it |
