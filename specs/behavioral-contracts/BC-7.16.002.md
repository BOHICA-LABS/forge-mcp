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
capability: "CAP-016"
lifecycle_status: active
introduced: v0.1.0
---

# BC-7.16.002 — SSRF Attempt Detection (Private IPs, Metadata Endpoints)

## Summary

The security auditor scans tool responses for private IP addresses and cloud
metadata endpoints that indicate potential SSRF (Server-Side Request Forgery)
attacks. Detection is deterministic (regex/CIDR matching), not heuristic.
Each finding includes the specific URI and message context as evidence.

## Preconditions

| ID | Condition |
|----|-----------|
| PRE-001 | Tool call responses are available for inspection |
| PRE-002 | The auditor has access to response content (text content items) |

## Postconditions

| ID | Condition |
|----|-----------|
| POST-001 | Cloud metadata endpoint `169.254.169.254` is detected with confidence ≥ 0.9 |
| POST-002 | RFC 1918 private IPs are detected: `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16` with confidence ≥ 0.7 |
| POST-003 | Loopback addresses `127.0.0.0/8` are detected with confidence ≥ 0.7 |
| POST-004 | Link-local addresses `169.254.0.0/16` (beyond .169.254) are detected with confidence ≥ 0.7 |
| POST-005 | IPv6 equivalents are detected: `::1` (loopback), `fe80::/10` (link-local), `fc00::/7` (ULA) |
| POST-006 | Each finding includes evidence: the specific URI or IP found, surrounding message context (±100 chars) |
| POST-007 | Detection operates on response text content; binary content is skipped |
| POST-008 | Known cloud metadata paths are detected: `/latest/meta-data/`, `/metadata/v1/`, `/computeMetadata/v1/` |

## Detection Rules

| Target | CIDR / Pattern | Confidence | Severity |
|--------|---------------|------------|----------|
| AWS/Azure/GCP metadata | `169.254.169.254` | 0.95 | Critical |
| AWS metadata path | `/latest/meta-data/` | 0.95 | Critical |
| GCP metadata path | `/computeMetadata/v1/` | 0.95 | Critical |
| Loopback | `127.0.0.0/8`, `::1` | 0.80 | High |
| Class A private | `10.0.0.0/8` | 0.70 | Medium |
| Class B private | `172.16.0.0/12` | 0.70 | Medium |
| Class C private | `192.168.0.0/16` | 0.70 | Medium |
| Link-local | `169.254.0.0/16` (excl. .169.254) | 0.70 | Medium |
| IPv6 ULA | `fc00::/7` | 0.70 | Medium |
| IPv6 link-local | `fe80::/10` | 0.70 | Medium |

## Invariants

| ID | Invariant |
|----|-----------|
| DI-010 | Every security finding must include specific evidence |

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | Tool response contains `http://169.254.169.254/latest/meta-data/iam/` | Critical finding, confidence 0.95, evidence includes full URL |
| EC-002 | Tool response contains `192.168.1.1` in a config explanation | Medium finding, confidence 0.70; may be false positive for internal tools (DEC-018) |
| EC-003 | Tool response mentions IP `10.0.0.1` as example in documentation text | Medium finding, confidence 0.70; user may suppress (BC-7.18.002) |
| EC-004 | False positive storm for internal service tool (FM-018) | Many findings generated; all have evidence. User should configure suppression rules for known-internal servers |
| EC-005 | IP address encoded as decimal integer (`2130706433` = 127.0.0.1) | Not detected (out of scope for v1; document as known limitation) |
| EC-006 | IP address in URL with non-standard port (`http://10.0.0.1:8080/api`) | Detected: `10.0.0.1` matches Class A private |
| EC-007 | IPv6 address `::ffff:192.168.1.1` (IPv4-mapped) | Detected: maps to 192.168.0.0/16 |
| EC-008 | Tool response is empty | No findings (nothing to scan) |
| EC-009 | URL in tool input (not response) | Not scanned by this contract (input scanning is separate) |

## Canonical Test Vectors

### Happy Path

| Response Content | Findings | Confidence | Severity |
|-----------------|----------|------------|----------|
| `"Fetched http://169.254.169.254/latest/meta-data/role"` | SSRF: cloud metadata | 0.95 | Critical |
| `"Connected to http://127.0.0.1:3000/health"` | SSRF: loopback | 0.80 | High |
| `"Response from http://10.0.5.20/api/v1/data"` | SSRF: private IP (10/8) | 0.70 | Medium |
| `"API at http://192.168.1.100:8080/"` | SSRF: private IP (192.168/16) | 0.70 | Medium |

### Edge Case

| Response Content | Findings | Notes |
|-----------------|----------|-------|
| `"The server IP is 8.8.8.8"` | No findings | Public IP, not flagged |
| `"Example: curl http://10.0.0.1 for local access"` | Medium: private IP | Detected even in example text |
| `"IPv6 address ::1 used for testing"` | High: loopback | IPv6 loopback detected |

### Error

| Scenario | Expected Behavior |
|----------|-------------------|
| Response contains binary data (non-UTF-8) | Skip binary content; no findings for that content item |
| Response has 10,000 IP addresses | All scanned; findings generated for each match (user should use suppression) |

## Verification Properties

| ID | Property | Type |
|----|----------|------|
| VP-001 | `169.254.169.254` in any response content always produces a critical finding | Unit test |
| VP-002 | Public IPs (e.g., 8.8.8.8, 1.1.1.1) never produce findings | Negative test |
| VP-003 | All RFC 1918 ranges are covered (10/8, 172.16/12, 192.168/16) | CIDR exhaustive test |
| VP-004 | IPv6 equivalents are detected (::1, fe80::, fc00::) | Unit test |
| VP-005 | Every finding includes the matched IP/URL as evidence | Property test |
| VP-006 | Confidence scores match the detection rules table | Unit test |

## Traceability

| Source | Target |
|--------|--------|
| CAP-016 | This contract |
| DI-010 | Invariant (evidence required) |
| DEC-018 | Edge case (false positives for internal services) |
| FM-018 | Failure mode (false positive storm) |
| BC-7.16.004 | Severity classification (SSRF findings use confidence scores) |
| BC-7.18.002 | Suppression rules (users suppress false positives) |
| BC-7.18.003 | AST10 coverage (SSRF maps to AST02 — Excessive Data Exposure) |
