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
subsystem: "Security Auditing"
capability: "CAP-017"
lifecycle_status: active
introduced: v0.1.0
---

# BC-7.17.003 — Authentication Handling Validation

## Summary

The security auditor inspects MCP tool responses and transport configuration for
authentication-related issues: leaked credentials in tool output, missing
authentication on HTTP transports, and auth bypass patterns. This protects
against credential exposure and unauthorized access.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | Tool call responses are available for inspection |
| PRE-002 | Transport configuration is accessible (protocol, auth headers) |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Tool responses are scanned for auth token patterns (API keys, Bearer tokens, JWTs, passwords) |
| POST-002 | Detected credentials produce a high-severity finding with the credential type and redacted evidence |
| POST-003 | Evidence includes the tool name and content type but NEVER the actual credential value (redacted to first/last 4 chars) |
| POST-004 | HTTP/SSE transport connections without authentication headers are flagged as medium findings |
| POST-005 | Self-signed or expired TLS certificates on HTTP transport are flagged as medium findings |
| POST-006 | Stdio transport is exempt from auth checks (local process, no network auth) |

## Credential Detection Patterns

| Pattern | Regex / Heuristic | Confidence | Severity |
|---------|-------------------|------------|----------|
| Bearer token | `Bearer [A-Za-z0-9\-._~+/]+=*` | 0.95 | High |
| JWT | `eyJ[A-Za-z0-9\-_]+\.eyJ[A-Za-z0-9\-_]+\.[A-Za-z0-9\-_]+` | 0.95 | High |
| AWS access key | `AKIA[0-9A-Z]{16}` | 1.0 | Critical |
| AWS secret key | `[A-Za-z0-9/+=]{40}` (near "aws_secret" context) | 0.8 | Critical |
| GitHub token | `gh[ps]_[A-Za-z0-9]{36,}` | 1.0 | Critical |
| Generic API key | `(api[_-]?key\|apikey\|api[_-]?token)\s*[:=]\s*["']?[A-Za-z0-9]{20,}` | 0.7 | High |
| Password in URL | `://[^:]+:[^@]+@` | 0.9 | High |
| Private key header | `-----BEGIN (RSA\|EC\|DSA)? ?PRIVATE KEY-----` | 1.0 | Critical |

## Invariants

| ID | Invariant |
|----|-----------|
| DI-010 | Every security finding must include specific evidence |
| INV-001 | Actual credential values are NEVER included in finding evidence (always redacted) |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Tool response contains a JWT in a "token" field | High finding: JWT detected; evidence shows `eyJh...XYZw` (first 4 + last 4 chars) |
| EC-002 | Tool response contains `AKIA` followed by 16 uppercase alphanumeric | Critical finding: AWS access key pattern |
| EC-003 | Tool response contains "password: ****" (already masked) | No finding: masked values don't match credential patterns |
| EC-004 | Tool response contains the word "Bearer" in documentation text (not a token) | Potential false positive: "Bearer" without token pattern doesn't trigger; full regex required |
| EC-005 | HTTP transport with no Authorization header but using localhost | Medium finding: missing auth (even localhost can be accessed by local processes) |
| EC-006 | Stdio transport with no auth | No finding: stdio is local, auth not applicable |
| EC-007 | Tool response is base64-encoded and contains embedded credentials | Not detected in v1 (base64 decoding is out of scope); documented as known limitation |
| EC-008 | Multiple credentials in single response | One finding per credential detected |
| EC-009 | Credential-like string that is actually a hash (SHA-256 of data) | May trigger false positive; confidence < 1.0 for generic patterns |

## Canonical Test Vectors

### Happy Path — Credential Detection

| Response Content | Findings | Severity | Confidence |
|-----------------|----------|----------|------------|
| `"Authorization: Bearer eyJhbGciOiJI..."` | JWT in response | High | 0.95 |
| `"aws_access_key_id: AKIAIOSFODNN7EXAMPLE"` | AWS access key | Critical | 1.0 |
| `"token: ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"` | GitHub token | Critical | 1.0 |
| `"-----BEGIN RSA PRIVATE KEY-----\nMIIE..."` | Private key | Critical | 1.0 |

### Happy Path — Transport Auth

| Transport | Auth Config | Findings |
|-----------|------------|----------|
| HTTP, `Authorization: Bearer xxx` | Auth present | None |
| HTTP, no auth headers | Auth missing | Medium: missing auth |
| Stdio | N/A | None (exempt) |

### Edge Case

| Response Content | Findings | Notes |
|-----------------|----------|-------|
| `"password: ********"` | None | Masked, doesn't match pattern |
| `"Use Bearer authentication"` | None | "Bearer" without token value |
| `"sha256:abcdef1234567890abcdef1234567890abcdef12"` | Possible false positive | Confidence 0.5 for generic key pattern |

### Error

| Scenario | Expected Behavior |
|----------|-------------------|
| Cannot inspect transport config (e.g., proxy) | Warning: transport auth check skipped; info finding |
| Response too large to scan (> 10MB) | Scan first 10MB; info finding noting truncated scan |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | Known credential patterns (JWT, AWS key, GitHub token, private key) are always detected | Unit test |
| VP-002 | Credential values are never present in finding evidence (redaction check) | Property test |
| VP-003 | Masked credentials (`****`) do not trigger findings | Negative test |
| VP-004 | HTTP transport without auth produces a finding | Unit test |
| VP-005 | Stdio transport without auth does NOT produce a finding | Negative test |
| VP-006 | Multiple credentials in one response produce multiple findings | Unit test |

## Traceability

| Source | Target |
|--------|--------|
| CAP-017 | This contract |
| DI-010 | Invariant (evidence required) |
| AST05 | OWASP AST10 mapping (Inadequate Authentication) |
| BC-7.16.004 | Severity classification (auth findings classified here) |
| BC-7.18.001 | Audit report (auth findings in report) |
| BC-7.18.002 | Suppression rules (users may suppress known-safe patterns) |
