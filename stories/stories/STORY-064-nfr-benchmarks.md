---
document_type: story
story_id: STORY-064
epic_id: SR
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-024, STORY-033, STORY-029]
blocks: []
behavioral_contracts: []
verification_properties: []
priority: P0
assumption_validations: []
risk_mitigations: [R-005]
---

# STORY-064: NFR Validation Suite & Performance Benchmarks

## Narrative
- **As a** developer ensuring release quality
- **I want to** have automated NFR validation tests for all performance and scalability targets
- **So that** CI catches regressions before they reach users

## Acceptance Criteria

### AC-001 (traces to NFR-001 — cold start < 50ms)
`hyperfine --warmup 3 'forge-mcp --help'` measures < 50ms on linux-x64 in CI. Benchmark fails build if threshold exceeded.
- **Test:** CI benchmark step with fail-on-threshold

### AC-002 (traces to NFR-002 — TUI 60fps)
Using `ratatui::backend::TestBackend`, feed 100 synthetic events/sec for 1 second. Assert `draw_count >= 60`. (SR-003 implementation.)
- **Test:** `test_nfr_002_tui_60fps_test_backend()`

### AC-003 (traces to NFR-003 — token efficiency < 500 tokens)
`forge-mcp list <server>` + `forge-mcp call <server> <tool> {}` combined output, tokenized with GPT-4 tokenizer, is ≤ 500 tokens.
- **Test:** `test_nfr_003_token_count_workflow()`

### AC-004 (traces to NFR-009 — binary < 25MB)
CI checks release build binary size on linux-x64. Fails if > 25MB stripped.
- **Test:** CI size check step

### AC-005 (traces to NFR-012 — capture bounded < 100MB)
Load test: 1000 messages/sec for 30 seconds. Assert RSS delta < 100MB.
- **Test:** `test_nfr_012_capture_bounded_memory()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| NFR benchmark suite | `tests/benchmarks/` | Effectful (process + metrics) |
| TUI 60fps test | `crates/forge-tui/tests/fps_test.rs` | Effectful |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Cold start on slow CI | Allow 2x threshold with warning |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| NFR tests | Effectful | Benchmarks and process measurements |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~700 |
| NFR catalog | ~500 |
| **Total** | **~1,200** |
| Agent context window | 200K |
| **Budget usage** | **~0.6%** |

## Tasks

1. [ ] Write all 5 NFR validation tests
2. [ ] Add hyperfine cold start benchmark to CI
3. [ ] Implement TestBackend 60fps test (SR-003)
4. [ ] Add token count integration test
5. [ ] Add binary size CI check
6. [ ] Add memory load test
7. [ ] Verify all tests pass on current codebase

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-037 | TestBackend established for TUI tests | Reuse for 60fps test | CI slow VMs may need threshold tolerance |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| NFR-001, NFR-002, NFR-003, NFR-009, NFR-012 | nfr-catalog.md | All 5 must pass in CI |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| criterion | dev | Rust microbenchmarks | `criterion::Criterion` |
| tiktoken-rs | dev | Token counting | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `tests/benchmarks/cold_start.sh` | hyperfine cold start test | NO |
| `crates/forge-tui/tests/fps_test.rs` | 60fps TestBackend test | NO |
| `tests/benchmarks/token_count.rs` | Token count test | NO |
| `tests/benchmarks/capture_load.rs` | Memory load test | NO |
