---
document_type: ux-spec-screen
screen_id: SCR-008
screen_name: Conformance Test Runner
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-03-29T14:15:00
phase: 1c
complexity: complex
traces_to: UX-INDEX.md
prd_requirements: [BC-8.19.001, BC-8.19.002, BC-8.19.003, BC-8.20.001, BC-8.20.002]
priority: P1
---

# Screen: Conformance Test Runner (SCR-008)

> Full content-area tab for running the MCP protocol conformance test suite
> against the active server. Shows test suite progress with pass/fail indicators,
> detailed per-test results, and export in JUnit XML / JSON formats for CI/CD.

---

## Wireframe

### Test Suite Progress (Running)

```
╔══════════════════════════════════════════════════════════════════════════╗
║  CONFORMANCE — my-server  [● RUNNING]  MCP 2025-11-25                    ║
║  ──────────────────────────────────────────────────────────────────────  ║
║  ████████████████████████████░░░░░░░░░░░  71%  43 / 60 tests            ║
║                                                                          ║
║  SUITE               PASS  FAIL  SKIP  STATUS                           ║
║  ──────────────────  ────  ────  ────  ──────────────────────────────── ║
║  ✓ Lifecycle         4     0     0    PASS                              ║
║  ✓ Capability Neg.   6     0     0    PASS                              ║
║  ✓ Tools             12    0     0    PASS                              ║
║  ✓ Resources         8     0     0    PASS                              ║
║  ◌ Prompts           8     0     2    RUNNING…                          ║
║  ○ Error Handling    —     —     —    PENDING                           ║
║  ○ Transport         —     —     —    PENDING                           ║
║                                                                          ║
║  Current: prompts/get (test 45/60) — 2.3s                               ║
║  [ Esc: Cancel ]                                                         ║
╚══════════════════════════════════════════════════════════════════════════╝
```

### Test Suite Results (Complete — Pass)

```
╔══════════════════════════════════════════════════════════════════════════╗
║  CONFORMANCE — my-server  [✓ PASS]  MCP 2025-11-25  │  58/60 pass      ║
║  ──────────────────────────────────────────────────────────────────────  ║
║  SUITE               PASS  FAIL  SKIP  STATUS                           ║
║  ──────────────────  ────  ────  ────  ──────────────────────────────── ║
║  ✓ Lifecycle         4     0     0    PASS                              ║
║  ✓ Capability Neg.   6     0     0    PASS                              ║
║  ✓ Tools             12    0     0    PASS                              ║
║  ✓ Resources         8     0     0    PASS                              ║
║  ✓ Prompts           10    0     0    PASS                              ║
║  ✓ Error Handling    9     0     2    PASS (2 skipped)                  ║
║  ✓ Transport         9     0     0    PASS                              ║
║                                                                          ║
║  Total: 58 pass  0 fail  2 skip   Duration: 12.4s                       ║
║  s:re-run  f:show-fails  i:detail  E:export  Tab:next-pane              ║
╚══════════════════════════════════════════════════════════════════════════╝
```

### Test Suite Results (With Failures)

```
╔══════════════════════════════════════════════════════════════════════════╗
║  CONFORMANCE — my-server  [✗ FAIL]  MCP 2025-11-25  │  55/60           ║
║  ──────────────────────────────────────────────────────────────────────  ║
║  SUITE               PASS  FAIL  SKIP  STATUS                           ║
║  ──────────────────  ────  ────  ────  ──────────────────────────────── ║
║  ✓ Lifecycle         4     0     0    PASS                              ║
║  ✓ Capability Neg.   6     0     0    PASS                              ║
║  ✗ Tools            10     2     0    FAIL                              ║  ← red
║  ✓ Resources         8     0     0    PASS                              ║
║  ✓ Prompts          10     0     0    PASS                              ║
║  ✗ Error Handling    7     1     1    FAIL                              ║
║  ✓ Transport         9     0     0    PASS                              ║
║                                                                          ║
║  Total: 55 pass  3 fail  1 skip  (3 failures require attention)         ║
║  s:re-run  f:show-fails  i:detail  E:export                             ║
╚══════════════════════════════════════════════════════════════════════════╝
```

### Test Detail View (Individual Test)

```
╔══════════════════════════════════════════════════════════════════════════╗
║  TEST DETAIL — tools/list.pagination                                     ║
║  ──────────────────────────────────────────────────────────────────────  ║
║  Suite:   Tools                                                          ║
║  Result:  ✗ FAIL                                                         ║
║                                                                          ║
║  Description:                                                            ║
║  Verifies that tools/list returns correct nextCursor when list           ║
║  exceeds one page and that pagination iterates to completion.            ║
║                                                                          ║
║  Assertion Failed:                                                       ║
║  Expected: response contains 'nextCursor' field when tools > page_size   ║
║  Actual:   response has no 'nextCursor' field (tools: 12, page_size: 10)  ║
║                                                                          ║
║  Evidence:                                                               ║
║  ┌────────────────────────────────────────────────────────────────────┐  ║
║  │ Method: tools/list                                                 │  ║
║  │ Response: { "tools": [...10 items], "result": {...} }              │  ║
║  │ Missing field: nextCursor                                          │  ║
║  └────────────────────────────────────────────────────────────────────┘  ║
║                                                                          ║
║  MCP Spec Reference: §6.1 — Tool List Pagination                         ║
║  [ Esc: Back ]                                                           ║
╚══════════════════════════════════════════════════════════════════════════╝
```

---

## Element Inventory

| ID | Widget | Type | Position | Description |
|----|--------|------|----------|-------------|
| ELM-001 | Panel | Panel | Full content area | Pane border |
| ELM-002 | Title | Text | Top border | "CONFORMANCE — <server>" + status badge |
| ELM-003 | Status badge | Badge | Title right | `[● RUNNING]` / `[✓ PASS]` / `[✗ FAIL]` |
| ELM-004 | Progress bar | ProgressBar | Row 2 | Fills during test run |
| ELM-005 | Progress text | Text | Row 2 right | "N% | N / N tests" |
| ELM-006 | Suite table | Table | Main area | Per-suite row |
| ELM-007 | Suite status glyph | Glyph | Col 1 | `✓`/`✗`/`◌`/`○` |
| ELM-008 | Suite name | Text | Col 2 | Test suite name |
| ELM-009 | Pass count | Text | Col 3 | Green when > 0 |
| ELM-010 | Fail count | Text | Col 4 | Red when > 0 |
| ELM-011 | Skip count | Text | Col 5 | Dim when > 0 |
| ELM-012 | Suite status | Text | Col 6 | PASS/FAIL/RUNNING.../PENDING |
| ELM-013 | Current test | Text | Below table | "Current: <test-name> (N/N) — Ns" |
| ELM-014 | Summary line | Text | Last data row | Total counts + duration |
| ELM-015 | Key hint bar | Text | Last row | Context shortcuts |
| ELM-016 | Detail view | FullPane | On `i` | Individual test detail |

---

## State Definitions

| State Variable | Type | Description |
|----------------|------|-------------|
| `run_status` | Enum: NotRun\|Running\|Complete\|Cancelled\|Error | Overall run state |
| `suites` | Vec<TestSuiteResult> | Per-suite results |
| `current_test` | Option<String> | Currently executing test |
| `total_tests` | usize | Total test count |
| `completed_tests` | usize | Completed (pass+fail+skip) |
| `selected_suite` | usize | Highlighted suite row |
| `detail_open` | bool | Test detail view visible |
| `detail_test_id` | Option<TestId> | Test in detail view |
| `filter_failures` | bool | Show failures only mode |
| `spec_version` | String | Target spec version |
| `start_time` | Option<Instant> | Run start time |

---

## Suite → Test Drill-Down

When `Enter` is pressed on a suite row with failures, the suite row expands to show individual tests:

```
║  ✗ Tools            10     2     0    FAIL                              ║
║    ▶ ✓ tools/list.basic                                  PASS          ║
║      ✓ tools/list.meta                                   PASS          ║
║      ✗ tools/list.pagination          FAIL — missing nextCursor        ║
║      ✓ tools/call.basic                                  PASS          ║
║      ✗ tools/call.error_format        FAIL — wrong error code          ║
```

Each test row shows: glyph + name + result + brief failure summary.

---

## Keyboard Shortcuts

| Key | Action | Notes |
|-----|--------|-------|
| `j` / `↓` | Next suite row | |
| `k` / `↑` | Previous suite row | |
| `Enter` | Expand suite / show test list | |
| `i` | Show detail for selected test | |
| `Esc` | Collapse suite / close detail | |
| `s` | Start / re-run test suite | |
| `f` | Toggle show-failures-only filter | |
| `E` | Export results | Opens format/output dialog |
| `Ctrl+C` | Cancel running suite | |
| `Tab` | Move focus to Traffic Inspector | |

---

## Export Dialog

```
┌ Export Conformance Report ─────────── ┐
│ Format: [junit▼]  (junit / json / text)│
│ Output: [● stdout  ○ file]             │
│ File:   [                            ] │
│ Spec version:  [2025-11-25           ] │
│                                        │
│  [ Export ]  [ Cancel ]                │
└──────────────────────────────────────── ┘
```

---

## Accessibility Notes

- **Status badges:** `✓ PASS` / `✗ FAIL` / `● RUNNING` / `○ PENDING` use glyphs + text
- **Suite results:** Color + glyph + text for pass/fail — no color-only
- **Progress:** Percentage text + fraction shown alongside progress bar
- **Failure detail:** Assertion failures shown as text (Expected/Actual)
- **Focus order:** Progress bar (status only) → Suite table → Key hint bar

---

## BC Traceability

| BC ID | How This Screen Satisfies It |
|-------|------------------------------|
| BC-8.19.001 | Capability negotiation test suite |
| BC-8.19.002 | Method coverage suite (tools, resources, prompts, errors) |
| BC-8.19.003 | Transport compliance suite |
| BC-8.20.001 | `E` key → export as JUnit XML |
| BC-8.20.002 | `E` key → export as JSON |
| BC-3.08.005 | Pass/fail uses glyphs + text + color |
