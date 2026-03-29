---
document_type: ux-spec-screen
screen_id: SCR-007
screen_name: Security Audit View
version: "1.1"
status: draft
producer: ux-designer
timestamp: 2026-03-29T14:15:00
revised: 2026-03-29
revision_reason: ADV-P1-003 — align suppression UX with BC-7.18.002 overlay semantics
phase: 1c
complexity: complex
traces_to: UX-INDEX.md
prd_requirements: [BC-7.16.001, BC-7.16.002, BC-7.16.003, BC-7.16.004, BC-7.17.001, BC-7.17.002, BC-7.17.003, BC-7.18.001, BC-7.18.002, BC-7.18.003]
priority: P1
---

# Screen: Security Audit View (SCR-007)

> Full content-area tab showing runtime security audit findings for the active
> server. Displays findings with severity indicators, confidence scores, and
> OWASP AST10 mapping. Supports detail expansion, severity filtering, and
> compliance report export. Available as `[ Audit ]` tab in main dashboard.

---

## Wireframe

### Findings List (Audit Complete — Default View: Active Findings)

```
╔══════════════════════════════════════════════════════════════════════════╗
║  SECURITY AUDIT — my-server         [✗ 2 CRITICAL] [⚠ 5 HIGH] [● OK]  ║
║  ──────────────────────────────────────────────────────────────────────  ║
║  Filter: [active▼]  Sort: severity▼   10 active  [2 suppressed — V:show]║
║  ──────────────────────────────────────────────────────────────────────  ║
║  SEV    CONF  CATEGORY          TITLE                             AST10  ║
║  ─────  ────  ───────────────── ─────────────────────────────── ──────  ║
║  [CRIT] 0.92  Dangerous Pattern  execute_command: shell exec      AST02  ║
║  [CRIT] 0.87  SSRF Attempt       http_get to 169.254.169.254      AST04  ║
║  [HIGH] 0.81  Permission Escal.  Tool claimed unrestricted path   AST03  ║
║  [HIGH] 0.79  Auth Validation    Missing auth header validation   AST07  ║
║  [HIGH] 0.71  Dangerous Pattern  write_file: unrestricted write   AST02  ║
║  [MED]  0.65  Schema Drift       read_file schema changed (hash)  AST09  ║
║  [MED]  0.61  Root Enforcement   Root boundary not enforced       AST06  ║
║  [LOW]  0.55  Dangerous Pattern  get_env: exposes env variables   AST02  ║
║  [INFO] 0.51  Coverage           8 tools analyzed, 2 untested     —      ║
║                                                                          ║
║  ──────────────────────────────────────────────────────────────────────  ║
║  s:start  f:filter  V:suppressed  E:export  i:detail  u:suppress  ?:ast10║
╚══════════════════════════════════════════════════════════════════════════╝
```

### Findings List (Suppressed Filter Active)

> Pressing `V` toggles the filter to show only suppressed findings.
> Suppressed findings are NEVER deleted — they remain in the audit data
> with `suppressed: true` per BC-7.18.002 (DI-012 overlay invariant).

```
╔══════════════════════════════════════════════════════════════════════════╗
║  SECURITY AUDIT — my-server         [✗ 2 CRITICAL] [⚠ 5 HIGH]          ║
║  ──────────────────────────────────────────────────────────────────────  ║
║  Filter: [suppressed▼]  Sort: severity▼   Showing: 2 suppressed         ║
║  ──────────────────────────────────────────────────────────────────────  ║
║  SEV    CONF  CATEGORY          TITLE                   REASON    AST10  ║
║  ─────  ────  ───────────────── ──────────────────────  ────────  ─────  ║
║  [SUPP] [HIGH] 0.75  Auth Val.  Missing auth (known)   FP:cfg    AST07  ║
║  [SUPP] [MED]  0.60  Sch Drift  resource_list drift    Accepted  AST09  ║
║                                                                          ║
║  Suppressed findings are preserved in reports (suppressed: true).        ║
║  ──────────────────────────────────────────────────────────────────────  ║
║  V:back-to-active  i:detail  u:unsuppress  E:export(incl. suppressed)   ║
╚══════════════════════════════════════════════════════════════════════════╝
```

### Finding Detail Expansion

```
╔══════════════════════════════════════════════════════════════════════════╗
║  FINDING DETAIL — SEC-001                                                ║
║  ──────────────────────────────────────────────────────────────────────  ║
║  [CRIT] execute_command: shell execution capability detected             ║
║                                                                          ║
║  Severity:    Critical                 Confidence: 0.92 (high)          ║
║  Category:    Dangerous Tool Pattern   AST10 Ref: AST02 (Indirect       ║
║  Finding ID:  SEC-001                            Prompt Injection)       ║
║                                                                          ║
║  Description:                                                            ║
║  The tool 'execute_command' exposes arbitrary shell command execution.   ║
║  This pattern enables prompt injection attacks where an attacker can     ║
║  cause the AI agent to execute arbitrary system commands via crafted     ║
║  tool arguments.                                                         ║
║                                                                          ║
║  Evidence (from traffic capture):                                        ║
║  ┌────────────────────────────────────────────────────────────────────┐  ║
║  │ Message ID: 003  Method: tools/call  Time: 14:02:45               │  ║
║  │ "name": "execute_command"                                          │  ║
║  │ "arguments": { "command": "ls -la /etc" }                         │  ║
║  └────────────────────────────────────────────────────────────────────┘  ║
║                                                                          ║
║  Recommendation:                                                         ║
║  Restrict execute_command to an allowlist of safe commands. Validate    ║
║  all arguments against an allowlist. Consider removing shell execution  ║
║  capability entirely unless strictly required.                           ║
║                                                                          ║
║  [ u: Suppress ]  [ Esc: Back ]                                          ║
╚══════════════════════════════════════════════════════════════════════════╝
```

### Audit In Progress

```
╔══════════════════════════════════════════════════════════════════════════╗
║  SECURITY AUDIT — my-server         [● Scanning…]                       ║
║  ──────────────────────────────────────────────────────────────────────  ║
║  ████████████████████████░░░░░░░░░░░░░░  Analyzing traffic…  62%       ║
║                                                                          ║
║  Checks in progress:                                                     ║
║  ✓ Tool pattern analysis (12 tools)                                      ║
║  ✓ SSRF detection                                                        ║
║  ◌ Permission escalation check                                           ║
║  ○ Root enforcement validation                                           ║
║  ○ Auth handling validation                                              ║
║  ○ Schema drift detection                                                ║
║                                                                          ║
║  Findings so far: [CRIT] 2  [HIGH] 3  …                                 ║
║                                                                          ║
║  [ Esc: Cancel ]                                                         ║
╚══════════════════════════════════════════════════════════════════════════╝
```

### No Audit Run State

```
╔══════════════════════════════════════════════════════════════════════════╗
║  SECURITY AUDIT — my-server                                              ║
║  ──────────────────────────────────────────────────────────────────────  ║
║                                                                          ║
║                  No audit has been run yet.                              ║
║                                                                          ║
║                  Press s to start a security audit.                      ║
║                  Requires active traffic capture.                        ║
║                                                                          ║
║  s:start audit                                                           ║
╚══════════════════════════════════════════════════════════════════════════╝
```

### OWASP AST10 Coverage Map

```
╔══════════════════════════════════════════════════════════════════════════╗
║  OWASP AST10 COVERAGE — my-server audit                                  ║
║  ──────────────────────────────────────────────────────────────────────  ║
║  AST01  Prompt Injection               ✓ Covered (0 findings)            ║
║  AST02  Indirect Prompt Injection      ✓ Covered (2 findings)            ║
║  AST03  Over-Privileged Skills         ✓ Covered (1 finding)             ║
║  AST04  SSRF                           ✓ Covered (1 finding)             ║
║  AST05  Data Exfiltration              ✓ Covered (0 findings)            ║
║  AST06  Weak Isolation                 ✓ Covered (1 finding)             ║
║  AST07  Insufficient Auth              ✓ Covered (1 finding)             ║
║  AST08  Resource Exhaustion            ○ Not Applicable                  ║
║  AST09  Supply Chain                   ✓ Covered (1 finding)             ║
║  AST10  Sensitive Data Exposure        ✓ Covered (0 findings)            ║
║                                                                          ║
║  Coverage: 83% (9/10 — 1 N/A excluded)                                  ║
║  [ Esc: Back ]                                                           ║
╚══════════════════════════════════════════════════════════════════════════╝
```

---

## Element Inventory

| ID | Widget | Type | Position | Description |
|----|--------|------|----------|-------------|
| ELM-001 | Panel | Panel | Full content area | Pane border |
| ELM-002 | Title | Text | Top border | "SECURITY AUDIT — <server>" |
| ELM-003 | Severity summary | Badges | Title right | `[✗ N CRITICAL] [⚠ N HIGH] [● OK]` |
| ELM-004 | Filter bar | Dropdown + text | Row 2 | Severity filter + sort + count |
| ELM-005 | Column headers | Text | Row 4 | SEV, CONF, CATEGORY, TITLE, AST10 |
| ELM-006 | Finding rows | Table | Data area | One row per finding |
| ELM-007 | Severity badge | Badge | Col 1 | `[CRIT]`/`[HIGH]`/`[MED]`/`[LOW]`/`[INFO]` |
| ELM-008 | Confidence score | Text | Col 2 | Decimal 0.0–1.0 |
| ELM-009 | Category | Text | Col 3 | Finding category |
| ELM-010 | Title | Text | Col 4 | Truncated finding title |
| ELM-011 | AST10 ref | Text | Col 5 | `ASTxx` or `—` |
| ELM-012 | Detail view | FullPane | Overlay | Finding detail on `i` |
| ELM-013 | Progress bar | ProgressBar | Scanning state | Percentage + current check |
| ELM-014 | Check list | List | Scanning state | Per-check progress (✓/◌/○) |
| ELM-015 | AST10 map | FullPane | On `?` | OWASP coverage map |
| ELM-016 | Key hint bar | Text | Last row | Context shortcuts |
| ELM-017 | View mode indicator | Text | Filter row right | "N active [M suppressed — V:show]" or "M suppressed [V:back]" |
| ELM-018 | Suppressed badge | Badge | Finding row col 1 (suppressed view) | `[SUPP]` prefix before original severity badge |

---

## Column Widths

| Column | Width | Notes |
|--------|-------|-------|
| SEV | 7 cols | `[CRIT]` etc. |
| CONF | 5 cols | `0.92 ` |
| CATEGORY | 17 cols | Truncated |
| TITLE | remaining | Truncated with `…` |
| AST10 | 6 cols | `AST02` or `—` |

---

## State Definitions

| State Variable | Type | Description |
|----------------|------|-------------|
| `audit_status` | Enum: NotRun\|InProgress\|Complete\|Error | Current audit state |
| `findings` | Vec<SecurityFinding> | **All** findings — active and suppressed (overlay model per DI-012) |
| `filtered_findings` | Vec<FindingRef> | References into `findings` after view_mode + severity filter |
| `view_mode` | Enum: Active\|Suppressed | Whether showing active or suppressed findings; toggled by `V` |
| `selected_idx` | usize | Highlighted row |
| `severity_filter` | Enum: All\|Critical\|High\|Medium\|Low\|Info | Active severity filter (applies within view_mode) |
| `sort_by` | Enum: Severity\|Confidence\|Category | Sort column |
| `sort_dir` | Enum: Asc\|Desc | Sort direction |
| `detail_open` | bool | Detail view visible |
| `detail_finding_id` | Option<FindingId> | Finding in detail view |
| `ast10_view` | bool | AST10 coverage map visible |
| `active_count` | usize | Count of unsuppressed findings |
| `suppressed_count` | usize | Count of suppressed findings (shown in footer always) |
| `scan_progress` | Option<ScanProgress> | Current scan state |

---

## Severity Filter Options

> The filter dropdown (`f` key) controls severity-based filtering within the
> current view mode. The `V` key separately toggles between **active** and
> **suppressed** view modes. These two axes are independent.

| Option | Shows | View mode |
|--------|-------|-----------|
| `active` | All unsuppressed findings (default) | Active mode |
| `critical` | Critical unsuppressed only | Active mode |
| `high+` | High and above, unsuppressed | Active mode |
| `medium+` | Medium and above, unsuppressed | Active mode |
| `suppressed` | All suppressed findings | Suppressed mode (same as `V` toggle) |

> **V key:** Shortcut to jump directly to suppressed view mode, equivalent to
> selecting `suppressed` from the filter dropdown. `V` again (or `Esc` from
> suppressed view) returns to active view.

---

## Export Report Flow

`E` key opens export dialog. Exports **always include suppressed findings**
(with `suppressed: true` field set) per BC-7.18.002 overlay contract.
The summary section shows `suppressed_count` regardless of current view mode.

```
┌ Export Audit Report ───────────────────── ┐
│ Format: [json▼]  (json / text / sarif)    │
│ Output: [● stdout  ○ file]                 │
│ File:   [                              ]   │  (when file selected)
│ Include OWASP AST10 summary: [✓]           │
│                                            │
│ ℹ  Suppressed findings are always         │
│    included with suppressed: true set.     │
│    Use --no-suppress flag to export        │
│    without applying suppression rules.     │
│                                            │
│  [ Export ]  [ Cancel ]                    │
└──────────────────────────────────────────── ┘
```

---

## Suppress Finding Flow

> **Suppression semantics (BC-7.18.002 / DI-012):** Suppression is an overlay.
> The finding is marked `suppressed: true` in the audit data and report.
> The original finding content (severity, confidence, evidence) is UNCHANGED.
> Suppressed findings are removed from the **default active view** but remain
> **fully inspectable** via the `V` (suppressed filter) toggle and are
> **always included in exports** (with `suppressed: true` field set).

`u` key on a finding opens suppression dialog:

```
┌ Suppress Finding ──────────────────────────────────── ┐
│ Finding: execute_command shell execution (SEC-001)     │
│ Severity: CRITICAL   Confidence: 0.92                  │
│                                                        │
│ Reason: [                                           ]  │
│  (required — recorded in audit trail)                  │
│                                                        │
│ Scope:  [● this finding  ○ this category  ○ server]    │
│                                                        │
│ Note: Finding stays in report as suppressed: true.     │
│ Press V to review suppressed findings later.           │
│                                                        │
│  [ Suppress ]  [ Cancel ]                              │
└──────────────────────────────────────────────────────── ┘
```

After suppression:
- Finding moves from `active` view to `suppressed` view
- Footer updates: "10 active  [3 suppressed — V:show]"
- Finding is **NOT deleted** — it remains in audit data and all exports
- `V` toggle reveals suppressed findings for review at any time

`u` on a suppressed finding (in suppressed view) **unsuppresses** it, returning it to the active list.

### Suppression and Export Interaction

Exports always include suppressed findings. The export dialog notes this explicitly:

```
┌ Export Audit Report ───────────────────── ┐
│ Format: [json▼]  (json / text / sarif)    │
│ Output: [● stdout  ○ file]                 │
│ File:   [                              ]   │
│ Include OWASP AST10 summary: [✓]           │
│                                            │
│ ℹ  Suppressed findings are always         │
│    included with suppressed: true set.     │
│    Use --no-suppress to audit without      │
│    applying suppression rules.             │
│                                            │
│  [ Export ]  [ Cancel ]                    │
└──────────────────────────────────────────── ┘
```

---

## Keyboard Shortcuts

| Key | Action | Notes |
|-----|--------|-------|
| `j` / `↓` | Next finding | |
| `k` / `↑` | Previous finding | |
| `Enter` / `i` | Show finding detail | |
| `Esc` | Close detail / return to list | |
| `s` | Start new audit | Requires active server |
| `f` | Cycle severity filter | Active → Critical → High+ → Medium+ |
| `S` | Toggle sort direction | |
| `u` | Suppress selected finding | Opens suppress dialog; finding preserved in data |
| `V` | Toggle suppressed findings view | Shows findings with `suppressed: true`; `V` again returns to active |
| `E` | Export audit report | Opens export dialog; always includes suppressed findings |
| `?` | Show OWASP AST10 coverage map | |
| `Tab` | Move focus to Capability Browser | |

---

## Accessibility Notes

- **Severity:** Always uses `[CRIT]`/`[HIGH]`/`[MED]`/`[LOW]`/`[INFO]` text badges; not color-only
- **Summary badges:** Title bar shows `[✗ N CRITICAL]` with glyph + count + text
- **Confidence score:** Decimal value always shown; "high/medium/low" label in detail view
- **AST10 map:** Coverage shown with `✓`/`○` glyphs + text; not color-only
- **Finding count:** "Showing N findings (M suppressed)" always in filter bar
- **Focus order:** Filter bar → Finding table → Key hint bar

---

## BC Traceability

| BC ID | How This Screen Satisfies It |
|-------|------------------------------|
| BC-7.16.001 | Findings list shows dangerous tool patterns |
| BC-7.16.002 | SSRF findings displayed with category + AST10 ref |
| BC-7.16.003 | Schema drift findings shown in list |
| BC-7.16.004 | Confidence score shown per finding |
| BC-7.17.001 | Permission escalation findings |
| BC-7.17.002 | Root enforcement findings |
| BC-7.17.003 | Auth validation findings |
| BC-7.18.001 | `E` key triggers structured report export; export always includes suppressed findings |
| BC-7.18.002 | `u` key triggers finding suppression (overlay — finding preserved as suppressed:true); `V` key inspects suppressed findings; export includes suppressed_count in summary |
| BC-7.18.003 | `?` key shows OWASP AST10 coverage map |
| BC-3.08.005 | Severity uses badge text + color; no color-only |
