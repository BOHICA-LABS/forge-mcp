---
document_type: story
story_id: STORY-014
epic_id: EPIC-02
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-013]
blocks: [STORY-019, STORY-020, STORY-021]
behavioral_contracts: [BC-2.04.002]
verification_properties: [VP-001]
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-014: Client Capability Advertisement (Sampling, Elicitation, Roots)

## Narrative
- **As an** MCP server author
- **I want to** have Forge MCP correctly advertise its client capabilities
- **So that** servers can invoke sampling, elicitation, and roots operations on the client side

## Acceptance Criteria

### AC-001 (traces to BC-2.04.002 postcondition — sampling capability)
The `initialize` request advertises `sampling: {}` capability. The `ClientHandler` implements `create_message` and routes it to the configured LLM proxy endpoint. If no endpoint is configured, returns `Err(E-PRO-007)`.
- **Test:** `test_BC_2_04_002_sampling_capability_advertised()`

### AC-002 (traces to BC-2.04.002 postcondition — elicitation capability)
The `initialize` request advertises `elicitation: {}` capability. The `ClientHandler` implements `elicit_user_data` which opens the appropriate interaction mode (form or URL). In non-interactive CLI mode, returns `Err(E-PRO-008)` per DEC-017.
- **Test:** `test_BC_2_04_002_elicitation_capability_advertised()`

### AC-003 (traces to BC-2.04.002 postcondition — roots capability)
The `initialize` request advertises `roots: { listChanged: true }`. The `ClientHandler` implements `list_roots` returning configured root paths. Root change notifications are emitted when roots change.
- **Test:** `test_BC_2_04_002_roots_capability_advertised()`

### AC-004 (traces to BC-2.04.002 — capability selective advertisement)
Client capabilities are NOT hard-coded. They are determined at runtime based on configuration: if no LLM proxy is configured, `sampling` is NOT advertised. This prevents servers from sending sampling requests that can't be handled.
- **Test:** `test_BC_2_04_002_sampling_not_advertised_without_config()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `ClientHandler` impl | `forge-core/src/handler.rs` | Effectful |
| `ClientCapabilities` builder | `forge-core/src/connection.rs` | Pure |

## UX Screens
- N/A — protocol story

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Sampling request in non-interactive TUI | Routes to LLM proxy (TUI can show dialog) |
| EC-002 | Elicitation in CLI non-interactive mode | Err(E-PRO-008) immediately |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Capability builder | Pure | Conditional logic on config |
| ClientHandler callbacks | Effectful | LLM API calls, user prompts |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~800 |
| BC-2.04.002 | ~500 |
| rmcp ClientHandler trait | ~400 |
| **Total** | **~1,700** |
| Agent context window | 200K |
| **Budget usage** | **~0.9%** |

## Tasks

1. [ ] Write failing tests for all 4 ACs
2. [ ] Implement `ClientHandler` trait impl in forge-core
3. [ ] Implement `create_message` handler (sampling proxy stub)
4. [ ] Implement `elicit_user_data` handler
5. [ ] Implement `list_roots` handler
6. [ ] Add runtime capability enablement logic
7. [ ] Verify Red Gate
8. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-013 | initialize() uses ClientCapabilitiesBuilder | Capabilities built once at connection | rmcp ClientHandler is a separate trait from protocol ops |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| ClientHandler in forge-core (DI-017) | module-decomposition.md | Not in forge-daemon |
| Capabilities based on runtime config | BC-2.04.002 | No hard-coded capability set |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| rmcp | workspace | ClientHandler trait | `impl rmcp::ClientHandler for ForgeClientHandler` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-core/src/handler.rs` | ClientHandler impl | NO — this story creates it |
| `crates/forge-core/src/connection.rs` | Capability builder updated | YES (from STORY-013) |
