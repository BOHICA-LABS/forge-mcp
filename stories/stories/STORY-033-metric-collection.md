---
document_type: story
story_id: STORY-033
epic_id: EPIC-06
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-027]
blocks: [STORY-034, STORY-035, STORY-036, STORY-046, STORY-026]
behavioral_contracts: [BC-6.13.001]
verification_properties: [VP-007, VP-008]
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-033: Passive Latency & Throughput Metric Collection

## Narrative
- **As an** AI Platform Engineer monitoring an MCP server
- **I want to** have latency and throughput metrics automatically collected
- **So that** I can detect performance degradations without manual measurement

## Acceptance Criteria

### AC-001 (traces to BC-6.13.001 postcondition — latency histogram)
`LatencyCollector` subscribes to `MessageCaptured` events from forge-core and maintains a latency histogram (p50, p95, p99) across a sliding window. (Proptest for VP-008.)
- **Test:** `test_BC_6_13_001_latency_histogram_computed()` (proptest for VP-008)

### AC-002 (traces to BC-6.13.001 postcondition — throughput counter)
`ThroughputCollector` maintains a windowed counter of messages/second. Updated on every `MessageCaptured` event.
- **Test:** `test_BC_6_13_001_throughput_counter_updates()`

### AC-003 (traces to BC-6.13.001 — passive only)
Metric collection adds < 1% latency overhead to server operations (NFR-004). No synthetic probes. Collection is a pure computation on received events.
- **Test:** Verified by benchmark (NFR-004)

### AC-004 (traces to BC-6.13.001 — MetricSnapshot)
`MetricSnapshot { server_name, timestamp, latency_p50_ms, latency_p95_ms, latency_p99_ms, throughput_rps, error_rate_pct, alert_state }` is produced on demand from collector state.
- **Test:** `test_BC_6_13_001_metric_snapshot_structure()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `LatencyCollector` | `forge-health/src/latency.rs` | Pure core + effectful timer |
| `ThroughputCollector` | `forge-health/src/throughput.rs` | Pure |
| `MetricSnapshot` | `forge-health/src/snapshot.rs` | Pure |

## UX Screens
- SCR-005 (Health Metrics Panel)

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Zero messages | All metrics zero, no error |
| EC-002 | Single message | Histogram with one data point |
| EC-003 | Very high latency (>60s) | No overflow; exact value stored |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| LatencyCollector (core) | Pure | Histogram math on timestamps |
| ThroughputCollector | Pure | Counter math |
| Timer tick | Effectful | Window rotation |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~900 |
| BC-6.13.001 | ~500 |
| VP-007, VP-008 | ~400 |
| **Total** | **~1,800** |
| Agent context window | 200K |
| **Budget usage** | **~0.9%** |

## Tasks

1. [ ] Write failing tests for all 4 ACs
2. [ ] Implement `LatencyCollector` with histogram (p50/p95/p99)
3. [ ] Implement `ThroughputCollector` with sliding window
4. [ ] Define `MetricSnapshot` struct
5. [ ] Add proptest for VP-008 histogram math correctness
6. [ ] Subscribe to MessageCaptured broadcast channel
7. [ ] Verify Red Gate
8. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-027 | MessageCaptured broadcast channel | Subscribe in forge-health | forge-health L1 depends only on forge-core |
| STORY-028 | TimedMessage has latency_ms | Can reuse latency computation | |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| forge-health L1 (depends only on forge-core) | dependency-graph.md | No import from forge-traffic |
| Passive metrics only (NFR-004) | nfr-catalog.md | No synthetic probes |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| hdrhistogram | >= 7.0 | HDR histogram (p50/p95/p99) | `hdrhistogram::Histogram` |
| proptest | dev | VP-008 | `proptest::proptest!` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-health/src/latency.rs` | LatencyCollector | NO — this story creates it |
| `crates/forge-health/src/throughput.rs` | ThroughputCollector | NO |
| `crates/forge-health/src/snapshot.rs` | MetricSnapshot | NO |
