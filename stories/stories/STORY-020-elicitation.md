---
document_type: story
story_id: STORY-020
epic_id: EPIC-02
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-014]
blocks: []
behavioral_contracts: [BC-2.05.005]
verification_properties: [VP-001]
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-020: Elicitation Request Handling (Form + URL Modes)

## Narrative
- **As an** AI Platform Engineer using the TUI
- **I want to** respond to server elicitation requests interactively
- **So that** I can test servers that need user input before proceeding

## Acceptance Criteria

### AC-001 (traces to BC-2.05.005 postcondition — form mode)
When a server sends `elicitation/create` with a JSON Schema form definition, the TUI renders a form modal. On submission, the client sends back the form data as `ElicitResult`.
- **Test:** `test_BC_2_05_005_form_mode_submission()`

### AC-002 (traces to BC-2.05.005 postcondition — URL mode)
When `elicitation/create` contains a URL, the TUI displays the URL and prompts the user to open it and confirm completion. Returns a completed `ElicitResult` when confirmed.
- **Test:** `test_BC_2_05_005_url_mode_display()`

### AC-003 (traces to BC-2.05.005 — non-interactive rejection)
In CLI non-interactive mode (no TUI), `elicitation/create` returns `Err(E-PRO-008: elicitation request received in non-interactive mode)`. (DEC-017.)
- **Test:** `test_BC_2_05_005_non_interactive_rejects()`

### AC-004 (traces to BC-2.05.005 — cancel)
User pressing `Escape` in the elicitation form returns a cancelled `ElicitResult` to the server.
- **Test:** `test_BC_2_05_005_elicitation_cancel()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `elicit_user_data` handler | `forge-core/src/handler.rs` | Effectful |
| Elicitation TUI form | `forge-tui/src/dialogs/elicitation.rs` | Effectful |

## UX Screens
- SCR-006 (Tool Execution Dialog) — reused for elicitation form

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Elicitation schema has required field | Form validation before submit |
| EC-002 | Multiple concurrent elicitations | Queued, shown one at a time |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `elicit_user_data` | Effectful | User I/O + async |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~700 |
| BC-2.05.005 | ~500 |
| **Total** | **~1,200** |
| Agent context window | 200K |
| **Budget usage** | **~0.6%** |

## Tasks

1. [ ] Write failing tests for all 4 ACs
2. [ ] Implement `elicit_user_data` handler in ClientHandler
3. [ ] Implement CLI non-interactive rejection
4. [ ] Add elicitation form to TUI (stub for now, TUI wired in STORY-037+)
5. [ ] Verify Red Gate
6. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-014 | elicitation capability advertised only in TUI mode | Check interactive flag in handler | |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Non-interactive rejection (DEC-017) | BC-2.05.005 | Error in CLI mode |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| (no new deps) | | | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-core/src/handler.rs` | elicit_user_data impl | YES (from STORY-014) |
| `crates/forge-tui/src/dialogs/elicitation.rs` | Elicitation form widget | NO — this story creates stub |
