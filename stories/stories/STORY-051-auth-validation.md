---
document_type: story
story_id: STORY-051
epic_id: EPIC-07
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 3
depends_on: [STORY-048]
blocks: [STORY-052]
behavioral_contracts: [BC-7.17.003]
verification_properties: []
priority: P1
assumption_validations: []
risk_mitigations: []
---

# STORY-051: Authentication Handling Validation

## Narrative
- **As a** Security Team member
- **I want to** detect authentication-related security issues in MCP traffic
- **So that** I can identify servers that bypass or mishandle auth

## Acceptance Criteria

### AC-001 (traces to BC-7.17.003 postcondition — auth bypass detection)
When a tool call succeeds after connection was established without valid auth (e.g., using expired/malformed headers), emits Finding `category: AuthBypass`, `severity: High`.
- **Test:** `test_BC_7_17_003_auth_bypass_detected()`

### AC-002 (traces to BC-7.17.003 postcondition — plaintext credentials)
When tool call arguments or results contain patterns matching credentials (Bearer tokens, API keys, passwords, `Authorization: Basic`), emits Finding `category: CredentialLeak`, `severity: High`.
- **Test:** `test_BC_7_17_003_plaintext_credentials()`

### AC-003 (traces to BC-7.17.003 — OWASP AST08 mapping)
Auth findings tagged `owasp: AST08` (Weak Authentication). Contributes to NFR-006.
- **Test:** `test_BC_7_17_003_ast08_tag()`

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `AuthValidator` | `forge-security/src/auth.rs` | Pure |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | "Authorization" appearing in documentation | Low confidence finding |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| AuthValidator | Pure | Pattern matching on messages |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~600 |
| BC-7.17.003 | ~400 |
| **Total** | **~1,000** |
| Agent context window | 200K |
| **Budget usage** | **~0.5%** |

## Tasks

1. [ ] Write failing tests for all 3 ACs
2. [ ] Implement AuthValidator with auth bypass and credential leak patterns
3. [ ] Add OWASP AST08 tag
4. [ ] Verify Red Gate
5. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-048 | SSRF pattern matching established | Same pattern for auth detection | Credential patterns should have high precision threshold |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Pure detection | purity-boundary-map.md | |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| regex | >= 1.10 | Credential pattern matching | |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-security/src/auth.rs` | AuthValidator | NO — this story creates it |
