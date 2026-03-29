---
document_type: story
story_id: STORY-016
epic_id: EPIC-02
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 8
depends_on: [STORY-013]
blocks: [STORY-022, STORY-045, STORY-061, STORY-063]
behavioral_contracts: [BC-2.05.001]
verification_properties: [VP-001, VP-002, VP-003]
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-016: Tool List & Invocation with Pagination

## Narrative
- **As an** AI agent or developer
- **I want to** list all tools available on an MCP server and invoke them with arguments
- **So that** I can discover and use server capabilities programmatically

## Acceptance Criteria

### AC-001 (traces to BC-2.05.001 postcondition — tool list)
`connection.list_tools()` returns `Vec<Tool>` where each entry has `name: String`, `description: Option<String>`, `inputSchema: JsonSchema`. Supports pagination: iterates through all pages using cursor until `nextCursor` is absent.
- **Test:** `test_BC_2_05_001_list_tools_paginated()`

### AC-002 (traces to BC-2.05.001 — pagination terminates)
The pagination iterator terminates when `nextCursor` is absent. If cursor loops are detected (same cursor seen twice), iteration stops after 100 pages (NFR-013) and emits warning `E-PRO-004`.
- **Test:** `test_BC_2_05_001_pagination_cursor_loop_detected()` (Kani proof sketch for VP-003)

### AC-003 (traces to BC-2.05.001 postcondition — tool invocation)
`connection.call_tool(name, arguments)` sends `tools/call` request and returns `ToolResult { content: Vec<Content>, isError: bool }`. Arguments are validated against `inputSchema` before sending.
- **Test:** `test_BC_2_05_001_tool_invocation_success()`

### AC-004 (traces to BC-2.05.001 — tool error distinction)
When `isError: true` in the response, `call_tool` returns `Ok(ToolResult { isError: true, ... })` — NOT an `Err`. Only protocol-level failures return `Err`. (Traces to BC-2.05.010, DI-020, VP-002.)
- **Test:** `test_BC_2_05_001_tool_error_vs_protocol_error()`

### AC-005 (traces to BC-2.05.001 — schema validation)
Arguments that don't match `inputSchema` (missing required fields, wrong types) return `Err(E-PRO-005)` before sending the request. This saves a round-trip for invalid calls.
- **Test:** `test_BC_2_05_001_argument_schema_validation()`

### AC-006 (traces to BC-2.05.001 — list_changed notification)
When the server sends `notifications/tools/list_changed`, `list_tools()` result is invalidated and refetched. Observable via event subscription.
- **Test:** `test_BC_2_05_001_list_changed_notification()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `list_tools()` | `forge-core/src/protocol.rs` | Effectful (RPC) |
| `call_tool()` | `forge-core/src/protocol.rs` | Effectful (RPC) |
| Pagination state machine | `forge-core/src/pagination.rs` | Pure |
| Error classification | `forge-core/src/error.rs` | Pure (VP-002) |

## UX Screens
- SCR-003 (Capability Browser, Tools tab)
- SCR-006 (Tool Execution Dialog)
- FLOW-002 (Tool Discovery & Execution)

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Server with 0 tools | Empty Vec, no error |
| EC-002 | Tool with no inputSchema | `inputSchema: {}` (accept any args) |
| EC-003 | Cursor loop (same cursor twice) | Stop after 100 pages, E-PRO-004 warning |
| EC-004 | Tool invocation times out | Err with timeout detail |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Pagination state machine | Pure | Cursor + page count, no I/O |
| Error classification | Pure | isError flag check, pure |
| list_tools(), call_tool() | Effectful | rmcp RPC calls |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~1,200 |
| BC-2.05.001 | ~700 |
| BC-2.05.010 (error distinction) | ~400 |
| forge-test-server mock (STORY-002) | ~300 |
| VP-002, VP-003 | ~400 |
| **Total** | **~3,000** |
| Agent context window | 200K |
| **Budget usage** | **~1.5%** |

## Tasks

1. [ ] Write failing tests for all 6 ACs
2. [ ] Implement `list_tools()` with pagination iteration
3. [ ] Implement pagination state machine (pure)
4. [ ] Implement cursor loop detection (NFR-013)
5. [ ] Implement `call_tool()` with argument passing
6. [ ] Implement schema validation before send
7. [ ] Implement error classification (isError vs protocol error)
8. [ ] Add Kani proof for pagination termination (VP-003)
9. [ ] Add Kani proof for error classification (VP-002)
10. [ ] Verify Red Gate
11. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-013 | Capability guard: check server has tools capability | Call guard before list_tools | list_changed notification handling needs event bus |
| STORY-014 | list_changed notification from ClientHandler | Event routing needed | |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Pagination terminates (DI-019, NFR-013) | nfr-catalog.md | Cursor dedup set + page cap |
| Tool error vs protocol error (DI-020) | purity-boundary-map.md | isError in response, not Err() |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| jsonschema | >= 0.17 | inputSchema validation | `jsonschema::validate` |
| kani | dev | VP-002, VP-003 proofs | `#[kani::proof]` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-core/src/protocol.rs` | list_tools(), call_tool() | NO — this story creates it |
| `crates/forge-core/src/pagination.rs` | Pagination state machine | NO — this story creates it |
| `crates/forge-core/src/error.rs` | Error classification | NO — this story creates it |
| `crates/forge-core/proofs/pagination.rs` | VP-003 Kani proof | NO |
| `crates/forge-core/proofs/error_class.rs` | VP-002 Kani proof | NO |
