---
document_type: wave-schedule
version: "1.0"
status: draft
producer: story-writer
timestamp: 2026-03-29T16:00:00
phase: 2
traces_to: STORY-INDEX.md
cycle: v0.1.0-greenfield
---

# Wave Schedule — Forge MCP

> **65 stories across 7 waves (Wave 0–Wave 6).**
> Wave assignments are derived from the dependency DAG in `dependency-graph.md`.
> Parallelization within each wave is constrained by intra-wave dependencies.
> Story points: S=1, M=2, L=3, XL=5 (story index uses a finer scale; sizes below
> map the declared points directly for effort estimation).

---

## Summary

| Wave | Theme | Stories | Points | P0 | P1 | P2 | Holdout Scenarios |
|------|-------|---------|--------|----|----|----|--------------------|
| Wave 0 | Infrastructure Foundation | 3 | 8 | 3 | — | — | — |
| Wave 1 | Core Protocol + Discovery | 12 | 50 | 12 | — | — | HS-012 |
| Wave 2 | Protocol Operations + CLI | 10 | 51 | 10 | — | — | HS-008, HS-011 |
| Wave 3 | Traffic Inspection + Health Monitoring + Metric Export | 11 | 51 | 11 | — | — | HS-002, HS-005, HS-009, HS-010 |
| Wave 4 | TUI Dashboard | 10 | 49 | 10 | — | — | HS-001, HS-017, HS-018 |
| Wave 5 | Security Auditing | 8 | 39 | — | 8 | — | HS-003, HS-006, HS-014, HS-015, HS-016 |
| Wave 6 | Integration, NFR Validation & Cross-Cutting | 11 | 50 | 2 | 7 | 2 | HS-004, HS-007, HS-013 |
| **Total** | | **65** | **298** | **48** | **15** | **2** | **18 holdout scenarios** |

> Wave assignments exactly match STORY-INDEX.md column "Wave".
> STORY-026 (Metric Snapshot JSON Export) has been moved to Wave 3 because it depends
> on STORY-033 (Passive Latency & Throughput Metric Collection), which is also Wave 3.
> Wave 2 contains STORY-016 through STORY-025 (10 stories); Wave 3 contains STORY-026
> through STORY-036 (11 stories).

---

## Wave 0 — Infrastructure Foundation

**Theme:** Cargo workspace, CI pipeline, both DTU mock servers.
All 65 product stories compile against the workspace STORY-001 establishes.
Mock servers are DTU clones — they have no product dependencies themselves.

**Effort:** 8 story points
**Blocking:** Every subsequent story ultimately depends on STORY-001.

### Execution Plan

```
┌─────────────────────────────────────────────────────┐
│  SEQUENCE (must start first)                        │
│  STORY-001  Cargo Workspace Scaffold & CI Pipeline  │
│             [3 pts · P0]                            │
└───────────────────┬─────────────────────────────────┘
                    │ completes → unblocks both mocks
          ┌─────────┴──────────┐
          ▼                    ▼
┌──────────────────┐  ┌───────────────────────────────┐
│ PARALLEL GROUP A │  │ PARALLEL GROUP B               │
│ STORY-002        │  │ STORY-003                      │
│ Mock stdio server│  │ Mock HTTP server                │
│ [3 pts · P0]     │  │ [2 pts · P0]                   │
└──────────────────┘  └───────────────────────────────┘
```

| Step | Stories | Can Parallelize? | Notes |
|------|---------|-----------------|-------|
| 1 | STORY-001 | No (sole story) | Gate for wave; CI matrix must pass |
| 2 | STORY-002, STORY-003 | **Yes — run in parallel** | Both only need workspace; independent |

**Wave 0 exit gate:** Workspace compiles on all CI platforms; both mock servers
pass their own unit tests and respond to a basic `initialize` request.

---

## Wave 1 — Core Protocol + Discovery

**Theme:** Config discovery, dual-schema parsing, stdio + HTTP transport,
connection lifecycle, daemon, session pooling, and full capability negotiation.
No user-facing output in this wave — everything is library/daemon infrastructure.

**Effort:** 50 story points
**Key constraint:** STORY-013 (capability negotiation) is the critical-path bottleneck;
every Wave 2+ story depends on it.

### Execution Plan

```
┌──────────────────────────────────────────────────────────────────┐
│ SEQUENCE CHAIN A — Config Pipeline                               │
│ STORY-004 → STORY-005 → STORY-006                               │
│   [5]          [5]          [3]                                  │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│ PARALLEL GROUP B — Transport Layer (start after STORY-004)       │
│ STORY-007 (stdio) ║ STORY-008 (HTTP)                            │
│   [5]              ║   [5]                                       │
└──────────────────────────────────────────────────────────────────┘
         │                     │
         └──────────┬──────────┘
                    ▼  (both complete)
┌──────────────────────────────────────────────────────────────────┐
│ SEQUENCE CHAIN C — Daemon Stack                                  │
│ STORY-009 → STORY-010                                           │
│   [3]          [5]                                               │
│                 ├── STORY-011 (parallel)  [3]                    │
│                 └── STORY-012 (parallel)  [3]                    │
└──────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────┐
│ SEQUENCE CHAIN D — Protocol Negotiation (start after 007+008)    │
│ STORY-013 → STORY-014                                           │
│   [5]          [5]                                               │
│                 ├── STORY-019 (parallel, Wave 2)                 │
│                 └── STORY-020 (parallel, Wave 2)                 │
│ STORY-013 → STORY-015  [3] (parallel with STORY-014)           │
└──────────────────────────────────────────────────────────────────┘
```

| Step | Stories | Can Parallelize? | Dependency Notes |
|------|---------|-----------------|-----------------|
| 1a | STORY-004 | No | Needs STORY-001 (workspace) |
| 1b | STORY-007, STORY-008 | **Yes — parallel with each other** | Both need STORY-004 (config types); start same time as or after STORY-004 |
| 2 | STORY-005 | No (after STORY-004) | Config parser extends discovery types |
| 3 | STORY-006 | No (after STORY-005) | Aggregation extends parser |
| 4a | STORY-009 | No (after STORY-007 AND STORY-008 complete) | Lifecycle spans both transports |
| 4b | STORY-013 | No (after STORY-007 AND STORY-008 complete) | Negotiation needs both transport types |
| 5 | STORY-010 | No (after STORY-009) | Daemon builds on lifecycle |
| 6a | STORY-011, STORY-012 | **Yes — parallel with each other** | Both only need STORY-010 |
| 6b | STORY-014, STORY-015 | **Yes — parallel with each other** | Both only need STORY-013 |

**Recommended agent allocation:** 2 parallel agents can be used starting at step 1b.
The tightest serial chain is: `001→004→007→009→013` (5 stories, 23 pts in series).

**Wave 1 exit gate:** `forge discover` returns server list; `forge info <server>`
connects and returns capability summary; daemon starts on first invocation.

---

## Wave 2 — Protocol Operations + CLI Mode

**Theme:** Full MCP protocol operations (tools, resources, prompts, sampling,
elicitation), CLI subcommand dispatch, structured JSON output, and metric export.
First wave with user-visible output.

**Effort:** 51 story points
**Key constraint:** All stories in this wave depend on STORY-013 (negotiation).
STORY-016 (tool list) is on the critical path — STORY-022 and STORY-045 depend on it.

### Execution Plan

```
After STORY-013 completes:

┌─────────────────────────────────────────────────────────────────────────┐
│ PARALLEL GROUP A — Protocol Operations (all depend only on STORY-013)   │
│ STORY-016  [8]  ← highest priority; STORY-022, STORY-045 block on this │
│ STORY-017  [5]                                                          │
│ STORY-018  [5]                                                          │
│ STORY-021  [5]                                                          │
└─────────────────────────────────────────────────────────────────────────┘
         │
         ▼  (STORY-016 completes)
┌──────────────────────────────────────────────────────────────────────────┐
│ STORY-022  Progress Tracking & Cancellation  [5]                         │
└──────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│ PARALLEL GROUP B — Client Capability Operations (need STORY-014)        │
│ STORY-019  Sampling Proxy  [5]                                          │
│ STORY-020  Elicitation     [5]                                          │
└─────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────┐
│ SEQUENCE CHAIN C — CLI Pipeline (need STORY-013)                        │
│ STORY-023 → STORY-024 → STORY-025                                       │
│   [5]          [5]          [3]                                         │
└─────────────────────────────────────────────────────────────────────────┘
```

| Step | Stories | Can Parallelize? | Dependency Notes |
|------|---------|-----------------|-----------------|
| 1 | STORY-016, STORY-017, STORY-018, STORY-021 | **Yes — all parallel** | All need only STORY-013 |
| 1b | STORY-019, STORY-020 | **Yes — parallel with group A and each other** | Need STORY-014 (Wave 1) |
| 1c | STORY-023 | **Yes — parallel with groups A and B** | Needs STORY-013 |
| 2 | STORY-022 | No (after STORY-016) | Needs tool invocation |
| 3 | STORY-024 | No (after STORY-023) | |
| 4 | STORY-025 | No (after STORY-024) | |

**Recommended agent allocation:** Up to 6 parallel agents in step 1 (groups A + B + C first story).

**Wave 2 exit gate:** `forge list tools <server>` returns paginated tool list;
`forge call <tool>` executes and returns structured JSON; `forge <subcmd> | jq` works
end-to-end.

---

## Wave 3 — Traffic Inspection + Health Monitoring

**Theme:** JSON-RPC message capture, timing/throughput analysis, bounded ring buffer,
traffic filtering, full-text search, replay, passive metric collection, error rate
tracking, configurable alerting, and alert state machine.

**Effort:** 51 story points
**Key constraint:** STORY-027 (capture) is the root of both the traffic and health
subtrees. STORY-033 (metrics) is on the critical path to the TUI sparklines (Wave 4).
STORY-026 (Metric Snapshot CLI Export) is also in this wave because it depends on
STORY-033; it can run in parallel with the rest of Wave 3 once STORY-023 (Wave 2)
and STORY-033 both complete.

### Execution Plan

```
After STORY-013 completes (from Wave 1):

┌──────────────────────────────────────────────────────┐
│ STORY-027  Transparent JSON-RPC Message Capture  [5] │
│ (single entry point — all Wave 3 stories depend on it)│
└──────────────────┬───────────────────────────────────┘
                   │
         ┌─────────┴──────────────────────────────┐
         ▼                                        ▼
┌─────────────────────────┐           ┌────────────────────────────┐
│ PARALLEL GROUP A        │           │ PARALLEL GROUP B           │
│ (all depend on 027 only)│           │ (all depend on 027 only)   │
│ STORY-028 Timing  [5]   │           │ STORY-033 Metrics  [5] ←CP │
│ STORY-029 Buffer  [5]   │           └───────────┬────────────────┘
└──────┬──────────────────┘                       │
       │ (STORY-029 completes)           ┌─────────┴────────────┐
       ▼                                 ▼                      ▼
┌────────────────────────┐    ┌────────────────┐    ┌──────────────────────┐
│ PARALLEL GROUP C       │    │ STORY-034      │    │ STORY-035            │
│ (need 029)             │    │ Error Rate [5] │    │ Alert Thresholds [5] │
│ STORY-030 Filter  [5]  │    └────────────────┘    └──────────┬───────────┘
│ STORY-031 Search  [3]  │                                     │
│ STORY-032 Replay  [5]  │                          ┌──────────┴───────────┐
└────────────────────────┘                          │ STORY-036            │
                                                    │ Alert State Machine  │
                                                    │ [5] ←CRITICAL PATH   │
                                                    └──────────────────────┘
```

| Step | Stories | Can Parallelize? | Dependency Notes |
|------|---------|-----------------|-----------------|
| 1 | STORY-027 | No (sole entry) | Needs STORY-013 (Wave 1) |
| 2 | STORY-028, STORY-029, STORY-033 | **Yes — all parallel** | All need only STORY-027 |
| 3a | STORY-030, STORY-031, STORY-032 | **Yes — parallel** | All need STORY-029 |
| 3b | STORY-034, STORY-035 | **Yes — parallel with each other and group 3a** | Need STORY-033 |
| 3c | STORY-026 | **Yes — parallel with group 3b** | Needs STORY-023 (W2) + STORY-033; starts once both done |
| 4 | STORY-036 | No (after STORY-035) | Alert SM builds on threshold events |

**Wave 3 exit gate:** `forge traffic inspect <server>` streams live messages; buffer
does not exceed configured MB limit under 1000 msg/sec; `forge health` returns latency
p50/p95 and current alert state.

---

## Wave 4 — TUI Dashboard

**Theme:** ratatui-based interactive TUI. Main layout, color system, vi navigation,
search/command mode, mouse input, JSON-RPC rendering, health sparklines, server
browser, capability explorer, and time-series metric visualization.

**Effort:** 49 story points
**Key constraint:** STORY-037 (layout) is the sole entry point; all Wave 4 stories
depend on it. STORY-043 (sparklines) is on the critical path to STORY-046.
STORY-045 (cap explorer) has the most dependencies (needs STORY-016–018 from Wave 2).

### Execution Plan

```
After STORY-006 (Wave 1) + STORY-013 (Wave 1) complete:

┌────────────────────────────────────────────────────────────────┐
│ STORY-037  TUI Main Dashboard Layout & Adaptive Panes  [8]     │
│ ←CRITICAL PATH — all Wave 4 stories gate on this              │
└───────────────────────────────┬────────────────────────────────┘
                                │
           ┌────────────────────┼────────────────────┐
           ▼                    ▼                    ▼
┌──────────────────┐  ┌─────────────────────┐  ┌──────────────────────┐
│ PARALLEL A       │  │ PARALLEL B           │  │ PARALLEL C           │
│ (need 037 only)  │  │ (need 037 only)      │  │ (need 037 only)      │
│ STORY-038 Color  │  │ STORY-039 Navigation │  │ STORY-044 Server     │
│ [3]              │  │ [5] ←CP             │  │ Browser [5]          │
└──────────────────┘  └────────┬────────────┘  └──────────────────────┘
                               │
                    ┌──────────┴──────────┐
                    ▼                     ▼
           ┌──────────────────┐  ┌──────────────────────┐
           │ STORY-040        │  │ STORY-041            │
           │ Search & Command │  │ Mouse Input [3]      │
           │ Mode [5]         │  │                      │
           └──────────────────┘  └──────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│ PARALLEL D — Data-Source Panes (need STORY-037 + data source)   │
│ STORY-042 JSON-RPC Rendering [5] — also needs STORY-027 (W3)   │
│ STORY-043 Sparklines [5] ←CP    — also needs STORY-036 (W3)   │
│ STORY-045 Cap Explorer [5]       — also needs 016,017,018 (W2)  │
└──────────────────────┬──────────────────────────────────────────┘
                       │ (STORY-043 completes)
                       ▼
              ┌─────────────────────────┐
              │ STORY-046               │
              │ Time-Series Metric Viz  │
              │ [5]   — also needs 033  │
              └─────────────────────────┘
```

| Step | Stories | Can Parallelize? | Dependency Notes |
|------|---------|-----------------|-----------------|
| 1 | STORY-037 | No (sole entry) | Needs STORY-006 (W1) + STORY-013 (W1) |
| 2 | STORY-038, STORY-039, STORY-044 | **Yes — all parallel** | All need only STORY-037 |
| 2b | STORY-042, STORY-043, STORY-045 | **Yes — parallel with each other and group 2** | Need STORY-037 + respective data sources already done (W2/W3) |
| 3 | STORY-040, STORY-041 | **Yes — parallel** | Both need STORY-039 |
| 4 | STORY-046 | No (after STORY-043 AND STORY-033) | Time-series needs sparkline component + metric data |

**Wave 4 exit gate:** `forge tui` launches without panic in 80×24 terminal; all 6 P0
UX screens render (SCR-001–006); vi navigation switches panes; `q` exits cleanly;
health sparklines update in real time.

---

## Wave 5 — Security Auditing

**Theme:** Dangerous tool pattern detection, SSRF detection, schema drift/rug pull
detection, permission escalation, authentication validation, structured audit report
with OWASP AST10 mapping, suppression rules, and TUI security view.

**Effort:** 39 story points
**Key constraint:** All detectors depend on STORY-027 (captured messages, Wave 3).
STORY-052 (report generation) requires all five detectors to complete first.
STORY-054 (TUI view) requires both the report and STORY-037 (Wave 4 layout).

### Execution Plan

```
After STORY-027 (Wave 3) completes:

┌─────────────────────────────────────────────────────────────────────┐
│ PARALLEL GROUP A — Independent Detectors (all depend only on 027)   │
│ STORY-047  Dangerous Tool Pattern Detection  [5]                    │
│ STORY-048  SSRF Attempt Detection           [5]                    │
│ STORY-049  Schema Drift / Rug Pull Detection [5]                    │
└──────────────┬──────────┬──────────────────────────────────────────┘
               │          │
               ▼          ▼
      ┌──────────────┐  ┌──────────────────────────────────────┐
      │ STORY-050    │  │ STORY-051                            │
      │ Permission   │  │ Auth Validation [3]                  │
      │ Escalation   │  │ (depends on STORY-048)               │
      │ [5]          │  └────────────────────────────────────┬─┘
      │ (dep: 047)   │                                       │
      └──────┬───────┘                                       │
             └──────────────────┬────────────────────────────┘
                                ▼  (all of 047–051 complete)
              ┌──────────────────────────────────────────────────────┐
              │ STORY-052  Security Audit Report & OWASP Mapping [8] │
              └──────────────────────┬───────────────────────────────┘
                                     │
                          ┌──────────┴──────────┐
                          ▼                     ▼
                 ┌─────────────────┐   ┌───────────────────────────────┐
                 │ STORY-053       │   │ STORY-054                     │
                 │ Suppression [3] │   │ TUI Security View [5]         │
                 └────────┬────────┘   │ (needs 052 + 053 + 037)      │
                          │            └───────────────────────────────┘
                          └──────────────────────────────────────────────▶
                           (STORY-054 cannot start until 053 completes)
```

| Step | Stories | Can Parallelize? | Dependency Notes |
|------|---------|-----------------|-----------------|
| 1 | STORY-047, STORY-048, STORY-049 | **Yes — all parallel** | All need STORY-027 (W3) |
| 2 | STORY-050, STORY-051 | **Yes — parallel with each other** | STORY-050 needs 047; STORY-051 needs 048 |
| 3 | STORY-052 | No (after 047 + 048 + 049 + 050 + 051) | Report aggregates all detectors |
| 4 | STORY-053 | No (after STORY-052) | Suppression operates on generated reports |
| 5 | STORY-054 | No (after STORY-052 + STORY-053 + STORY-037) | TUI view needs full pipeline + layout |

**Wave 5 exit gate:** `forge audit <server>` returns a structured JSON report with
OWASP AST10 mappings; precision ≥80% on test corpus; rug-pull scenario detected;
`forge tui` shows SCR-007 security pane.

---

## Wave 6 — Integration, NFR Validation & Cross-Cutting Concerns

**Theme:** Protocol conformance testing, config drift detection, server comparison,
NFR performance benchmarks, and SR spec refinements. Mostly P1/P2 features
plus the final cross-cutting validation story.

**Effort:** 31 story points

> **Note on story count:** 11 stories (STORY-055–065). Wave 6 points are lower per
> story than earlier waves because these are focused, narrow-scope stories.

### Execution Plan

```
After STORY-013 (W1) + STORY-006 (W1) + STORY-016 (W2) + STORY-052/053 (W5) complete:

┌──────────────────────────────────────────────────────────────────────────┐
│ PARALLEL GROUP A — Conformance Suite (depends on STORY-013 + STORY-002) │
│ STORY-055  Conformance Cap Negotiation  [5]                              │
└────────────────────────────┬─────────────────────────────────────────────┘
                             │
              ┌──────────────┼─────────────────┐
              ▼              ▼                 ▼
   ┌───────────────┐  ┌──────────────┐  ┌──────────────┐
   │ STORY-056 [5] │  │ STORY-057 [3]│  │ STORY-058 [5]│
   │ Method        │  │ Transport    │  │ Conformance  │
   │ Coverage      │  │ Compliance   │  │ Reports      │
   │ (dep: 055)    │  │ (dep:055+003)│  │ (dep:055-057)│
   └───────────────┘  └──────────────┘  └──────────────┘

┌──────────────────────────────────────────────────────────────────────────┐
│ PARALLEL GROUP B — Config Drift (depends on STORY-006)                  │
│ STORY-059  Cross-Editor Config Comparison [8]                            │
│ → STORY-060  Reconciliation Guidance [3]  (sequential after 059)        │
└──────────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────────┐
│ PARALLEL GROUP C — Server Comparison (P2)                                │
│ STORY-061  Server Tool Schema Diff  [5]  (deps: 016+006)                │
│ STORY-062  Capability Set Delta     [3]  (dep: 013)                     │
│ STORY-063  Response Behavior Delta  [5]  (dep: 016)                     │
└──────────────────────────────────────────────────────────────────────────┘

┌──────────────────────────────────────────────────────────────────────────┐
│ PARALLEL GROUP D — NFR + SR Refinements                                  │
│ STORY-064  NFR Validation Suite [5]  (deps: 024+033+029)                │
│ STORY-065  SR Refinements [3]        (deps: 005+047)                    │
└──────────────────────────────────────────────────────────────────────────┘
```

| Step | Stories | Can Parallelize? | Dependency Notes |
|------|---------|-----------------|-----------------|
| 1 | STORY-055 | No (sole entry for conformance) | Needs STORY-013 (W1) + STORY-002 (W0) |
| 1b | STORY-059 | **Yes — parallel with STORY-055** | Needs STORY-006 (W1) |
| 1c | STORY-061, STORY-062, STORY-063 | **Yes — parallel with 055, 059, each other** | 061 needs 016+006; 062 needs 013; 063 needs 016 |
| 1d | STORY-064, STORY-065 | **Yes — parallel with all of step 1** | 064 needs 024+033+029; 065 needs 005+047 |
| 2 | STORY-056, STORY-057 | **Yes — parallel** | Both need STORY-055 |
| 3 | STORY-058 | No (after STORY-055 + 056 + 057) | Reports need all conformance suites |
| 3b | STORY-060 | No (after STORY-059) | Reconciliation extends drift report |

**Maximum parallelism in Wave 6:** Groups A–D can all start simultaneously
(all their prerequisites are complete by end of Wave 5). Up to 8 stories
can run in parallel in step 1.

**Wave 6 exit gate:** All 18 holdout scenarios pass with ≥70% score;
NFR benchmarks pass (cold start <500ms, TUI 60fps, buffer <10MB at 1000 msg/sec);
conformance report generates valid JUnit XML; CI pipeline is green on all platforms.

---

## Holdout Scenario Wave Mapping

> Holdout scenarios are evaluated **after** the wave whose stories implement the
> tested behaviors. Evaluator runs against the completed binary — not the source.

| Wave | HS IDs | Scenario Title | Category | Priority |
|------|--------|----------------|----------|----------|
| Wave 0 | (none) | — | — | — |
| Wave 1 | HS-012 | Cross-platform daemon lifecycle | Daemon | P0 |
| Wave 2 | HS-008 | Agent workflow: discover → inspect → call in <500 tokens | Agent UX | P1 |
| Wave 2 | HS-011 | Elicitation request in non-interactive CLI mode | Edge Case | P1 |
| Wave 3 | HS-002 | Server crash during TUI traffic inspection | Resilience | P0 |
| Wave 3 | HS-005 | Performance under high message volume (1000+ msg/sec) | Performance | P1 |
| Wave 3 | HS-009 | Pagination with misbehaving server (cursor loop) | Resilience | P1 |
| Wave 3 | HS-010 | Sampling proxy failure with graceful degradation | Resilience | P1 |
| Wave 4 | HS-001 | Multi-server workflow: discover, inspect, compare | Integration | P0 |
| Wave 4 | HS-017 | TUI with minimal terminal (80×24, 16-color, no Unicode) | Accessibility | P1 |
| Wave 4 | HS-018 | Concurrent TUI and CLI access to same server | Concurrency | P1 |
| Wave 5 | HS-003 | Security audit discovers actual SSRF vulnerability | Security | P0 |
| Wave 5 | HS-006 | Rug pull attack: tool schema changes between connections | Security | P0 |
| Wave 5 | HS-014 | Security finding suppression and re-classification | Security | P1 |
| Wave 5 | HS-015 | Known-good corpus: filesystem server (false-positive test) | Corpus | P0 |
| Wave 5 | HS-016 | Known-problematic corpus: server with known CVEs | Corpus | P0 |
| Wave 6 | HS-004 | Config drift across 3+ editors with dual schema | Config | P0 |
| Wave 6 | HS-007 | Conformance test against incomplete server | Conformance | P1 |
| Wave 6 | HS-013 | Mixed protocol version environment | Protocol | P1 |

---

## Critical Path

The **critical path** is the longest chain of dependent stories across all waves.
This chain determines the earliest possible completion date; any slip on these
stories delays the overall delivery.

```
Wave 0          Wave 1                              Wave 3          Wave 4
STORY-001  →  STORY-002  →  STORY-007  →  STORY-013  →  STORY-027  →  STORY-033
  [3]            [3]           [5]           [5]           [5]           [5]
                                                                          │
                                                               ┌──────────┘
                                                               ▼
                                                          STORY-035  →  STORY-036  →  STORY-037  →  STORY-043  →  STORY-046
                                                            [5]           [5]           [8]           [5]           [5]
```

**Full critical path (12 stories, Wave 0 → Wave 4):**

| # | Story | Title | Wave | Points |
|---|-------|-------|------|--------|
| 1 | STORY-001 | Cargo Workspace Scaffold & CI Pipeline | 0 | 3 |
| 2 | STORY-002 | Mock MCP Server (stdio) | 0 | 3 |
| 3 | STORY-007 | Stdio Transport Connection Establishment | 1 | 5 |
| 4 | STORY-013 | Bidirectional Capability Negotiation | 1 | 5 |
| 5 | STORY-027 | Transparent JSON-RPC Message Capture | 3 | 5 |
| 6 | STORY-033 | Passive Latency & Throughput Metric Collection | 3 | 5 |
| 7 | STORY-035 | Configurable Alerting Thresholds | 3 | 5 |
| 8 | STORY-036 | Alert State Machine (Normal → Breached → Recovered) | 3 | 5 |
| 9 | STORY-037 | TUI Main Dashboard Layout & Adaptive Panes | 4 | 8 |
| 10 | STORY-043 | Sparkline & Histogram Health Metric Visualization | 4 | 5 |
| 11 | STORY-046 | Time-Series Metric Visualization in TUI | 4 | 5 |

**Critical path length:** 11 stories (the dependency-graph.md identifies 12 stories;
this wave schedule corrects the count by collapsing the `001→002` sub-chain which
runs in parallel after `001`, not strictly as a serial predecessor of `007`;
both STORY-002 and STORY-007 must complete before STORY-013, but `001→002→007→013`
is still the binding serial path because `007` requires `002`).

**Critical path points:** 54 story points in serial
**Minimum wave span:** Wave 0 → Wave 4 (5 waves)
**Critical path is complete by end of Wave 4.** Waves 5 and 6 are not on the
critical path — they can begin as soon as their respective Wave 3–4 prerequisites
are satisfied.

---

## Effort Per Wave

> Story point mapping: declared points used directly from STORY-INDEX.md.
> For reference: the index uses a finer scale where S≈1-3, M≈5, L≈8, XL≈13;
> this schedule preserves those exact values.

| Wave | Stories | Individual Points | Total Points | Cumulative |
|------|---------|-------------------|--------------|------------|
| Wave 0 | STORY-001(3), STORY-002(3), STORY-003(2) | 3+3+2 | **8 pts** | 8 |
| Wave 1 | STORY-004(5), STORY-005(5), STORY-006(3), STORY-007(5), STORY-008(5), STORY-009(3), STORY-010(5), STORY-011(3), STORY-012(3), STORY-013(5), STORY-014(5), STORY-015(3) | — | **50 pts** | 58 |
| Wave 2 | STORY-016(8), STORY-017(5), STORY-018(5), STORY-019(5), STORY-020(5), STORY-021(5), STORY-022(5), STORY-023(5), STORY-024(5), STORY-025(3) | — | **51 pts** | 109 |
| Wave 3 | STORY-026(3), STORY-027(5), STORY-028(5), STORY-029(5), STORY-030(5), STORY-031(3), STORY-032(5), STORY-033(5), STORY-034(5), STORY-035(5), STORY-036(5) | — | **51 pts** | 160 |
| Wave 4 | STORY-037(8), STORY-038(3), STORY-039(5), STORY-040(5), STORY-041(3), STORY-042(5), STORY-043(5), STORY-044(5), STORY-045(5), STORY-046(5) | — | **49 pts** | 209 |
| Wave 5 | STORY-047(5), STORY-048(5), STORY-049(5), STORY-050(5), STORY-051(3), STORY-052(8), STORY-053(3), STORY-054(5) | — | **39 pts** | 248 |
| Wave 6 | STORY-055(5), STORY-056(5), STORY-057(3), STORY-058(5), STORY-059(8), STORY-060(3), STORY-061(5), STORY-062(3), STORY-063(5), STORY-064(5), STORY-065(3) | — | **50 pts** | 298 |

> All point totals are derived from the per-story declared values in the Full Story
> Registry (STORY-INDEX.md). Grand total: 298 story points across 65 stories.

---

## Parallelization Summary

| Wave | Max Parallel Stories | Parallel Groups | Serial Bottlenecks |
|------|---------------------|-----------------|-------------------|
| Wave 0 | 2 | STORY-002 ∥ STORY-003 | STORY-001 (gate) |
| Wave 1 | 4 | {007,008} ∥ {011,012} ∥ {014,015} | STORY-001→004→007→009→010; STORY-013 |
| Wave 2 | 6 | {016,017,018,021} ∥ {019,020} ∥ {023} | STORY-016→022; STORY-023→024→025 |
| Wave 3 | 3 | {028,029,033} ∥ {030,031,032} ∥ {034,035,026} | STORY-027→buffer/metrics; STORY-035→036; STORY-026 after 023+033 |
| Wave 4 | 6 | {038,039,044} ∥ {042,043,045} | STORY-037 (gate); STORY-039→{040,041}; STORY-043→046 |
| Wave 5 | 3 | {047,048,049} | STORY-052 (aggregation gate); STORY-052→053→054 |
| Wave 6 | 8 | {055,059,061,062,063,064,065} start together | STORY-055→{056,057}→058; STORY-059→060 |

---

## Complete Story-to-Wave Reference

| Story | Title | Wave | Points | Priority | Parallel Group |
|-------|-------|------|--------|----------|---------------|
| STORY-001 | Cargo Workspace Scaffold & CI Pipeline | 0 | 3 | P0 | Serial gate |
| STORY-002 | Mock MCP Server (stdio) | 0 | 3 | P0 | W0-parallel |
| STORY-003 | Mock MCP Server (HTTP) | 0 | 2 | P0 | W0-parallel |
| STORY-004 | Config File Discovery & Path Resolution | 1 | 5 | P0 | Serial (config chain) |
| STORY-005 | Dual-Schema Config Parsing | 1 | 5 | P0 | Serial (config chain) |
| STORY-006 | Config Source Aggregation & Conflict Attribution | 1 | 3 | P0 | Serial (config chain) |
| STORY-007 | Stdio Transport Connection Establishment | 1 | 5 | P0 | W1-transport-parallel |
| STORY-008 | Streamable HTTP Transport Connection Establishment | 1 | 5 | P0 | W1-transport-parallel |
| STORY-009 | Connection Lifecycle Management | 1 | 3 | P0 | Serial (daemon chain) |
| STORY-010 | Daemon Lazy Start & Session Pooling | 1 | 5 | P0 | Serial (daemon chain) |
| STORY-011 | Named Session Persistence Across CLI Invocations | 1 | 3 | P0 | W1-daemon-parallel |
| STORY-012 | Daemon Socket Conflict Detection & Recovery | 1 | 3 | P0 | W1-daemon-parallel |
| STORY-013 | Bidirectional Capability Negotiation | 1 | 5 | P0 | Serial (critical path) |
| STORY-014 | Client Capability Advertisement | 1 | 5 | P0 | W1-negotiation-parallel |
| STORY-015 | Graceful Degradation with Older Spec Versions | 1 | 3 | P0 | W1-negotiation-parallel |
| STORY-016 | Tool List & Invocation with Pagination | 2 | 8 | P0 | W2-ops-parallel (highest priority) |
| STORY-017 | Resource List, Read & Subscription Management | 2 | 5 | P0 | W2-ops-parallel |
| STORY-018 | Prompt List & Retrieval with Pagination | 2 | 5 | P0 | W2-ops-parallel |
| STORY-019 | Sampling Proxy to External LLM | 2 | 5 | P0 | W2-caps-parallel |
| STORY-020 | Elicitation Request Handling | 2 | 5 | P0 | W2-caps-parallel |
| STORY-021 | Roots, Logging, Completion & Protocol Utilities | 2 | 5 | P0 | W2-ops-parallel |
| STORY-022 | Progress Tracking, Cancellation & Error Distinction | 2 | 5 | P0 | Serial (after 016) |
| STORY-023 | CLI Subcommand Dispatch & Exit Code Semantics | 2 | 5 | P0 | W2-cli-parallel |
| STORY-024 | Structured JSON Output & Agent-Optimized Tokens | 2 | 5 | P0 | Serial (CLI chain) |
| STORY-025 | Pipeable Output & Shell Composition | 2 | 3 | P0 | Serial (CLI chain) |
| STORY-026 | Metric Snapshot JSON Export via CLI | 3 | 3 | P0 | Parallel with W3-metrics (after 023+033) |
| STORY-027 | Transparent JSON-RPC Message Capture | 3 | 5 | P0 | Serial gate (traffic) |
| STORY-028 | Per-Message Timing & Throughput Analysis | 3 | 5 | P0 | W3-capture-parallel |
| STORY-029 | Capture Buffer Management with Bounded Memory | 3 | 5 | P0 | W3-capture-parallel |
| STORY-030 | Traffic Filtering by Method, Direction, Time & Content | 3 | 5 | P0 | W3-filter-parallel |
| STORY-031 | Full-Text Payload Search | 3 | 3 | P0 | W3-filter-parallel |
| STORY-032 | Message Sequence Replay | 3 | 5 | P0 | W3-filter-parallel |
| STORY-033 | Passive Latency & Throughput Metric Collection | 3 | 5 | P0 | W3-capture-parallel (critical path) |
| STORY-034 | Error Rate Trend Tracking | 3 | 5 | P0 | W3-metrics-parallel |
| STORY-035 | Configurable Alerting Thresholds | 3 | 5 | P0 | W3-metrics-parallel |
| STORY-036 | Alert State Machine (Normal → Breached → Recovered) | 3 | 5 | P0 | Serial (after 035) |
| STORY-037 | TUI Main Dashboard Layout & Adaptive Panes | 4 | 8 | P0 | Serial gate (TUI) |
| STORY-038 | Color System Auto-Detection & Degradation | 4 | 3 | P0 | W4-layout-parallel |
| STORY-039 | Vi-Style Keyboard Navigation | 4 | 5 | P0 | W4-layout-parallel (critical path) |
| STORY-040 | Search & Command Mode (/ and :) | 4 | 5 | P0 | W4-nav-parallel |
| STORY-041 | Mouse Supplementary Input | 4 | 3 | P0 | W4-nav-parallel |
| STORY-042 | JSON-RPC Syntax-Highlighted Message Rendering | 4 | 5 | P0 | W4-panes-parallel |
| STORY-043 | Sparkline & Histogram Health Metric Visualization | 4 | 5 | P0 | W4-panes-parallel (critical path) |
| STORY-044 | Server Browser with Status Badges | 4 | 5 | P0 | W4-layout-parallel |
| STORY-045 | Capability Explorer (Tools/Resources/Prompts Tree) | 4 | 5 | P0 | W4-panes-parallel |
| STORY-046 | Time-Series Metric Visualization in TUI | 4 | 5 | P0 | Serial (after 043+033) |
| STORY-047 | Dangerous Tool Pattern Detection | 5 | 5 | P1 | W5-detectors-parallel |
| STORY-048 | SSRF Attempt Detection | 5 | 5 | P1 | W5-detectors-parallel |
| STORY-049 | Schema Drift / Rug Pull Detection | 5 | 5 | P1 | W5-detectors-parallel |
| STORY-050 | Permission Escalation & Root Enforcement Detection | 5 | 5 | P1 | Serial (after 047) |
| STORY-051 | Authentication Handling Validation | 5 | 3 | P1 | Serial (after 048) |
| STORY-052 | Security Audit Report Generation & OWASP Mapping | 5 | 8 | P1 | Serial (after 047–051) |
| STORY-053 | Security Finding Suppression Rules | 5 | 3 | P1 | Serial (after 052) |
| STORY-054 | Security Audit TUI View (SCR-007) | 5 | 5 | P1 | Serial (after 052+053+037) |
| STORY-055 | Conformance Capability Negotiation Validation | 6 | 5 | P1 | W6-conformance-serial-gate |
| STORY-056 | Conformance Method Coverage & Error Handling | 6 | 5 | P1 | W6-conformance-parallel |
| STORY-057 | Conformance Transport Compliance | 6 | 3 | P1 | W6-conformance-parallel |
| STORY-058 | Conformance Report Output (JUnit XML + JSON) | 6 | 5 | P1 | Serial (after 055–057) |
| STORY-059 | Cross-Editor Config Comparison & Drift Report | 6 | 8 | P1 | W6-drift-gate |
| STORY-060 | Config Drift Reconciliation Workflow Guidance | 6 | 3 | P2 | Serial (after 059) |
| STORY-061 | Server Tool Schema Diff | 6 | 5 | P2 | W6-comparison-parallel |
| STORY-062 | Capability Set Delta Comparison | 6 | 3 | P2 | W6-comparison-parallel |
| STORY-063 | Response Behavior Delta Testing | 6 | 5 | P2 | W6-comparison-parallel |
| STORY-064 | NFR Validation Suite & Performance Benchmarks | 6 | 5 | P0 | W6-nfr-parallel |
| STORY-065 | SR Refinements: Config Struct Clarity & Security Heuristics | 6 | 3 | P0 | W6-nfr-parallel |
