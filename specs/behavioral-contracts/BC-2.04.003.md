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

# BC-2.04.003 — Graceful Degradation with Older Spec Versions

## Summary

Detects the server's MCP protocol version from the initialize response and gracefully disables capabilities that are not available in older spec versions. Reports degradation clearly so users understand why certain features are unavailable with a given server.

## Preconditions

- PRE-001: The initialize handshake has completed successfully (BC-2.04.001).
- PRE-002: The server's `protocolVersion` field is present in the `InitializeResult`.

## Postconditions

- POST-001: The server's protocol version is parsed and compared against the client's known version table:
  - `2025-03-26` (latest at time of writing): full feature set
  - `2024-11-05` (previous stable): no tasks, no elicitation, no extensions, no completions with references
  - Unknown future versions: treated as latest known + unknown extras (forward-compatible)
- POST-002: For servers on older protocol versions, the `NegotiatedCapabilities` record is further restricted:
  - Features not in the older spec are removed from negotiated capabilities, even if the server advertised them.
- POST-003: A degradation report is emitted via `E-CON-006` containing:
  - Server name and version
  - Server protocol version
  - List of disabled features with reason ("not available in protocol version X")
- POST-004: The degradation report is displayed in the TUI connection panel and in CLI `--verbose` output.
- POST-005: For unknown (newer) protocol versions, all server-advertised capabilities are accepted. No degradation applied.

## Invariants

- **DI-001**: Even with degradation, the initialize/initialized handshake is fully completed.

## Edge Cases

| ID | Condition | Expected Behavior |
|----|-----------|-------------------|
| EC-001 | Server reports protocol version `2024-11-05` but advertises `elicitation` capability | `elicitation` is removed from negotiated capabilities (not in 2024-11-05 spec). Warning in degradation report: "Server advertises elicitation but protocol version 2024-11-05 does not support it." |
| EC-002 | Server reports unknown future version `2026-06-01` | Accept all advertised capabilities. No degradation. Log info: "Server uses protocol version 2026-06-01 (newer than client's known versions)." |
| EC-003 | Server omits `protocolVersion` field entirely | Treat as oldest known version (most restrictive). Log warning. |
| EC-004 | Server reports an invalid version string (not a date) | Log warning `E-CON-013: Invalid protocol version format: <value>`. Treat as oldest known version. |
| EC-005 | Server on old protocol but all its capabilities are compatible with that version | Negotiation succeeds with no degradation. No E-CON-006 emitted. |

## Canonical Test Vectors

### Happy Path

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-001 | Server: `protocolVersion: "2025-03-26"`, full capabilities | No degradation. All capabilities available. |
| TV-002 | Server: `protocolVersion: "2024-11-05"`, capabilities: `{tools: {}, resources: {}}` | Capabilities accepted as-is (tools and resources existed in 2024-11-05). No degradation. |

### Edge Case

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-003 | Server: `protocolVersion: "2024-11-05"`, capabilities: `{tools: {}, elicitation: {}}` | `elicitation` removed from negotiated caps. Degradation report: "elicitation disabled (not in 2024-11-05)". |
| TV-004 | Server: `protocolVersion: "2026-06-01"`, capabilities: `{tools: {}, newFeature: {}}` | All capabilities accepted including `newFeature`. |

### Error

| ID | Input | Expected Output |
|----|-------|-----------------|
| TV-005 | Server: `protocolVersion` field missing | Treat as oldest known version. Warning logged. Maximum degradation applied. |
| TV-006 | Server: `protocolVersion: "not-a-date"` | Warning `E-CON-013`. Treat as oldest. |

## Verification Properties

| ID | Property | Method |
|----|----------|--------|
| VP-001 | Old protocol version never enables capabilities absent from that spec | Unit test: version 2024-11-05 → verify elicitation, tasks, extensions are gated |
| VP-002 | Unknown future versions never degrade capabilities | Unit test: version 2099-01-01 → verify all server caps accepted |
| VP-003 | Degradation report includes all disabled features | Integration test: compare report against expected disabled list |

## Traceability

- **L2 Capability**: CAP-004 (Capability Negotiation)
- **Domain Invariant**: DI-001
- **Priority**: P1
