---
document_type: story
story_id: STORY-026
epic_id: EPIC-06
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 3
depends_on: [STORY-023, STORY-033]
blocks: []
behavioral_contracts: [BC-6.15.002]
verification_properties: []
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-026: Metric Snapshot JSON Export via CLI

## Narrative
- **As a** DevOps engineer
- **I want to** export a JSON snapshot of health metrics for a server via CLI
- **So that** I can integrate health data into monitoring dashboards or alerts

## Acceptance Criteria

### AC-001 (traces to BC-6.15.002 postcondition — metric snapshot)
`forge-mcp info <server> --metrics` outputs a JSON snapshot with: `server_name`, `timestamp`, `latency_p50_ms`, `latency_p95_ms`, `latency_p99_ms`, `throughput_rps`, `error_rate_pct`, `alert_state` (normal|breached|recovered).
- **Test:** `test_BC_6_15_002_metric_snapshot_json()`

### AC-002 (traces to BC-6.15.002 — real-time snapshot)
The snapshot reflects the current metric window at the time of the CLI call. The daemon provides the snapshot from its in-memory `forge-health` state.
- **Test:** `test_BC_6_15_002_snapshot_is_current()`

### AC-003 (traces to BC-6.15.002 — no daemon running)
If the daemon is not running or the server has no recorded metrics, returns `{ "error": "no metrics available" }` with exit code 0 (degraded).
- **Test:** `test_BC_6_15_002_no_daemon_graceful()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| Metric snapshot serialization | `forge-health/src/snapshot.rs` | Pure |
| CLI metric export command | `forge-mcp/src/commands/info.rs` | Effectful |

## UX Screens
- N/A (CLI output)

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Server with zero messages | All metrics zero, no error |
| EC-002 | Very high latency values | No overflow, exact values |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Metric snapshot serialization | Pure | Serialize MetricSnapshot to JSON |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~600 |
| BC-6.15.002 | ~300 |
| MetricSnapshot struct (STORY-033) | ~300 |
| **Total** | **~1,200** |
| Agent context window | 200K |
| **Budget usage** | **~0.6%** |

## Tasks

1. [ ] Write failing tests for all 3 ACs
2. [ ] Depends on MetricSnapshot from STORY-033 (run concurrently)
3. [ ] Implement metric snapshot JSON serialization (pure)
4. [ ] Add `--metrics` flag to info subcommand
5. [ ] Verify Red Gate
6. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-024 | JSON on stdout established | Same pattern | |
| STORY-033 | MetricSnapshot defined | Import from forge-health | STORY-033 must ship before this can be fully tested |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Pure snapshot serialization | purity-boundary-map.md | No I/O in serialize function |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| serde | >= 1.0 | MetricSnapshot serialization | `#[derive(Serialize)]` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-health/src/snapshot.rs` | MetricSnapshot serialization | NO — this story creates it (with STORY-033) |
| `crates/forge-mcp/src/commands/info.rs` | --metrics flag | YES (from STORY-023) |
