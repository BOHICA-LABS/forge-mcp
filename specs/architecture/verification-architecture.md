---
document_type: architecture-section
level: L3
section: verification-architecture
version: "1.0"
status: draft
producer: architect
timestamp: 2026-03-29T14:05:00
phase: 1b
inputs: [domain-spec/L2-INDEX.md, specs/prd.md]
traces_to: ARCH-INDEX.md
---

# Verification Architecture

## Provable Properties Catalog

### Must Prove (Kani model checking)

- **Ring buffer bounds:** append never overflows, FIFO eviction preserves ordering (forge-traffic)
- **IP classification correctness:** RFC1918 / link-local / metadata IP detection has zero false negatives (forge-security)
- **Alert state machine:** transitions follow exactly normal→breached→recovered, no invalid states (forge-health)
- **Config parser:** never panics on arbitrary JSON input (forge-discovery)
- **Error classification:** tool errors (isError) vs protocol errors are never conflated (forge-core)
- **Pagination state machine:** always terminates, cursor dedup detects cycles (forge-core)
- **Confidence score bounds:** always in [0.0, 1.0] (forge-security)

### Should Prove (proptest property-based testing)

- **Traffic filter correctness:** filter(capture) ⊆ capture, and filter preserves ordering (forge-traffic)
- **Health metric calculations:** latency histogram is mathematically correct for all input distributions (forge-health)
- **Config normalization:** parse(serialize(entry)) == entry for all valid entries (forge-discovery)
- **Schema drift detection:** detects any single-field change in tool metadata (forge-security)
- **TUI state machine:** all key bindings reach expected states, no unreachable states (forge-tui)
- **Output formatting:** JSON output is always valid JSON (forge-mcp)
- **JUnit XML generation:** output is always well-formed XML (forge-conformance)

### Fuzz (cargo-fuzz)

- **JSON-RPC message parsing:** arbitrary bytes → no panic (forge-core)
- **Config file parsing:** arbitrary JSON → no panic, returns Ok or Err (forge-discovery)
- **Traffic filter regex:** arbitrary regex patterns → no panic or hang (forge-traffic)
- **Security rule matching:** arbitrary message payloads → no panic (forge-security)

### Test Sufficient (integration/manual)

- TUI rendering (manual visual inspection + screenshot comparison)
- Daemon lifecycle (integration tests with real processes)
- rmcp transport behavior (integration tests against mock servers)
- Crossterm terminal I/O
- File system operations in discovery

## Proof Strategy by Module

| Module | Kani | proptest | cargo-fuzz | Integration | Manual |
|--------|------|---------|------------|-------------|--------|
| forge-core | error classification, pagination FSM | message type roundtrip | JSON-RPC parsing | rmcp integration | — |
| forge-discovery | config parser no-panic | normalization roundtrip | arbitrary JSON input | file fixture tests | — |
| forge-traffic | ring buffer bounds | filter correctness | regex patterns | replay against mock | — |
| forge-health | alert state machine, arithmetic | histogram accuracy | — | metric collection | — |
| forge-security | IP classification, confidence bounds | rule engine coverage, drift detection | arbitrary payloads | end-to-end audit | — |
| forge-tui | — | state machine transitions | — | — | visual inspection |
| forge-conformance | — | assertion logic | — | mock server tests | — |
| forge-config | — | diff correctness | — | fixture-based | — |
| forge-daemon | — | — | — | process lifecycle | — |
| forge-mcp | — | output formatting | — | CLI integration | — |
