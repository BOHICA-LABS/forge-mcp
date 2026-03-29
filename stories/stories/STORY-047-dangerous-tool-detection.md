---
document_type: story
story_id: STORY-047
epic_id: EPIC-07
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-027]
blocks: [STORY-050, STORY-052]
behavioral_contracts: [BC-7.16.001, BC-7.16.004]
verification_properties: [VP-010]
priority: P1
assumption_validations: []
risk_mitigations: [R-004]
---

# STORY-047: Dangerous Tool Pattern Detection

## Narrative
- **As a** Security Team member
- **I want to** have Forge MCP automatically detect dangerous tool patterns in live MCP traffic
- **So that** I can identify servers that execute arbitrary filesystem operations or shell commands

> **SR-002 addressed here:** Concrete canonical security rule examples are defined in AC criteria and the file structure requirements. This gives implementers concrete starting rules to achieve NFR-005 > 80% precision.

## Acceptance Criteria

### AC-001 (traces to BC-7.16.001 postcondition — filesystem pattern detection)
The rule engine detects tools whose description or call arguments contain filesystem operation patterns: `read_file`, `write_file`, `delete_file`, `ls`, `find`, path-like strings matching `/etc/`, `/home/`, `C:\`. Emits Finding with `category: Filesystem`.
- **Test:** `test_BC_7_16_001_filesystem_patterns()`

### AC-002 (traces to BC-7.16.001 postcondition — exec pattern detection)
Detects tools containing exec/shell patterns: `exec`, `eval`, `system`, `subprocess`, `sh -c`, `bash -c`, `cmd.exe`, `powershell`. Emits Finding with `category: Execution`. (SR-002 canonical example.)
- **Test:** `test_BC_7_16_001_execution_patterns()`

### AC-003 (traces to BC-7.16.001 postcondition — network pattern detection)
Detects tools making network calls: `http://`, `https://`, `fetch`, `curl`, `wget`, `socket`. Emits Finding with `category: Network`.
- **Test:** `test_BC_7_16_001_network_patterns()`

### AC-004 (traces to BC-7.16.004 postcondition — severity classification with confidence)
Each Finding has `severity: SecuritySeverity` (Critical/High/Medium/Low) and `confidence: f64 ∈ [0.0, 1.0]`. Exec patterns → Critical/0.9. Filesystem → High/0.85. Network → Medium/0.75. Confidence bounds verified by VP-010 Kani proof.
- **Test:** `test_BC_7_16_004_severity_and_confidence()` (Kani for VP-010)

### AC-005 (traces to BC-7.16.001 — corpus precision ≥ 80%)
Rule set achieves > 80% precision on the test corpus (50 known-good + 50 known-dangerous tools). (NFR-005.)
- **Test:** Corpus test in CI

### AC-006 (traces to BC-7.16.001 — rules analyzable from MessageCaptured)
Rules fire on both tool description (from `tools/list` response) and tool call arguments (from `tools/call` request). Both directions analyzed.
- **Test:** `test_BC_7_16_001_both_directions_analyzed()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `RuleEngine` | `forge-security/src/engine.rs` | Pure |
| Pattern matchers | `forge-security/src/patterns.rs` | Pure |
| `SecurityFinding` | `forge-security/src/finding.rs` | Pure |

## UX Screens
- SCR-007 (Security Audit View)

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Tool description is empty | No finding (nothing to analyze) |
| EC-002 | False positive: "network_status" tool | Low confidence finding, not high |
| EC-003 | Rule matches in nested JSON | Still detected |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| RuleEngine | Pure | Pattern matching on message content |
| Pattern matchers | Pure | Regex/string matching |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~1,100 |
| BC-7.16.001, BC-7.16.004 | ~900 |
| SR-002 (heuristic examples) | ~400 |
| VP-010 | ~300 |
| **Total** | **~2,700** |
| Agent context window | 200K |
| **Budget usage** | **~1.4%** |

## Tasks

1. [ ] Write failing tests for all 6 ACs (including corpus test)
2. [ ] Define `SecurityFinding` and `SecuritySeverity` types
3. [ ] Implement `RuleEngine` with pattern matching
4. [ ] Define initial rule set (filesystem, exec, network patterns per SR-002)
5. [ ] Implement `confidence_score()` with bounds [0.0, 1.0]
6. [ ] Write Kani proof for VP-010 confidence bounds
7. [ ] Build test corpus (50 good + 50 bad tool descriptions)
8. [ ] Run corpus precision test (must be > 80%)
9. [ ] Verify Red Gate
10. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-027 | MessageCaptured contains method + payload | Analyze payload on tools/list response | Rule engine L2: depends on forge-core + forge-traffic |
| STORY-029 | RingBuffer holds captured messages | Security subscribes to live events | |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| forge-security L2 (depends on forge-core + forge-traffic) | dependency-graph.md | |
| Rule engine is pure core | purity-boundary-map.md | No I/O in engine.analyze() |
| Confidence ∈ [0.0, 1.0] (DI-010, DI-011) | module-decomposition.md | VP-010 Kani proof |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| regex | >= 1.10 | Pattern matching | `regex::Regex` |
| kani | dev | VP-010 proof | `#[kani::proof]` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-security/src/engine.rs` | RuleEngine | NO — this story creates it |
| `crates/forge-security/src/patterns.rs` | Pattern matchers | NO |
| `crates/forge-security/src/finding.rs` | SecurityFinding, SecuritySeverity | NO |
| `crates/forge-security/proofs/confidence.rs` | VP-010 Kani proof | NO |
| `tests/corpus/security_corpus.json` | 100-entry test corpus (SR-002) | NO |
