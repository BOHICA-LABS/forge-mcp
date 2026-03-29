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
capability: "CAP-004"
lifecycle_status: active
introduced: v0.1.0
---

# BC-2.04.002 — Client Capability Advertisement

## Summary

Constructs the client's capability advertisement for the initialize handshake. Advertises support for sampling, elicitation, and roots based on runtime configuration. Ensures all advertised capability handlers are registered BEFORE the `notifications/initialized` notification is sent.

## Preconditions

- PRE-001: The Forge runtime configuration is loaded, including: LLM provider settings (for sampling), roots configuration, and elicitation mode (TUI vs CLI).
- PRE-002: Transport connection is established but initialize handshake has not yet been sent.

## Postconditions

- POST-001: The `capabilities` object in the `initialize` request contains:
  - `sampling: {}` — if an LLM provider is configured
  - `elicitation: {}` — if running in TUI mode (interactive)
  - `roots: {listChanged: true}` — if filesystem roots are configured
- POST-002: Capability handlers (sampling handler, elicitation handler, roots handler) are registered with the rmcp client BEFORE `notifications/initialized` is sent.
- POST-003: If sampling is advertised but no LLM provider is actually configured at connection time, a warning is logged: `E-CAP-005: Sampling capability advertised but no LLM provider configured. sampling/createMessage requests will fail.`
- POST-004: The capability advertisement is immutable for the lifetime of the connection. Changing config requires reconnection.

## Invariants

- **DI-016**: Capabilities are explicitly advertised. Forge never implies support for a capability it has not advertised.
- **DI-017**: All handlers for advertised capabilities MUST be registered before `notifications/initialized` is sent. The server may call these handlers immediately after receiving initialized.

## Edge Cases

| ID | Condition | Expected Behavior |
|----|-----------|-------------------|
| EC-001 | DEC-016: Sampling advertised but no LLM provider configured | Warning `E-CAP-005` logged. Sampling is still advertised (to allow configuration hot-reload in future). `sampling/createMessage` will return error per BC-2.05.004. |
| EC-002 | Running in CLI (non-interactive) mode | `elicitation` is NOT advertised. Server cannot send elicitation requests. |
| EC-003 | No roots configured | `roots` is NOT advertised. Server cannot call `roots/list`. |
| EC-004 | All capabilities disabled (no LLM, CLI mode, no roots) | `capabilities` object is empty `{}`. Client is passive — only sends requests, never handles server-initiated calls (except notifications). |
| EC-005 | Handler registration fails (internal error) | Return `Err(E-CAP-006: Failed to register <capability> handler)`. Connection initialization aborts. |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-001 | LLM configured, TUI mode, roots = ["/workspace"] | `capabilities: {sampling: {}, elicitation: {}, roots: {listChanged: true}}` |
| TV-002 | LLM configured, CLI mode, no roots | `capabilities: {sampling: {}}` |
| TV-003 | No LLM, TUI mode, roots = ["/workspace"] | `capabilities: {elicitation: {}, roots: {listChanged: true}}` |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-004 | LLM configured but API key empty string | Warning `E-CAP-005` logged. `sampling` still advertised. |
| TV-005 | No LLM, CLI mode, no roots | `capabilities: {}` — fully passive client |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-006 | Sampling handler registration fails | `Err(E-CAP-006: Failed to register sampling handler)`. Connection aborted. |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | Handlers are registered before initialized notification | Integration test: mock server sends sampling/createMessage immediately after initialized, verify handler responds |
| VP-002 | CLI mode never advertises elicitation | Unit test: CLI config → capabilities object does not contain `elicitation` key |
| VP-003 | Capability advertisement matches runtime config for all config permutations | Property test: random config → verify capabilities object |

## Traceability

- **L2 Capability**: CAP-004 (Capability Negotiation)
- **Domain Invariants**: DI-016, DI-017
- **Edge Cases**: DEC-016
- **Priority**: P0
