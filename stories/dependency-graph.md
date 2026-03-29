---
document_type: story-dependency-graph
level: L3
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T15:30:00
phase: 2
cycle: v0.1.0-greenfield
traces_to: STORY-INDEX.md
---

# Story Dependency Graph — Forge MCP

> Dependencies are ACYCLIC. Validated by topological sort below.
> Arrow direction: A → B means "A must complete before B can start."

---

## Dependency Table

| Story | Depends On | Notes |
|-------|-----------|-------|
| STORY-001 | (none) | Foundation |
| STORY-002 | STORY-001 | Needs workspace structure |
| STORY-003 | STORY-001 | Needs workspace structure |
| STORY-004 | STORY-001 | Needs crate stubs |
| STORY-005 | STORY-004 | parse_config needs DiscoveredConfig |
| STORY-006 | STORY-005 | aggregate needs ServerEntry |
| STORY-007 | STORY-002, STORY-004 | Needs mock server + StdioConfig |
| STORY-008 | STORY-003, STORY-004 | Needs HTTP mock + HttpConfig |
| STORY-009 | STORY-007, STORY-008 | Lifecycle on top of both transports |
| STORY-010 | STORY-007, STORY-008, STORY-009 | Daemon needs connections |
| STORY-011 | STORY-010 | Named sessions extend pool |
| STORY-012 | STORY-010 | Socket conflict on top of daemon |
| STORY-013 | STORY-007, STORY-008 | Capability negotiation needs transport |
| STORY-014 | STORY-013 | Client caps after negotiation |
| STORY-015 | STORY-013 | Spec version after negotiation |
| STORY-016 | STORY-013 | Tool ops need negotiated connection |
| STORY-017 | STORY-013 | Resource ops |
| STORY-018 | STORY-013 | Prompt ops |
| STORY-019 | STORY-014 | Sampling needs client capability |
| STORY-020 | STORY-014 | Elicitation needs client capability |
| STORY-021 | STORY-013 | Utility protocol ops |
| STORY-022 | STORY-016 | Progress/cancel needs tool invocation |
| STORY-023 | STORY-013 | CLI needs a protocol connection |
| STORY-024 | STORY-023 | Output on top of subcommands |
| STORY-025 | STORY-024 | Pipe on top of JSON output |
| STORY-026 | STORY-023, STORY-033 | Metric export needs CLI + metrics |
| STORY-027 | STORY-013 | Capture needs connection events |
| STORY-028 | STORY-027 | Timing on captured messages |
| STORY-029 | STORY-027 | Buffer on captured messages |
| STORY-030 | STORY-029 | Filter needs buffer |
| STORY-031 | STORY-029 | Search needs buffer |
| STORY-032 | STORY-029 | Replay needs buffer |
| STORY-033 | STORY-027 | Metrics subscribe to captured events |
| STORY-034 | STORY-033 | Error rate on top of metric collector |
| STORY-035 | STORY-033 | Thresholds on metric snapshots |
| STORY-036 | STORY-035 | State machine on threshold events |
| STORY-037 | STORY-006, STORY-013 | TUI needs server registry + protocol |
| STORY-038 | STORY-037 | Color on top of TUI layout |
| STORY-039 | STORY-037 | Navigation on TUI state machine |
| STORY-040 | STORY-039 | Search/command extends navigation |
| STORY-041 | STORY-039 | Mouse extends navigation |
| STORY-042 | STORY-037, STORY-027 | JSON viewer needs TUI + captured msgs |
| STORY-043 | STORY-037, STORY-036 | Health viz needs TUI + alert state |
| STORY-044 | STORY-037, STORY-006 | Server browser needs TUI + registry |
| STORY-045 | STORY-037, STORY-016, STORY-017, STORY-018 | Cap explorer needs TUI + protocol |
| STORY-046 | STORY-043, STORY-033 | Time-series viz needs sparklines + metrics |
| STORY-047 | STORY-027 | Dangerous pattern detection on captured messages |
| STORY-048 | STORY-027 | SSRF detection on captured messages |
| STORY-049 | STORY-027 | Schema drift on captured messages |
| STORY-050 | STORY-047 | Escalation builds on pattern detection |
| STORY-051 | STORY-048 | Auth builds on SSRF detection |
| STORY-052 | STORY-047, STORY-048, STORY-049, STORY-050, STORY-051 | Report aggregates all detectors |
| STORY-053 | STORY-052 | Suppression runs on generated report |
| STORY-054 | STORY-052, STORY-053, STORY-037 | TUI view needs report + suppression + layout |
| STORY-055 | STORY-013, STORY-002 | Conformance needs connection + mock |
| STORY-056 | STORY-055 | Method coverage extends cap conformance |
| STORY-057 | STORY-055, STORY-003 | Transport compliance needs HTTP mock |
| STORY-058 | STORY-055, STORY-056, STORY-057 | Reports need all conformance suites |
| STORY-059 | STORY-006 | Config drift needs registry/conflicts |
| STORY-060 | STORY-059 | Reconciliation extends drift report |
| STORY-061 | STORY-016, STORY-006 | Schema diff needs tool list + configs |
| STORY-062 | STORY-013 | Capability delta needs negotiation |
| STORY-063 | STORY-016 | Response delta needs tool invocation |
| STORY-064 | STORY-024, STORY-033, STORY-029 | NFR tests need CLI + metrics + buffer |
| STORY-065 | STORY-005, STORY-047 | SR refinements need both stories |

---

## ASCII DAG (Critical Path Highlighted)

```
Wave 0: STORY-001
         ├── STORY-002 (mock stdio)
         └── STORY-003 (mock HTTP)

Wave 1: STORY-004 (config discovery)
         └── STORY-005 (config parsing)
              └── STORY-006 (aggregation)
         
         STORY-007 (stdio transport) [deps: 001,002,004]
         STORY-008 (HTTP transport)  [deps: 001,003,004]
              └── STORY-009 (lifecycle)
                   └── STORY-010 (daemon)
                        ├── STORY-011 (session persistence)
                        └── STORY-012 (socket conflict)
         
         STORY-013 (capability negotiation) [deps: 007,008]  ← CRITICAL PATH START
              ├── STORY-014 (client caps)
              │    ├── STORY-019 (sampling)
              │    └── STORY-020 (elicitation)
              └── STORY-015 (spec degradation)

Wave 2: STORY-016 (tool list/invoke) [dep: 013]  ← CRITICAL PATH
         ├── STORY-022 (progress/cancel)
         ├── STORY-045 (cap explorer) [also: 017,018]
         ├── STORY-061 (schema diff) [also: 006]
         └── STORY-063 (response delta)
         
         STORY-017 (resources) [dep: 013]
         STORY-018 (prompts)   [dep: 013]
         STORY-021 (utilities) [dep: 013]
         
         STORY-023 (CLI subcommands) [dep: 013]
              └── STORY-024 (JSON output)
                   └── STORY-025 (pipe output)

Wave 3: STORY-027 (message capture) [dep: 013]  ← CRITICAL PATH (feeds security)
         ├── STORY-028 (timing)
         ├── STORY-029 (buffer)  ← CRITICAL PATH
         │    ├── STORY-030 (filtering)
         │    ├── STORY-031 (search)
         │    └── STORY-032 (replay)
         ├── STORY-033 (metrics)  ← CRITICAL PATH
         │    ├── STORY-034 (error rate)
         │    ├── STORY-035 (thresholds)
         │    │    └── STORY-036 (alert SM)  ← CRITICAL PATH (blocks TUI)
         │    └── STORY-026 (metric export) [also: 023]
         ├── STORY-047 (dangerous tools)  [early security]
         ├── STORY-048 (SSRF)
         └── STORY-049 (schema drift)

Wave 4: STORY-037 (TUI layout) [deps: 006,013]  ← CRITICAL PATH
         ├── STORY-038 (color)
         ├── STORY-039 (navigation)  ← CRITICAL PATH
         │    ├── STORY-040 (search/command mode)
         │    └── STORY-041 (mouse)
         ├── STORY-042 (JSON rendering) [also: 027]
         ├── STORY-043 (sparklines) [also: 036]  ← CRITICAL PATH
         │    └── STORY-046 (time-series) [also: 033]
         ├── STORY-044 (server browser) [also: 006]
         └── STORY-045 (cap explorer) [also: 016,017,018]

Wave 5: STORY-050 (escalation) [dep: 047]
         STORY-051 (auth) [dep: 048]
         STORY-052 (report) [deps: 047-051]
              ├── STORY-053 (suppression)
              └── STORY-054 (TUI security view) [also: 037,053]

Wave 6: STORY-055 (conformance cap) [deps: 013,002]
         ├── STORY-056 (method coverage)
         ├── STORY-057 (transport) [also: 003]
         └── STORY-058 (reports) [deps: 055-057]
         
         STORY-059 (config drift) [dep: 006]
              └── STORY-060 (reconciliation)
         
         STORY-061 (schema diff) [deps: 016,006]
         STORY-062 (cap delta) [dep: 013]
         STORY-063 (response delta) [dep: 016]
         
         STORY-064 (NFR benchmarks) [deps: 024,033,029]
         STORY-065 (SR refinements) [deps: 005,047]
```

---

## Topological Sort (Validates Acyclicity)

Layer 0: STORY-001
Layer 1: STORY-002, STORY-003, STORY-004
Layer 2: STORY-005, STORY-007, STORY-008
Layer 3: STORY-006, STORY-009
Layer 4: STORY-010, STORY-013
Layer 5: STORY-011, STORY-012, STORY-014, STORY-015, STORY-016, STORY-017, STORY-018, STORY-021, STORY-023, STORY-027
Layer 6: STORY-019, STORY-020, STORY-022, STORY-024, STORY-028, STORY-029, STORY-033, STORY-055, STORY-062
Layer 7: STORY-025, STORY-030, STORY-031, STORY-032, STORY-034, STORY-035, STORY-047, STORY-048, STORY-049, STORY-056, STORY-057, STORY-059, STORY-061, STORY-063
Layer 8: STORY-026, STORY-036, STORY-037, STORY-050, STORY-051, STORY-058, STORY-060, STORY-065
Layer 9: STORY-038, STORY-039, STORY-043, STORY-044, STORY-045, STORY-052, STORY-064
Layer 10: STORY-040, STORY-041, STORY-042, STORY-046, STORY-053
Layer 11: STORY-054

✅ **No cycles detected.** All 65 stories placed in exactly one topological layer.

---

## Critical Path

The longest dependency chain determines minimum delivery time:

```
STORY-001 → STORY-002 → STORY-007 → STORY-013 → STORY-027 → STORY-033
         → STORY-035 → STORY-036 → STORY-037 → STORY-043 → STORY-046
```

**Critical path length:** 12 stories
**Critical path wave span:** Wave 0 → Wave 4

---

## BC Clause Coverage Matrix

> All 65 BCs mapped to covering stories. ✅ = covered, GAP = documented gap.

| BC ID | Covering Story | AC | Status |
|-------|---------------|-----|--------|
| BC-1.01.001 | STORY-004 | AC-001–009 | ✅ |
| BC-1.01.002 | STORY-005 | AC-001–010 | ✅ |
| BC-1.01.003 | STORY-006 | AC-001–005 | ✅ |
| BC-1.02.001 | STORY-007 | AC-001–005 | ✅ |
| BC-1.02.002 | STORY-008 | AC-001–006 | ✅ |
| BC-1.02.003 | STORY-009 | AC-001–004 | ✅ |
| BC-1.03.001 | STORY-010 | AC-001–005 | ✅ |
| BC-1.03.002 | STORY-011 | AC-001–004 | ✅ |
| BC-1.03.003 | STORY-012 | AC-001–004 | ✅ |
| BC-2.04.001 | STORY-013 | AC-001–005 | ✅ |
| BC-2.04.002 | STORY-014 | AC-001–004 | ✅ |
| BC-2.04.003 | STORY-015 | AC-001–004 | ✅ |
| BC-2.05.001 | STORY-016 | AC-001–006 | ✅ |
| BC-2.05.002 | STORY-017 | AC-001–005 | ✅ |
| BC-2.05.003 | STORY-018 | AC-001–004 | ✅ |
| BC-2.05.004 | STORY-019 | AC-001–004 | ✅ |
| BC-2.05.005 | STORY-020 | AC-001–004 | ✅ |
| BC-2.05.006 | STORY-021 | AC-001 | ✅ |
| BC-2.05.007 | STORY-021 | AC-002,004 | ✅ |
| BC-2.05.008 | STORY-021 | AC-003 | ✅ |
| BC-2.05.009 | STORY-022 | AC-001–003 | ✅ |
| BC-2.05.010 | STORY-022 | AC-004–005 | ✅ |
| BC-3.06.001 | STORY-037 | AC-001–007 | ✅ |
| BC-3.06.002 | STORY-038 | AC-001–005 | ✅ |
| BC-3.07.001 | STORY-039 | AC-001–007 | ✅ |
| BC-3.07.002 | STORY-040 | AC-001–004 | ✅ |
| BC-3.07.003 | STORY-041 | AC-001–004 | ✅ |
| BC-3.08.001 | STORY-042 | AC-001–005 | ✅ |
| BC-3.08.002 | STORY-043 | AC-001–005 | ✅ |
| BC-3.08.003 | STORY-044 | AC-001–005 | ✅ |
| BC-3.08.004 | STORY-045 | AC-001–005 | ✅ |
| BC-3.08.005 | STORY-044, STORY-045 | AC-002, AC-004 | ✅ |
| BC-4.09.001 | STORY-027 | AC-001–004 | ✅ |
| BC-4.09.002 | STORY-028 | AC-001–004 | ✅ |
| BC-4.09.003 | STORY-029 | AC-001–005 | ✅ |
| BC-4.10.001 | STORY-030 | AC-001–006 | ✅ |
| BC-4.10.002 | STORY-031 | AC-001–004 | ✅ |
| BC-4.10.003 | STORY-032 | AC-001–005 | ✅ |
| BC-5.11.001 | STORY-023 | AC-001 | ✅ |
| BC-5.11.002 | STORY-024 | AC-001–005 | ✅ |
| BC-5.11.003 | STORY-023 | AC-002–006 | ✅ |
| BC-5.12.001 | STORY-024 | AC-003–004 | ✅ |
| BC-5.12.002 | STORY-025 | AC-001–004 | ✅ |
| BC-6.13.001 | STORY-033 | AC-001–004 | ✅ |
| BC-6.13.002 | STORY-034 | AC-001–004 | ✅ |
| BC-6.14.001 | STORY-035 | AC-001–004 | ✅ |
| BC-6.14.002 | STORY-036 | AC-001–005 | ✅ |
| BC-6.15.001 | STORY-046 | AC-001–004 | ✅ |
| BC-6.15.002 | STORY-026 | AC-001–003 | ✅ |
| BC-7.16.001 | STORY-047 | AC-001–006 | ✅ |
| BC-7.16.002 | STORY-048 | AC-001–005 | ✅ |
| BC-7.16.003 | STORY-049 | AC-001–005 | ✅ |
| BC-7.16.004 | STORY-047 | AC-004 | ✅ |
| BC-7.17.001 | STORY-050 | AC-001,004 | ✅ |
| BC-7.17.002 | STORY-050 | AC-002–003 | ✅ |
| BC-7.17.003 | STORY-051 | AC-001–003 | ✅ |
| BC-7.18.001 | STORY-052, STORY-054 | AC-001–005 | ✅ |
| BC-7.18.002 | STORY-053 | AC-001–004 | ✅ |
| BC-7.18.003 | STORY-052 | AC-003–004 | ✅ |
| BC-8.19.001 | STORY-055 | AC-001–004 | ✅ |
| BC-8.19.002 | STORY-056 | AC-001–004 | ✅ |
| BC-8.19.003 | STORY-057 | AC-001–003 | ✅ |
| BC-8.20.001 | STORY-058 | AC-001–002,004 | ✅ |
| BC-8.20.002 | STORY-058 | AC-003 | ✅ |
| BC-9.21.001 | STORY-059 | AC-001–002 | ✅ |
| BC-9.21.002 | STORY-059 | AC-003–006 | ✅ |
| BC-9.22.001 | STORY-060 | AC-001–003 | ✅ |
| BC-10.23.001 | STORY-061 | AC-001–003 | ✅ |
| BC-10.24.001 | STORY-062 | AC-001–002 | ✅ |
| BC-10.25.001 | STORY-063 | AC-001–003 | ✅ |

**✅ BC Coverage: 65/65 (100%)**

---

## VP to Stories Matrix

| VP ID | Module | Stories Exercising It | BC Source |
|-------|--------|----------------------|-----------|
| VP-001 | forge-core | STORY-013, STORY-014, STORY-016, STORY-019 | BC-2.04.001 |
| VP-002 | forge-core | STORY-016, STORY-022 | BC-2.05.010 |
| VP-003 | forge-core | STORY-016 | BC-2.05.001 |
| VP-004 | forge-discovery | STORY-004, STORY-005 | BC-1.01.002 |
| VP-005 | forge-traffic | STORY-029 | BC-4.09.003 |
| VP-006 | forge-traffic | STORY-027, STORY-032 | BC-4.09.001 |
| VP-007 | forge-health | STORY-036 | BC-6.14.002 |
| VP-008 | forge-health | STORY-033 | BC-6.13.001 |
| VP-009 | forge-security | STORY-048, STORY-050 | BC-7.16.002 |
| VP-010 | forge-security | STORY-047 | BC-7.16.004 |
| VP-011 | forge-security | STORY-049 | BC-7.16.003 |
| VP-012 | forge-tui | STORY-037, STORY-039 | BC-3.07.001 |
| VP-013 | forge-core | STORY-007, STORY-008, STORY-009, STORY-013 | BC-1.02.003 |
| VP-014 | forge-traffic | STORY-030 | BC-4.10.001 |
| VP-015 | forge-daemon | STORY-010, STORY-011 | BC-1.03.001 |

**✅ VP Coverage: 15/15 (100%)**

---

## NFR to Stories Matrix

| NFR ID | Covering Stories | Validation Method |
|--------|-----------------|-------------------|
| NFR-001 | STORY-001, STORY-010, STORY-023, STORY-064 | hyperfine benchmark |
| NFR-002 | STORY-037, STORY-064 | TestBackend draw count |
| NFR-003 | STORY-024, STORY-064 | Token count test |
| NFR-004 | STORY-027, STORY-033 | Overhead benchmark |
| NFR-005 | STORY-047, STORY-052 | Corpus test |
| NFR-006 | STORY-052, STORY-050, STORY-048 | AST10 category audit |
| NFR-007 | STORY-049 | Hash-on-every-connection test |
| NFR-008 | STORY-001 | CI multi-platform matrix |
| NFR-009 | STORY-001, STORY-064 | CI binary size check |
| NFR-010 | STORY-056 | Method coverage assertion |
| NFR-011 | STORY-015, STORY-055 | Dual spec version tests |
| NFR-012 | STORY-029, STORY-064 | Load test 1000 msg/sec |
| NFR-013 | STORY-016 | Cursor loop detection |
| NFR-014 | STORY-007, STORY-013 | Code review |
| NFR-015 | STORY-044, STORY-045, STORY-043, STORY-054 | Accessibility audit |

**✅ NFR Coverage: 15/15 (100%)**

---

## Error Taxonomy Coverage

| Error Category | Error Codes | Covering Stories |
|----------------|------------|-----------------|
| CON (Connection) | E-CON-001–011 | STORY-007, STORY-008, STORY-009 |
| CFG (Config) | E-CFG-001–006 | STORY-004, STORY-005, STORY-006 |
| PRO (Protocol) | E-PRO-001–009 | STORY-013, STORY-016, STORY-021, STORY-022 |
| TUI (Terminal UI) | E-TUI-001–004 | STORY-037, STORY-038 |
| SEC (Security) | E-SEC-001–005 | STORY-047, STORY-049, STORY-053 |
| CAP (Capture) | E-CAP-001–003 | STORY-029, STORY-032 |
| MON (Monitoring) | E-MON-001–003 | STORY-034, STORY-036 |

**✅ Error Coverage: 7/7 categories covered**

---

## Holdout Scenario Assignment by Wave

| Wave | HS IDs | Scenarios |
|------|--------|-----------|
| Wave 1 | HS-012 | Cross-platform daemon lifecycle |
| Wave 2 | HS-008, HS-011 | Agent workflow token budget; Elicitation in CLI |
| Wave 3 | HS-002, HS-005, HS-009, HS-010 | Server crash; High volume; Pagination loop; Sampling failure |
| Wave 4 | HS-001, HS-017, HS-018 | Multi-server workflow; Minimal terminal; Concurrent TUI+CLI |
| Wave 5 | HS-003, HS-006, HS-014, HS-015, HS-016 | SSRF discovery; Rug pull; Suppression; Known-good corpus; CVE corpus |
| Wave 6 | HS-004, HS-007, HS-013 | Config drift 3+ editors; Incomplete conformance; Mixed proto versions |

---

## Gap Register

> All BCs, VPs, NFRs, and error codes are covered. No gaps requiring justification.

| Gap ID | Level | Source | Clause/Item | Justification | Resolution Target |
|--------|-------|--------|-------------|---------------|-------------------|
| GAP-001 | L2 | E-CON-010 (insecure HTTP) | Insecure HTTP is a warning not an error | Degraded severity by design; advisory only | v0.1.0 (by design) |
| GAP-002 | L2 | EC-007 (URL-encoded IP in SSRF) | URL decoding before SSRF detection not implemented | Documented as v0.1.0 scope limitation; attacker must use raw IP | v0.2.0 |
| GAP-003 | L3 | SCR-010 (Server Comparison TUI) | STORY-061–063 implement comparison but SCR-010 TUI not stories | P2 scope — CLI output sufficient for v0.1.0 | v0.2.0 |
