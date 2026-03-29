---
document_type: story
story_id: STORY-017
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
blocks: [STORY-045]
behavioral_contracts: [BC-2.05.002]
verification_properties: [VP-001, VP-003]
priority: P0
assumption_validations: []
risk_mitigations: []
---

# STORY-017: Resource List, Read & Subscription Management

## Narrative
- **As an** AI Platform Engineer
- **I want to** list, read, and subscribe to MCP server resources
- **So that** I can access file-like data from servers and receive real-time updates

## Acceptance Criteria

### AC-001 (traces to BC-2.05.002 postcondition — resource list)
`connection.list_resources()` returns `Vec<Resource>` with pagination support (same pattern as STORY-016). Each resource has `uri: String`, `name: String`, `description: Option<String>`, `mimeType: Option<String>`.
- **Test:** `test_BC_2_05_002_list_resources_paginated()`

### AC-002 (traces to BC-2.05.002 postcondition — resource read)
`connection.read_resource(uri)` returns `ResourceContent` with `uri`, `mimeType`, and `content` (text or blob). Binary (blob) content is base64-decoded to bytes.
- **Test:** `test_BC_2_05_002_read_resource()`

### AC-003 (traces to BC-2.05.002 postcondition — subscribe)
`connection.subscribe_resource(uri)` sends `resources/subscribe` and starts receiving `notifications/resources/updated` for that URI. Returns a subscription handle.
- **Test:** `test_BC_2_05_002_subscribe_resource()`

### AC-004 (traces to BC-2.05.002 postcondition — unsubscribe)
Dropping the subscription handle or calling `subscription.unsubscribe()` sends `resources/unsubscribe` to the server.
- **Test:** `test_BC_2_05_002_unsubscribe_resource()`

### AC-005 (traces to BC-2.05.002 — capability guard)
If server didn't advertise `resources` capability, `list_resources()` returns `Err(E-PRO-003)`.
- **Test:** `test_BC_2_05_002_capability_guard()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `list_resources()`, `read_resource()` | `forge-core/src/protocol.rs` | Effectful |
| Subscription management | `forge-core/src/subscriptions.rs` | Effectful |

## UX Screens
- SCR-003 (Capability Browser, Resources tab)

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Read non-existent resource | Err from server propagated |
| EC-002 | Subscribe to resource URI with special chars | URI passed as-is, no encoding |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `list_resources()`, `read_resource()` | Effectful | RPC calls |
| Subscription handle | Effectful | Manages active subscription |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~800 |
| BC-2.05.002 | ~500 |
| **Total** | **~1,300** |
| Agent context window | 200K |
| **Budget usage** | **~0.7%** |

## Tasks

1. [ ] Write failing tests for all 5 ACs
2. [ ] Implement `list_resources()` (reuse pagination from STORY-016)
3. [ ] Implement `read_resource()` with blob decoding
4. [ ] Implement `subscribe_resource()` + subscription handle
5. [ ] Implement `unsubscribe()` via Drop or explicit call
6. [ ] Verify Red Gate
7. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-016 | Pagination state machine reusable | Import from pagination module | Subscriptions need event channel |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Capability guard before API call | STORY-013 | check server_capabilities().resources |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| base64 | >= 0.21 | Blob content decoding | `base64::decode` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-core/src/protocol.rs` | list_resources(), read_resource() added | YES (from STORY-016) |
| `crates/forge-core/src/subscriptions.rs` | Subscription handle | NO — this story creates it |
