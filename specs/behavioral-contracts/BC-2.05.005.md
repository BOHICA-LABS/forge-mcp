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

# BC-2.05.005 — Elicitation Request Handling

## Summary

Handles `elicitation/create` requests from MCP servers per MCP 2025-11-25 elicitation spec. In TUI mode, renders a form (form mode) or presents a URL for user consent (URL mode). In CLI mode, behavior depends on the `--non-interactive` flag: when set (or when no TTY is detected), form-mode requests are declined with `action: "decline"`; otherwise CLI prompts on stdin for simple text inputs. Handles timeouts for unresponsive users. Uses the MCP-canonical three-action response model: `accept`, `decline`, `cancel`.

## Preconditions

- PRE-001: The client advertised `elicitation` capability during negotiation (BC-2.04.002).
- PRE-002: The elicitation handler is registered with rmcp.
- PRE-003: An `elicitation/create` request is received from the server.

## Postconditions

- POST-001: In TUI mode — form elicitation:
  - The form schema from the request is rendered as interactive form fields in the TUI.
  - User input is collected and validated against the schema.
  - The response is sent back to the server with `action: "submit"` and the form data.
- POST-002: In TUI mode — URL elicitation:
  - The URL is displayed to the user with instructions to open it.
  - The system waits for a callback or user confirmation.
- POST-003: In CLI mode (non-interactive):
  - The request is rejected with `action: "decline"` and reason `"Non-interactive mode"`.
  - Warning emitted: `E-ELC-001: Elicitation request declined (non-interactive CLI mode)`.
- POST-004: User can cancel the elicitation, sending `action: "cancel"` to the server.
- POST-005: Elicitation timeout (configurable, default 300s): if the user does not respond within the timeout, send `action: "cancel"` with reason `"Timeout"`.

## Invariants

- None specific beyond DI-001 (connection must be initialized).

## Edge Cases

| ID | Condition | Expected Behavior |
|----|-----------|-------------------|
| EC-001 | DEC-017 / FM-017: Elicitation in CLI mode | Immediately decline with `action: "decline"`. No user prompt. |
| EC-002 | FM-017: User does not respond within timeout (300s) | Auto-cancel: `action: "cancel"`, reason: "User did not respond within 300s". |
| EC-003 | Form schema contains unknown field types | Render as text input (best-effort). Log warning `E-ELC-002: Unknown form field type: <type>`. |
| EC-004 | Form validation fails (user input does not match schema) | Display validation error in TUI. Do not send to server. Allow user to correct. |
| EC-005 | Elicitation request arrives while another is pending | Queue the second request. Display to user after first completes. Server handles concurrent elicitation via request IDs. |
| EC-006 | TUI is in background (not focused) | Elicitation waits. Notification badge shown. Timeout still applies. |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-001 | TUI mode, form with `{name: {type: "string", description: "Your name"}}` | Form rendered, user types "Alice", response: `{action: "submit", data: {name: "Alice"}}` |
| TV-002 | TUI mode, user clicks cancel | Response: `{action: "cancel"}` |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-003 | CLI mode, any elicitation request | Response: `{action: "decline", reason: "Non-interactive mode"}`. Warning `E-ELC-001`. |
| TV-004 | TUI mode, 300s timeout elapses | Response: `{action: "cancel", reason: "Timeout"}` |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-005 | Malformed elicitation request (missing schema) | MCP error: `{code: -32602, message: "Invalid elicitation request: missing schema"}` |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | CLI mode never prompts user (always declines immediately) | Unit test: CLI config → verify decline sent without user interaction |
| VP-002 | Timeout triggers auto-cancel after configured duration | Integration test with short timeout (1s), verify cancel sent |
| VP-003 | Form validation prevents invalid data from being sent to server | Unit test: schema with constraints, invalid input, verify no submit |

## Traceability

- **L2 Capability**: CAP-005 (Core MCP Operations)
- **Edge Cases**: DEC-017
- **Failure Modes**: FM-017
- **Priority**: P1
