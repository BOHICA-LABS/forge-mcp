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

# BC-2.05.006 — Roots List Response and Change Notification

## Summary

Responds to `roots/list` requests from MCP servers with the configured filesystem root boundaries. Sends `notifications/roots/list_changed` when the root configuration changes at runtime.

## Preconditions

- PRE-001: The client advertised `roots` capability during negotiation (BC-2.04.002).
- PRE-002: The roots handler is registered with rmcp.
- PRE-003: Filesystem roots are configured (e.g., via CLI `--roots` flag or config file).

## Postconditions

- POST-001: On `roots/list` request, respond with a list of `Root` objects, each containing:
  - `uri`: file URI of the root directory (e.g., `file:///home/user/project`)
  - `name` (optional): human-readable label for the root
- POST-002: If no roots are configured, respond with an empty list `[]`.
- POST-003: When roots are added or removed at runtime (e.g., session configuration change), `notifications/roots/list_changed` is sent to all connected servers that support roots.
- POST-004: Root URIs use the `file://` scheme and are normalized to absolute paths.
- POST-005: Roots represent boundaries the server should be aware of — they do NOT restrict server access (the server is trusted to respect them, but Forge does not enforce).

## Invariants

- None specific. Roots are informational, not security boundaries.

## Edge Cases

| ID | Condition | Expected Behavior |
|----|-----------|-------------------|
| EC-001 | No roots configured | Respond with empty list `[]`. Valid state. Server has no context about filesystem boundaries. |
| EC-002 | Root path does not exist on filesystem | Include in response anyway. The root is a declaration of intent, not a validation of existence. Log warning `E-ROOT-001: Root path does not exist: <path>`. |
| EC-003 | Root path is a file, not a directory | Include with a warning `E-ROOT-002: Root path is a file, not a directory: <path>`. |
| EC-004 | Duplicate root paths configured | Deduplicate. Return each unique path once. |
| EC-005 | Roots changed while server is mid-operation | Notification sent asynchronously. Server may complete current operation with old roots. |
| EC-006 | 100+ roots configured | No hardcoded limit. All roots included in response. |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-001 | Roots configured: `["/home/user/project", "/home/user/shared"]` | `roots/list` response: `{roots: [{uri: "file:///home/user/project", name: "project"}, {uri: "file:///home/user/shared", name: "shared"}]}` |
| TV-002 | Root added at runtime | `notifications/roots/list_changed` sent to all connected servers |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-003 | No roots configured | `roots/list` response: `{roots: []}` |
| TV-004 | Root path `/nonexistent` does not exist | Included in response. Warning `E-ROOT-001` logged. |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-005 | `roots/list` received but client did not advertise roots capability | Should not happen (server should check capabilities). If it does, respond with empty list and log warning. |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | Root URIs are always absolute `file://` URIs | Unit test: relative input → verify absolute URI output |
| VP-002 | Duplicate roots are deduplicated | Unit test: duplicate inputs → verify single output entry |
| VP-003 | `list_changed` notification sent on every root configuration change | Integration test: add root → verify notification sent |

## Traceability

- **L2 Capability**: CAP-005 (Core MCP Operations)
- **Priority**: P1
