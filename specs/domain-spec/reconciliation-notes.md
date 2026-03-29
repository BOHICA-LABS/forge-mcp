---
document_type: domain-spec-section
level: L2
section: reconciliation-notes
version: "1.2"
status: draft
producer: business-analyst
timestamp: 2026-03-29T11:17:00
phase: 1a
inputs: [domain-research.md, L2-domain-spec-v1.1]
traces_to: L2-INDEX.md
---

# Reconciliation Notes — L2 v1.1 → v1.2

> This document summarizes all changes made to the L2 domain spec during
> reconciliation against `.factory/planning/domain-research.md` (738 lines).
> The v1.1 spec had partial annotations suggesting prior reconciliation for
> client capabilities, rmcp confirmation, OWASP AST10, and transport correction.
> This v1.2 pass addresses remaining gaps revealed by the full domain research.

## Methodology

1. Read complete domain research (7 sections: MCP spec, rmcp crate, editor configs, ratatui, JSON-RPC 2.0, security patterns, implications)
2. Read all 9 L2 section files + index
3. Systematic comparison for: missing capabilities, entity refinements, invariant gaps, assumption validation, uncovered edge cases/failure modes, protocol corrections
4. Updated affected files directly; new IDs follow existing numbering sequences

## Summary of Changes

| Category | Items Added | Items Modified | Files Changed |
|----------|------------|----------------|---------------|
| Capabilities | 0 new CAPs | 2 refined (CAP-001, CAP-005) | capabilities.md |
| Entities | 2 new (Resource Subscription, Paginated List Response) | 3 refined (Transport Connection, JSON-RPC Message, Config Source) | entities.md |
| Invariants | 2 new (DI-019, DI-020) | 0 | invariants.md |
| Events | 1 new (SchemaDriftDetected) | 0 | events.md |
| Edge Cases | 5 new (DEC-020 through DEC-024) | 0 | edge-cases.md |
| Assumptions | 2 new (ASM-014, ASM-015) | 0 | assumptions.md |
| Risks | 2 new (R-014, R-015) | 0 | risks.md |
| Failure Modes | 3 new (FM-019, FM-020, FM-021) | 0 | failure-modes.md |
| Differentiators | 0 new | 1 refined (Differentiator 1) | differentiators.md |
| Index | — | Updated counts, doc map, cross-refs, changelog | L2-INDEX.md |

**Total: 17 new IDs, 6 refined items, all 10 section files updated.**

## Detailed Change Log

### 1. VS Code Config Schema Correction (CAP-001, Config Source entity)

**Source:** Domain research §3.3 (VS Code GitHub Copilot MCP)

**Finding:** The product brief says "VS Code (`settings.json` MCP section)" but domain research reveals VS Code uses a separate `mcp.json` file (not `settings.json`) with a different schema:
- Top-level key: `"servers"` (not `"mcpServers"` like Claude/Cursor/Windsurf)
- Explicit `"type"` field: `"stdio"` or `"http"` (others infer transport implicitly)
- Paths: `~/Library/Application Support/Code/User/mcp.json` (user), `.vscode/mcp.json` (workspace)

**Impact:** CAP-001 must handle **two distinct JSON schemas**, not just different file paths. This is a parsing bifurcation, not just a config location difference.

**Changes:**
- `capabilities.md`: CAP-001 now specifies dual schema requirement (`"mcpServers"` vs `"servers"`)
- `entities.md`: Config Source entity now includes schema type attribute and full OS-specific paths for all 4 editors, plus editor-specific extension fields (Cursor `disabled`/`alwaysAllow`, VS Code IntelliSense)

### 2. Complete MCP Method Inventory (CAP-005)

**Source:** Domain research §1.2 (Complete JSON-RPC Method List)

**Finding:** CAP-005 listed an incomplete method set and had a typo (`completions/complete` should be `completion/complete`). The domain research enumerates ~25 distinct method names.

**Missing from v1.1:**
- `resources/subscribe`, `resources/unsubscribe` — resource change notification subscriptions
- `ping` — bidirectional keepalive
- Complete notification inventory (12+ distinct notification methods)
- Progress token support (`_meta.progressToken`)
- Cancellation protocol (`notifications/cancelled`)
- Exact method names for server-initiated methods (`sampling/createMessage`, `elicitation/create`)

**Changes:**
- `capabilities.md`: CAP-005 now lists all client-initiated methods, server-initiated methods, and notifications with exact method names. Includes pagination support.
- `entities.md`: New Resource Subscription entity for `subscribe`/`unsubscribe` lifecycle.

### 3. Cursor-Based Pagination (DI-019, DEC-020, DEC-021, FM-019, ASM-014)

**Source:** Domain research §1.7 (Additional Spec Features)

**Finding:** MCP list methods support cursor-based pagination. This was not addressed anywhere in v1.1 — no invariant for exhausting pages, no edge case for pagination anomalies, no failure mode for infinite loops.

**Changes:**
- `entities.md`: New Paginated List Response entity
- `invariants.md`: DI-019 — list methods must exhaust pagination
- `edge-cases.md`: DEC-020 (inconsistent cursors / infinite loops), DEC-021 (list changes mid-pagination)
- `failure-modes.md`: FM-019 (infinite pagination loop)
- `assumptions.md`: ASM-014 (servers implement pagination correctly)

### 4. Tool Error Semantics (DI-020, DEC-023)

**Source:** Domain research §1.6 (Error Codes) — "Tool execution errors use `result.isError: true` in the response content (not JSON-RPC error codes)"

**Finding:** MCP has two distinct error channels — JSON-RPC protocol errors (error codes) and tool execution errors (`result.isError`). Conflating them would corrupt health metrics, traffic display, and security analysis.

**Changes:**
- `invariants.md`: DI-020 — Forge MCP must distinguish tool errors from protocol errors
- `edge-cases.md`: DEC-023 — tool returns `isError` but response is JSON-RPC success

### 5. Batch JSON-RPC Support (JSON-RPC Message entity, DEC-022)

**Source:** Domain research §5.1 (Batch Request)

**Finding:** JSON-RPC 2.0 supports batch requests (array of request/notification objects). Server processes independently, response order may differ from request order. This was not reflected in the entity model.

**Changes:**
- `entities.md`: JSON-RPC Message entity refined with four message types (Request, Response, Notification, Batch), `is_batch` flag, error code detail
- `edge-cases.md`: DEC-022 (batch response ordering)

### 6. Progress/Cancellation Protocol (DEC-024)

**Source:** Domain research §1.2 (Progress / Cancellation), §1.7

**Finding:** Either side can send `notifications/progress` with a progress token and `notifications/cancelled` to cancel requests. Edge case: server sends progress after client cancellation.

**Changes:**
- `edge-cases.md`: DEC-024 (progress notifications for cancelled requests)
- `capabilities.md`: CAP-005 already mentions notifications; now explicitly includes `notifications/progress` and `notifications/cancelled`

### 7. Rug Pull Attack Pattern (FM-020, R-014, SchemaDriftDetected event)

**Source:** Domain research §6.2 (Rug Pull Attacks)

**Finding:** Domain research documents a specific, known attack vector where MCP servers present benign tool descriptions during approval, then swap to malicious versions post-onboarding. This elevates schema drift detection from a "nice-to-have comparison feature" to a security-critical capability.

**Changes:**
- `failure-modes.md`: FM-020 (rug pull attack — server changes schemas post-approval)
- `risks.md`: R-014 (rug pull / schema drift attacks)
- `events.md`: SchemaDriftDetected event with severity differentiation (announced vs. silent changes)
- `differentiators.md`: Differentiator 1 now explicitly includes schema drift detection via CAP-023

### 8. Streamable HTTP Session Management (FM-021, R-015)

**Source:** Domain research §1.3 (Streamable HTTP Transport), §1.5 (Session Management)

**Finding:** Streamable HTTP involves `Mcp-Session-Id` header management, SSE stream resumption, and multi-client support. Session loss requires re-initialization. This failure surface was not covered.

**Changes:**
- `failure-modes.md`: FM-021 (Streamable HTTP session loss)
- `risks.md`: R-015 (Streamable HTTP session management complexity)
- `entities.md`: Transport Connection entity expanded with session ID, response modes, resumption details

### 9. rmcp Handler Extensibility Assumption (ASM-015)

**Source:** Domain research §2.2 (Key Types and Traits) — `ClientHandler`, `ServerHandler`, `Service<R>` trait

**Finding:** Traffic capture (DI-005) requires intercepting all JSON-RPC messages without modification. Whether rmcp's handler traits support transparent passthrough interception is unvalidated.

**Changes:**
- `assumptions.md`: ASM-015 (rmcp handler extensibility for traffic capture)

### 10. Transport Connection Detail Expansion

**Source:** Domain research §1.3 (Transports), §2.4 (Transport Implementations)

**Finding:** The v1.1 Transport Connection entity was sparse. Domain research provides significant detail: stdio uses UTF-8 newline-delimited messages with stdout-only protocol data, Streamable HTTP uses POST for client→server and SSE for server→client with dual response modes.

**Changes:**
- `entities.md`: Transport Connection entity fully expanded with protocol details for both transport types

## Assumptions Validated or Refined

| ASM | Change | Basis |
|-----|--------|-------|
| ASM-001 | Confidence raised to Medium-High, status partially-validated | rmcp confirmed at warpdotdev/rmcp with `ClientCapabilitiesBuilder` |
| ASM-003 | Confidence lowered to Low | CVE-2025-68145/68143/68144, 36.7% SSRF rate confirm widespread quality issues |
| ASM-007 | Status partially-validated | Spec cadence confirmed ~6 months, faster than initial 12-month estimate |
| ASM-008 | Confidence raised to Medium-High | Substantial corpus of deterministic detection patterns confirmed by BlueRock research |

*These were already reflected in v1.1. No additional validation changes in v1.2.*

## Items NOT Changed (Confirmed as Correct)

- **CAP-002 through CAP-025** (except CAP-001, CAP-005): Already accurate
- **DI-001 through DI-018**: Already accurate
- **DEC-001 through DEC-019**: Already accurate
- **FM-001 through FM-018**: Already accurate
- **Differentiators 2–6**: Already accurate
- **Priority distribution**: Unchanged (P0: 15, P1: 6, P2: 4)
- **Brief→Capability traceability**: All 10 brief capabilities still map correctly

## Open Questions for Architect

1. **rmcp TCP transport**: Domain research mentions rmcp supports TCP in addition to stdio and HTTP. Should Forge MCP expose TCP as a user-configurable transport, or is it implementation-internal to rmcp?
2. **Batch JSON-RPC**: Does rmcp handle batch requests/responses transparently, or does Forge MCP need to assemble/decompose batches?
3. **Resource subscription lifecycle**: How should TUI handle resource subscriptions — auto-subscribe to viewed resources, or explicit user action?
4. **Schema drift storage**: Where should tool/resource metadata hashes be persisted across sessions for rug pull detection?
