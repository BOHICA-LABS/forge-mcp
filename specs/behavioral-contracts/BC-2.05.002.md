---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-03-29T11:25:00
phase: 1a
inputs: [domain-spec/L2-INDEX.md]
input-hash: ""
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
subsystem: "MCP Protocol Operations"
capability: "CAP-005"
lifecycle_status: active
introduced: v0.1.0
---

# BC-2.05.002 — Resource List, Read, and Subscription Management

## Summary

Lists all resources from a connected MCP server with pagination, reads individual resource content by URI, and manages subscriptions to resource change notifications. Handles `notifications/resources/updated` and `notifications/resources/list_changed` events.

## Preconditions

- PRE-001: Connection is initialized and `capabilities.resources` is present in negotiated capabilities.
- PRE-002: For `resources/subscribe`: the server's capabilities include `resources.subscribe: true`.

## Postconditions

- POST-001: `resources/list` exhausts all pagination pages (DI-019) and caches the complete resource list.
- POST-002: Each resource in the list contains: `uri`, `name`, `description` (optional), `mimeType` (optional).
- POST-003: `resources/read` sends `{uri}` and returns `ReadResourceResult` containing one or more `ResourceContents` items (text or blob).
- POST-004: `resources/subscribe` sends `{uri}` for a specific resource. The server will send `notifications/resources/updated` when the resource changes.
- POST-005: `resources/unsubscribe` sends `{uri}` to stop receiving updates for that resource.
- POST-006: On `notifications/resources/updated {uri}`, the cached content for that URI is invalidated. Subscribers (TUI panels, CLI watchers) are notified.
- POST-007: On `notifications/resources/list_changed`, the entire resource list cache is invalidated. Next `resources/list` fetches fresh data.

## Invariants

- **DI-019**: All pagination pages MUST be exhausted for resource listing.

## Edge Cases

| ID | Condition | Expected Behavior |
|----|-----------|-------------------|
| EC-001 | Resource URI does not exist on server | Server returns JSON-RPC error. Forge returns `Err(E-PRT-004: Resource not found: <uri>)`. |
| EC-002 | Resource content is binary (blob) | Return as `BlobResourceContents` with base64-encoded data and `mimeType`. |
| EC-003 | Subscribe to resource when server does not support subscriptions | Return `Err(E-CAP-001: Server does not support resource subscriptions)`. Gated by capability check. |
| EC-004 | Resource list_changed received during active subscription | Subscription remains valid. Subscriber is notified that the resource list may have changed (resource may have been removed). |
| EC-005 | Subscribe to same resource twice | Second subscription is a no-op. No duplicate notifications. |
| EC-006 | Server sends `resources/updated` for a resource we are not subscribed to | Notification is ignored. No error. |
| EC-007 | Resource content exceeds 10MB | Accept and return content. Display truncation is a TUI concern, not a protocol concern. |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-001 | Server has 2 resources: `file:///config.json`, `file:///readme.md` | `resources/list` returns 2 entries with URIs and names. |
| TV-002 | `resources/read {uri: "file:///config.json"}` | `ReadResourceResult` with text content of the file. |
| TV-003 | Subscribe to `file:///config.json`, server modifies it | `notifications/resources/updated {uri: "file:///config.json"}` received. Cache invalidated. |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-004 | `resources/read {uri: "file:///nonexistent"}` | `Err(E-PRT-004: Resource not found)` |
| TV-005 | Subscribe when server caps lack `resources.subscribe` | `Err(E-CAP-001)` — not sent to server |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-006 | Server returns malformed resource list (missing `uri` field) | `Err(E-PRT-005: Malformed resource entry: missing uri)` |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | Resource list pagination exhausts all pages | Integration test with multi-page resource list |
| VP-002 | Subscription notification invalidates cache | Integration test: subscribe → update → verify cache invalidated |
| VP-003 | Unsubscribe stops notifications for that resource | Integration test: unsubscribe → update → verify no notification |

## Traceability

- **L2 Capability**: CAP-005 (Core MCP Operations)
- **Domain Invariant**: DI-019
- **Priority**: P0
