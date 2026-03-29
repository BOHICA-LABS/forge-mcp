---
document_type: story
story_id: STORY-008
epic_id: EPIC-01
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-003, STORY-004]
blocks: [STORY-009, STORY-010, STORY-013]
behavioral_contracts: [BC-1.02.002]
verification_properties: [VP-013]
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-008: Streamable HTTP Transport Connection Establishment

## Narrative
- **As an** AI Platform Engineer
- **I want to** connect to an MCP server over Streamable HTTP transport
- **So that** I can inspect remote MCP servers hosted as HTTP services

## Acceptance Criteria

### AC-001 (traces to BC-1.02.002 postcondition — session established)
`connect_http(entry: &ServerEntry) -> Result<McpConnection>` sends an `initialize` request to `HttpConfig.url` via HTTP POST, obtains an `Mcp-Session-Id` header in the response, and returns a live `McpConnection` using that session ID for subsequent requests.
- **Test:** `test_BC_1_02_002_http_connect_success()`

### AC-002 (traces to BC-1.02.002 — custom headers)
`HttpConfig.headers` (e.g., `Authorization: Bearer <token>`) are attached to every HTTP request for this server.
- **Test:** `test_BC_1_02_002_custom_headers_sent()`

### AC-003 (traces to BC-1.02.002 — HTTP auth failure)
When the server returns HTTP 401, returns `Err(E-CON-009: authentication failed — HTTP 401 for <url>)`.
- **Test:** `test_BC_1_02_002_auth_failure_401()`

### AC-004 (traces to BC-1.02.002 — insecure HTTP warning)
When `HttpConfig.url` uses `http://` (not HTTPS), emits warning `E-CON-010` to stderr but proceeds with the connection.
- **Test:** `test_BC_1_02_002_insecure_http_warning()`

### AC-005 (traces to BC-1.02.002 — server unavailable)
When the server returns HTTP 503 or is unreachable, returns `Err(E-CON-011: server unavailable — HTTP <n> for <url>)`.
- **Test:** `test_BC_1_02_002_server_unavailable()`

### AC-006 (traces to BC-1.02.002 — session loss recovery)
When `Mcp-Session-Id` becomes invalid (HTTP 404/410), connection attempts re-initialization once. If re-init succeeds, emits `E-CON-008` (degraded). If re-init fails, returns `Err(E-CON-007)`.
- **Test:** `test_BC_1_02_002_session_loss_reinitialization()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `connect_http()` | `forge-core/src/transport.rs` | Effectful (HTTP) |
| Session management | `forge-core/src/connection.rs` | Effectful |

## UX Screens
- SCR-002 (Server Sidebar) — status badge shows connecting/connected
- FLOW-001 (Server Connection)

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | URL has no scheme | Treated as http://, E-CON-010 warning |
| EC-002 | DNS resolution fails | Err(E-CON-001) |
| EC-003 | Session ID in body instead of header | Protocol error E-PRO-001 |
| EC-004 | TLS certificate error | Err(E-CON-004) |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `connect_http()` | Effectful | Network I/O |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~900 |
| BC-1.02.002 (referenced) | ~600 |
| rmcp HTTP transport API | ~400 |
| **Total** | **~1,900** |
| Agent context window | 200K |
| **Budget usage** | **~1%** |

## Tasks

1. [ ] Write failing tests (uses forge-test-http-server from STORY-003)
2. [ ] Implement `connect_http()` using rmcp Streamable HTTP transport
3. [ ] Implement custom header injection
4. [ ] Implement insecure HTTP warning
5. [ ] Implement session loss recovery logic
6. [ ] Handle auth failures, DNS errors, TLS errors
7. [ ] Verify Red Gate
8. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-007 | McpConnection and connection state machine defined | Reuse McpConnection type | HTTP uses session ID header, not process handle |
| STORY-003 | HTTP mock on random port | Test must get port from mock server | |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| rmcp exclusive for HTTP transport (AD-002) | ARCH-INDEX.md | No custom HTTP client for MCP framing |
| Session ID management per spec | BC-1.02.002 | Must send Mcp-Session-Id on every request |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| rmcp | workspace | Streamable HTTP transport | rmcp HTTP client module |
| reqwest (via rmcp or direct) | >= 0.12 | HTTP client | Likely via rmcp dependency |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-core/src/transport.rs` | connect_http() added | YES (from STORY-007) |
| `crates/forge-core/tests/http_transport_tests.rs` | Integration tests | NO — this story creates it |
