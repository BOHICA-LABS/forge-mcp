---
document_type: story
story_id: STORY-002
epic_id: EPIC-00
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 3
depends_on: [STORY-001]
blocks: [STORY-007, STORY-013, STORY-016, STORY-017, STORY-018, STORY-019, STORY-020, STORY-022, STORY-027, STORY-055]
behavioral_contracts: []
verification_properties: []
priority: P0
assumption_validations: []
risk_mitigations: [R-003]
---

# STORY-002: Mock MCP Server (stdio) — forge-test-server

## Narrative
- **As a** test infrastructure consumer
- **I want to** have a behavioral mock MCP server over stdio transport at fidelity L3 (behavioral)
- **So that** integration tests for protocol operations, traffic capture, conformance, and security can execute deterministically without live servers

## Acceptance Criteria

### AC-001: forge-test-server implements all 25 MCP methods via rmcp ServerHandler
The mock server responds to: `initialize`, `tools/list`, `tools/call`, `resources/list`, `resources/read`, `resources/subscribe`, `resources/unsubscribe`, `prompts/list`, `prompts/get`, `logging/setLevel`, `completion/complete`, `roots/list`, plus server-initiated: `sampling/createMessage`, `elicitation/create`, `notifications/tools/list_changed`, `notifications/resources/list_changed`, `notifications/progress`, `$/cancel`. (Traces to DTU assessment — MCP Server stdio, fidelity L3.)
- **Test:** `test_mock_server_method_coverage()`

### AC-002: Capability advertisement is configurable per test scenario
The mock server accepts a `MockConfig` struct specifying which capabilities to advertise (tools, resources, prompts, sampling, elicitation, roots, logging). Tests can create a server with any capability subset. (Traces to BC-2.04.001 — capability negotiation.)
- **Test:** `test_mock_config_capability_subsets()`

### AC-003: Pagination behavior is configurable
Tests can configure the mock to return N tools/resources/prompts per page with a configurable cursor pattern, including intentional cursor loops for loop detection testing. (Traces to BC-2.05.001, BC-2.05.003, NFR-013.)
- **Test:** `test_mock_pagination_configurable()`

### AC-004: Schema drift simulation supported
The mock server supports a `drift_after_n_calls(n, schema_change)` configuration that changes tool descriptions after N invocations, enabling rug pull detection tests. (Traces to BC-7.16.003, DTU assessment.)
- **Test:** `test_mock_schema_drift_simulation()`

### AC-005: Error injection supported
The mock server supports injecting: timeout (no response), crash (process exit), malformed JSON-RPC response, and `isError: true` tool results. (Traces to BC-1.02.003, BC-2.05.010, DTU assessment.)
- **Test:** `test_mock_error_injection()`

### AC-006: Server runs as subprocess conforming to stdio MCP transport
The mock binary is spawned as a child process via `std::process::Command`. It communicates via stdin/stdout with newline-delimited JSON-RPC. (Traces to BC-1.02.001.)
- **Test:** `test_mock_server_spawns_as_subprocess()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| MockConfig struct | tests/forge-test-server/src/ | Pure |
| ServerHandler impl | tests/forge-test-server/src/ | Effectful (I/O) |
| Subprocess harness | integration tests | Effectful (process) |

## UX Screens
- N/A — infrastructure story

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Test requests capability not in MockConfig | Server returns JSON-RPC method-not-found (-32601) |
| EC-002 | Pagination loop injection | Server returns same cursor indefinitely; client must detect via NFR-013 |
| EC-003 | Schema drift after 0 calls | Drift active from first call |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| MockConfig | Pure | Configuration struct |
| ServerHandler impl | Effectful | stdin/stdout I/O |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~900 |
| rmcp ServerHandler trait | ~500 |
| MockConfig struct design | ~300 |
| Test harness code | ~500 |
| **Total** | **~2,200** |
| Agent context window | 200K |
| **Budget usage** | **~1.1%** |

## Tasks

1. [ ] Create `tests/forge-test-server/` crate
2. [ ] Add to workspace `[workspace]` members
3. [ ] Implement `MockConfig` struct with all configurable behaviors
4. [ ] Implement `rmcp::ServerHandler` for all 25 MCP methods
5. [ ] Implement pagination configuration (page size, cursor loops)
6. [ ] Implement schema drift simulation (`drift_after_n_calls`)
7. [ ] Implement error injection (timeout, crash, malformed, isError)
8. [ ] Write `main.rs` that parses `MockConfig` from env vars or stdin JSON
9. [ ] Write integration tests for all AC criteria
10. [ ] Verify subprocess spawn works in CI

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-001 | rmcp in forge-core only; Cargo workspace structure | Use workspace crate paths | Tests crate is in `tests/`, not `crates/` |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Use rmcp server API (not custom framing) | AD-002 | Implement `rmcp::ServerHandler` trait |
| Mock server NOT part of production binary | ARCH-INDEX.md | Keep in `tests/` directory, not `crates/` |
| Mock uses same protocol as production | DTU assessment | rmcp ensures protocol compatibility |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| rmcp | workspace | Must match forge-core rmcp version | `use rmcp::ServerHandler;` |
| tokio | >= 1.38 | Async subprocess I/O | `#[tokio::main]` |
| serde_json | >= 1.0 | MockConfig JSON parsing | `serde_json::from_str` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `tests/forge-test-server/Cargo.toml` | Mock stdio server crate | NO — this story creates it |
| `tests/forge-test-server/src/main.rs` | Binary entry | NO |
| `tests/forge-test-server/src/config.rs` | MockConfig struct | NO |
| `tests/forge-test-server/src/handler.rs` | ServerHandler impl | NO |
| `tests/integration/mock_server_tests.rs` | Integration tests | NO |
