---
document_type: architecture-section
level: L3
section: purity-boundary-map
version: "1.0"
status: draft
producer: architect
timestamp: 2026-03-29T14:05:00
phase: 1b
inputs: [domain-spec/L2-INDEX.md, specs/prd.md]
traces_to: ARCH-INDEX.md
---

# Purity Boundary Map

This is the VSDD-critical artifact that determines what can be formally verified.

**Classification principle:** A function is PURE if it takes data in and returns a result with no I/O, no network, no filesystem, no global mutable state. Everything else is EFFECTFUL.

## Module Boundary Classifications

| Module | Pure Core | Effectful Shell | Verification Impact |
|--------|-----------|-----------------|-------------------|
| **forge-core** | Message type definitions, error classification (DI-020), pagination state machine (DI-019), capability set operations | rmcp Peer calls, transport I/O, ClientHandler callbacks | Pure core: Kani/proptest. Shell: integration tests |
| **forge-discovery** | JSON schema parsing (mcpServers vs servers), path resolution logic, config normalization, conflict detection (BC-1.01.003) | File system reads, env var expansion | Pure core: Kani for parser correctness, proptest for schema fuzzing. Shell: integration tests with fixture files |
| **forge-daemon** | Session ID generation/validation (DI-003) | Socket management, process lifecycle, signal handling, connection pool | Shell-heavy: integration tests only |
| **forge-tui** | State machine (panel focus, navigation mode, input mode), layout calculation, color system selection (BC-3.06.002), widget data preparation | Terminal I/O (crossterm), event polling, frame rendering | Pure core: proptest for state transitions. Shell: manual + screenshot tests |
| **forge-traffic** | Ring buffer logic, message filtering, full-text search, timing calculations, ordering validation (DI-005, DI-006) | Disk spill on overflow, replay dispatch | Pure core: Kani for ring buffer bounds, proptest for filter correctness. Shell: integration tests |
| **forge-health** | Histogram calculation, error rate computation, throughput windowed counter, alert state machine (DI-009), metric snapshot construction | Timer ticks (minimal effectful surface) | Almost entirely pure: Kani for arithmetic, proptest for alert state machine |
| **forge-security** | Rule engine pattern matching, IP classification (RFC1918, link-local), schema hash comparison, finding severity classification (DI-010, DI-011), confidence scoring, suppression overlay (DI-012), AST10 category mapping | Rule file loading from disk | Almost entirely pure: Kani for IP classification, proptest for rule engine, fuzz for pattern matching |
| **forge-conformance** | Test case definitions, spec assertions, result aggregation, JUnit XML generation, JSON report generation | Test execution (sends real protocol messages) | Pure core: unit tests for assertion logic. Shell: integration tests against mock server |
| **forge-config** | Config comparison, drift report generation, tool schema diffing, capability delta computation | None (operates on in-memory data) | Fully pure: Kani/proptest for diff correctness |
| **forge-mcp (binary)** | Output formatting (JSON/text/table), token counting, exit code mapping (DI-014) | stdout/stderr I/O, clap parsing, signal handling, panic hook | Pure core: unit tests for formatting. Shell: integration tests |

## Purity Summary

- **Fully pure modules:** forge-config
- **Pure-heavy modules (>70% pure):** forge-health, forge-security, forge-traffic, forge-discovery
- **Mixed modules:** forge-core, forge-tui, forge-conformance, forge-mcp
- **Effectful-heavy modules:** forge-daemon

## Design Implications

1. **Dependency direction flows inward:** effectful shells depend on pure cores, never the reverse.
2. **Module boundaries enforce purity:** no I/O types appear in pure core function signatures.
3. **Formal verification targets pure cores exclusively:** Kani and proptest operate only on pure functions.
4. **Integration tests cover effectful shells:** real I/O, process lifecycle, and transport behavior validated through integration tests with mock servers and fixture files.
