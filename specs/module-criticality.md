---
document_type: module-criticality
level: ops
version: "1.0"
status: draft
producer: architect
timestamp: 2026-03-29T14:05:00
phase: 1b
inputs: [specs/architecture/ARCH-INDEX.md, specs/architecture/module-decomposition.md]
input-hash: ""
traces_to: specs/architecture/ARCH-INDEX.md
---

# Module Criticality Classification: Forge MCP

## Tier Definitions

| Tier | Mutation Kill Rate Target | Description | Examples |
|------|--------------------------|-------------|----------|
| **CRITICAL** | >= 95% | Core business logic, security boundaries, data integrity | Authentication, payment processing, state machines |
| **HIGH** | >= 90% | Important functionality with significant user impact | API handlers, validation, data transformation |
| **MEDIUM** | >= 80% | Supporting functionality, utilities | Logging, formatting, configuration parsing |
| **LOW** | >= 70% | Infrastructure, glue code, generated code | Build scripts, boilerplate, wrappers |

## Module Classification

| Module | Path | Tier | Rationale | Kill Rate Target | VP Count |
|--------|------|------|-----------|-----------------|----------|
| forge-core | crates/forge-core/ | CRITICAL | Protocol correctness, all other modules depend on it, handles untrusted network input, state machines for connection and pagination | >= 95% | 4 |
| forge-security | crates/forge-security/ | CRITICAL | Security analysis of untrusted MCP traffic, SSRF/attack detection, confidence scoring directly impacts security decisions, false negatives have severe consequences | >= 95% | 3 |
| forge-traffic | crates/forge-traffic/ | HIGH | Traffic capture integrity (DI-005), bounded memory (NFR-012), ring buffer correctness, feeds data to security and health modules | >= 90% | 3 |
| forge-health | crates/forge-health/ | HIGH | Alert state machine correctness (DI-009), metric calculations used for operational decisions, incorrect alerts erode trust | >= 90% | 2 |
| forge-discovery | crates/forge-discovery/ | HIGH | Config parsing handles untrusted file input, incorrect parsing silently drops servers, foundation for all discovery workflows | >= 90% | 1 |
| forge-conformance | crates/forge-conformance/ | HIGH | Test assertions must be correct — false pass is worse than false fail, CI/CD pipelines gate on results | >= 90% | 0 |
| forge-tui | crates/forge-tui/ | MEDIUM | State machine needs correctness but rendering is display-only, user can visually detect issues, no data corruption risk | >= 80% | 1 |
| forge-config | crates/forge-config/ | MEDIUM | Pure diffing logic, errors are advisory (drift reports), no data mutation risk | >= 80% | 0 |
| forge-daemon | crates/forge-daemon/ | MEDIUM | Session management, socket handling — effectful and hard to unit test, relies on integration tests | >= 80% | 0 |
| forge-mcp (binary) | crates/forge-mcp/ | LOW | Thin CLI dispatch, output formatting, exit codes — glue layer | >= 70% | 0 |

## Classification Summary

| Tier | Module Count | Percentage |
|------|-------------|------------|
| CRITICAL | 2 | 20% |
| HIGH | 4 | 40% |
| MEDIUM | 3 | 30% |
| LOW | 1 | 10% |
| **Total** | **10** | **100%** |

## Security Review Requirements

- **CRITICAL modules:** Full security review on every PR. Two reviewers required.
- **HIGH modules:** Security review on PRs touching public API or input handling.
- **MEDIUM modules:** Standard code review sufficient.
- **LOW modules:** Standard code review sufficient.
