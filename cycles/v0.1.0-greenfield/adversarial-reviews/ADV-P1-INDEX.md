---
document_type: adversarial-review
pass: 1
phase: 1d
---

# ADV-P1 Index

## Finding Summary

| ID | Severity | Category | Title |
|----|----------|----------|-------|
| ADV-P1-001 | HIGH | protocol-error | Streamable HTTP contract uses deprecated SSE endpoint example |
| ADV-P1-002 | HIGH | contradiction | CLI elicitation rejection conflicts with MCP host behavior requirements |
| ADV-P1-003 | HIGH | ux-inconsistency | Security audit suppression UX contradicts suppression contract |
| ADV-P1-004 | MEDIUM | ambiguity | “Proxy layer” is underspecified and architecture never assigns it cleanly |
| ADV-P1-005 | HIGH | architecture-concern | Dependency graph violates its own downward-flow rule |
| ADV-P1-006 | HIGH | verification-gap | No verification property covers suppression overlay integrity |
| ADV-P1-007 | HIGH | missing-edge-case | SSRF detection contract misses DNS rebinding and hostname resolution cases |
| ADV-P1-008 | MEDIUM | contradiction | Security audit input/response scope is inconsistent across contracts |
| ADV-P1-009 | MEDIUM | ux-inconsistency | Traffic replay UX omits explicit target designation required by contract |
| ADV-P1-010 | HIGH | spec-gap | Session pooling spec lacks isolation rules for server-initiated requests across shared clients |
| ADV-P1-011 | MEDIUM | verification-gap | No formal/property verification for replay safety invariant |
| ADV-P1-012 | MEDIUM | ambiguity | TUI pane model conflicts with full-tab screens added later in UX spec |
| ADV-P1-013 | HIGH | security-blind-spot | Sampling proxy logs can capture prompt and tool-call secrets with no redaction policy |
| ADV-P1-014 | MEDIUM | contradiction | Behavioral contracts invent error codes not present in taxonomy |
| ADV-P1-015 | HIGH | protocol-error | Cancellation message shape is underspecified against MCP progress/cancel model |
| ADV-P1-016 | HIGH | verification-gap | No VP covers daemon multiplexing correctness despite pooled-session risk |
| ADV-P1-017 | MEDIUM | implicit-assumption | Root enforcement assumes path-like schema fields can be identified reliably |
| ADV-P1-018 | MEDIUM | missing-edge-case | Holdout set misses malformed/incomplete transport-frame capture scenarios |

## Category Groups

### Protocol Errors
- ADV-P1-001
- ADV-P1-015

### Contradictions
- ADV-P1-002
- ADV-P1-008
- ADV-P1-014

### UX Inconsistencies
- ADV-P1-003
- ADV-P1-009

### Ambiguities
- ADV-P1-004
- ADV-P1-012

### Architecture Concerns
- ADV-P1-005

### Verification Gaps
- ADV-P1-006
- ADV-P1-011
- ADV-P1-016

### Missing Edge Cases
- ADV-P1-007
- ADV-P1-018

### Spec Gaps
- ADV-P1-010

### Security Blind Spots
- ADV-P1-013

### Implicit Assumptions
- ADV-P1-017

## Severity Totals
- HIGH: 10
- MEDIUM: 8
- LOW: 0
- CRITICAL: 0

## Priority Triage Order
1. ADV-P1-010 — pooled session routing gap for server-initiated requests
2. ADV-P1-013 — sensitive sampling traffic logging without redaction policy
3. ADV-P1-001 / ADV-P1-015 — MCP transport/cancellation protocol drift risks
4. ADV-P1-005 / ADV-P1-016 — architecture and verification mismatch around shared routing
5. ADV-P1-003 / ADV-P1-006 — suppression semantics inconsistent and under-verified
6. ADV-P1-007 / ADV-P1-008 / ADV-P1-017 — security analysis scope/model gaps
