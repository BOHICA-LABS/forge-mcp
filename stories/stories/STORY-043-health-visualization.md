---
document_type: story
story_id: STORY-043
epic_id: EPIC-03
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-037, STORY-036]
blocks: [STORY-046]
behavioral_contracts: [BC-3.08.002]
verification_properties: []
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-043: Sparkline & Histogram Health Metric Visualization

## Narrative
- **As an** AI Platform Engineer
- **I want to** see sparklines and histograms of latency and throughput in the TUI
- **So that** I can spot trends and anomalies at a glance

## Acceptance Criteria

### AC-001 (traces to BC-3.08.002 postcondition — sparkline rendering)
`SparklineWidget` renders a sparkline (▁▂▃▄▅▆▇█) showing the last N data points of a metric time series. Height is 3 rows minimum.
- **Test:** `test_BC_3_08_002_sparkline_rendering()`

### AC-002 (traces to BC-3.08.002 postcondition — histogram rendering)
`HistogramWidget` renders a bar chart showing latency distribution buckets. x-axis: latency ranges, y-axis: count.
- **Test:** `test_BC_3_08_002_histogram_rendering()`

### AC-003 (traces to BC-3.08.002 postcondition — alert threshold line)
When an alert threshold is configured, a dashed line marker is shown on the sparkline at the threshold value. Color: green below threshold, red above.
- **Test:** `test_BC_3_08_002_alert_threshold_line()`

### AC-004 (traces to BC-3.08.002 — 1Hz update)
Sparklines update at 1Hz. A 1-second tokio interval drives metric refresh. Frame rate is not blocked by metric refresh.
- **Test:** `test_BC_3_08_002_1hz_update_rate()`

### AC-005 (traces to BC-3.08.002 — no color-only)
Alert threshold breach is shown with color change AND `[ALERT]` text indicator. Not color-only. (BC-3.08.005, NFR-015.)
- **Test:** `test_BC_3_08_002_alert_not_color_only()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `SparklineWidget` | `forge-tui/src/widgets/sparkline.rs` | Pure |
| `HistogramWidget` | `forge-tui/src/widgets/histogram.rs` | Pure |

## UX Screens
- SCR-005 (Health Metrics Panel)

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | All data points = 0 | Flat sparkline (all ▁) |
| EC-002 | Single data point | Single bar sparkline |
| EC-003 | Very large latency outlier | Scale sparkline to max, others squished |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| SparklineWidget | Pure | Renders to Frame from data |
| HistogramWidget | Pure | Same |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~800 |
| BC-3.08.002 | ~400 |
| UX-INDEX.md sparkline widget | ~400 |
| **Total** | **~1,600** |
| Agent context window | 200K |
| **Budget usage** | **~0.8%** |

## Tasks

1. [ ] Write failing tests for all 5 ACs
2. [ ] Implement SparklineWidget with 8 block chars
3. [ ] Implement HistogramWidget
4. [ ] Implement threshold line marker
5. [ ] Implement 1Hz update interval
6. [ ] Verify accessibility (ALERT text indicator)
7. [ ] Verify Red Gate
8. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-036 | AlertStateMachine produces alert state | Pass to sparkline for coloring | Sparkline width must adapt to pane width |
| STORY-033 | MetricSnapshot has time-series data | Feed to sparkline data points | |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| No color-only indicators (NFR-015) | nfr-catalog.md | [ALERT] text required |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| ratatui | >= 0.28 | Sparkline, BarChart widgets | `ratatui::widgets::Sparkline` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-tui/src/widgets/sparkline.rs` | SparklineWidget | NO — this story creates it |
| `crates/forge-tui/src/widgets/histogram.rs` | HistogramWidget | NO |
