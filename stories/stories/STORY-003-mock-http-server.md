---
document_type: story
story_id: STORY-003
epic_id: EPIC-00
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 2
depends_on: [STORY-001]
blocks: [STORY-008, STORY-057]
behavioral_contracts: []
verification_properties: []
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-003: Mock MCP Server (HTTP) — forge-test-http-server

## Narrative
- **As a** test infrastructure consumer
- **I want to** have a behavioral mock MCP server over Streamable HTTP transport at fidelity L2 (stateful)
- **So that** HTTP-specific transport behaviors (session IDs, SSE streams, reconnection) can be tested deterministically

## Acceptance Criteria

### AC-001: HTTP mock server manages Mcp-Session-Id headers correctly
The mock server assigns a unique `Mcp-Session-Id` to each initialized session and rejects requests with missing or unknown session IDs per the Streamable HTTP spec. (Traces to BC-1.02.002 — HTTP transport establishment.)
- **Test:** `test_http_mock_session_id_management()`

### AC-002: Server-sent events (SSE) stream supported for server-initiated messages
The mock server can push server-initiated messages (progress, list change notifications) via SSE. Tests can subscribe to the SSE stream and assert received messages. (Traces to BC-1.02.002, DTU assessment.)
- **Test:** `test_http_mock_sse_notifications()`

### AC-003: Session loss and re-initialization supported
The mock server can be configured to expire a session (return HTTP 404 or 410) to simulate session loss, enabling reconnection testing. (Traces to BC-1.02.003, E-CON-007.)
- **Test:** `test_http_mock_session_expiry()`

### AC-004: Mock server binds to a random available port
The server binds to port 0 (OS-assigned) and reports its actual port, preventing CI port conflicts. (Practical requirement for test isolation.)
- **Test:** `test_http_mock_random_port()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| HTTP mock server | tests/forge-test-http-server/ | Effectful (HTTP) |
| Session state | tests/forge-test-http-server/src/ | Mixed |

## UX Screens
- N/A — infrastructure story

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Request with expired session ID | Returns 404 with JSON error body |
| EC-002 | Concurrent SSE connections | Each gets independent SSE stream |
| EC-003 | Port already in use | Binds to alternative port |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Session state | Mixed | State is pure; HTTP I/O is effectful |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~700 |
| rmcp HTTP transport API | ~400 |
| HTTP session management | ~300 |
| **Total** | **~1,400** |
| Agent context window | 200K |
| **Budget usage** | **~0.7%** |

## Tasks

1. [ ] Create `tests/forge-test-http-server/` crate
2. [ ] Add to workspace `[workspace]` members
3. [ ] Implement HTTP server using rmcp's Streamable HTTP transport
4. [ ] Implement session ID management per MCP Streamable HTTP spec
5. [ ] Implement SSE endpoint for server-initiated messages
6. [ ] Implement session expiry configuration
7. [ ] Implement random port binding with port reporting
8. [ ] Write integration tests for all ACs

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-001 | Workspace structure | Tests in `tests/` not `crates/` | |
| STORY-002 | MockConfig pattern for configurable behaviors | Use similar pattern for HTTP mock | Subprocess for stdio; in-process for HTTP |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Use rmcp HTTP transport (not custom) | AD-002 | Implement via rmcp Streamable HTTP server API |
| Keep in tests/ not crates/ | DTU assessment | Not part of production binary |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| rmcp | workspace | HTTP transport API | rmcp HTTP server module |
| tokio | >= 1.38 | Async HTTP | `#[tokio::main]` |
| axum or hyper | >= 0.7 | HTTP server framework | Via rmcp or direct |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `tests/forge-test-http-server/Cargo.toml` | Mock HTTP server crate | NO |
| `tests/forge-test-http-server/src/main.rs` | Binary entry | NO |
| `tests/forge-test-http-server/src/server.rs` | HTTP server impl | NO |
