---
document_type: story
story_id: STORY-048
epic_id: EPIC-07
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: prd.md
points: 5
depends_on: [STORY-027]
blocks: [STORY-051, STORY-052]
behavioral_contracts: [BC-7.16.002]
verification_properties: [VP-009]
priority: P1
assumption_validations: []
risk_mitigations: [R-004, R-011]
---

# STORY-048: SSRF Attempt Detection

## Narrative
- **As a** Security Team member
- **I want to** have Forge MCP detect SSRF-style requests in tool calls
- **So that** I can identify servers attempting to access internal network resources (36.7% SSRF rate per BlueRock research)

## Acceptance Criteria

### AC-001 (traces to BC-7.16.002 postcondition — RFC1918 private IP detection)
When a tool call argument contains an IP in RFC 1918 ranges (10.0.0.0/8, 172.16.0.0/12, 192.168.0.0/16), emits Finding with `category: SSRF`, `severity: Critical`. (Kani proof VP-009 for IP classification correctness.)
- **Test:** `test_BC_7_16_002_rfc1918_detection()` (VP-009 Kani proof)

### AC-002 (traces to BC-7.16.002 postcondition — link-local detection)
Detects link-local range 169.254.0.0/16 (AWS metadata endpoint `169.254.169.254`). Emits Finding with `category: SSRF`, `severity: Critical`.
- **Test:** `test_BC_7_16_002_link_local_169_detection()`

### AC-003 (traces to BC-7.16.002 postcondition — metadata endpoint detection)
Detects well-known cloud metadata hostnames: `metadata.google.internal`, `169.254.169.254`, `fd00:ec2::254`, `metadata.azure.com`. Critical severity.
- **Test:** `test_BC_7_16_002_cloud_metadata_endpoints()`

### AC-004 (traces to BC-7.16.002 postcondition — localhost detection)
Detects localhost references: `127.0.0.1`, `::1`, `localhost`, `0.0.0.0` in tool call URLs.
- **Test:** `test_BC_7_16_002_localhost_detection()`

### AC-005 (traces to BC-7.16.002 — false positive rate < 20%)
SSRF detection has < 20% false positive rate on known-good corpus of 50 tools making legitimate HTTP calls. (NFR-005 reverse side.)
- **Test:** Corpus test for false positive rate

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| `IpClassifier` | `forge-security/src/ip_classifier.rs` | Pure |
| SSRF rule in RuleEngine | `forge-security/src/engine.rs` | Pure |

## UX Screens
- SCR-007 (Security Audit View)

## Edge Cases

| ID | Scenario | Expected Behavior |
|----|----------|-------------------|
| EC-001 | IPv6 mapped addresses (::ffff:10.0.0.1) | Detected as RFC1918 |
| EC-002 | URL encoding of IP (%31%36%39...) | Not detected in v0.1.0 (documented gap) |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| IpClassifier | Pure | IP → classification, pure math |

## Token Budget Estimate

| Context Source | Estimated Tokens |
|----------------|-----------------|
| This story spec | ~900 |
| BC-7.16.002 | ~600 |
| VP-009 Kani proof | ~400 |
| **Total** | **~1,900** |
| Agent context window | 200K |
| **Budget usage** | **~1%** |

## Tasks

1. [ ] Write failing tests for all 5 ACs
2. [ ] Implement `IpClassifier` (pure, RFC1918 + link-local + localhost)
3. [ ] Write Kani proof for VP-009 IP classification correctness
4. [ ] Implement cloud metadata endpoint detection
5. [ ] Add SSRF rule to RuleEngine
6. [ ] Run false-positive corpus test
7. [ ] Verify Red Gate
8. [ ] Implement to pass tests

## Previous Story Intelligence

| Story | Key Decisions | Patterns Established | Gotchas Discovered |
|-------|--------------|---------------------|-------------------|
| STORY-047 | RuleEngine + finding types established | Add SSRF rule to engine | IP parsing needs IPv4 + IPv6 both |

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| IpClassifier is pure | purity-boundary-map.md | No network lookups in classifier |
| OWASP AST10 mapping required | BC-7.18.003 | SSRF maps to AST07 |

## Library & Framework Requirements

| Dependency | Version Constraint | Why This Version | Import Pattern |
|-----------|-------------------|-----------------|----------------|
| std::net | stdlib | IP parsing | `std::net::IpAddr` |
| kani | dev | VP-009 proof | `#[kani::proof]` |

## File Structure Requirements

| File | Purpose | Pre-exists? |
|------|---------|------------|
| `crates/forge-security/src/ip_classifier.rs` | IP classification | NO — this story creates it |
| `crates/forge-security/proofs/ip_classifier.rs` | VP-009 Kani proof | NO |
