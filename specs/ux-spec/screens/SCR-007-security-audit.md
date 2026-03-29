---
document_type: ux-spec-screen
screen_id: SCR-007
screen_name: Security Audit View
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-03-29T14:15:00
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

### Findings List (Audit Complete)

```
╔══════════════════════════════════════════════════════════════════════════╗
║  SECURITY AUDIT — my-server         [✗ 2 CRITICAL] [⚠ 5 HIGH] [● OK]  ║
║  ──────────────────────────────────────────────────────────────────────  ║
║  Filter: [all▼]   Sort: severity▼   Showing: 12 findings (2 suppressed) ║
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
║  s:start  f:filter  E:export  i:detail  u:suppress  ?:ast10-map         ║
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
| `findings` | Vec<SecurityFinding> | All findings |
| `filtered_findings` | Vec<FindingRef> | After severity filter applied |
| `selected_idx` | usize | Highlighted row |
| `severity_filter` | Enum: All\|Critical\|High\|Medium\|Low\|Info | Active filter |
| `sort_by` | Enum: Severity\|Confidence\|Category | Sort column |
| `sort_dir` | Enum: Asc\|Desc | Sort direction |
| `detail_open` | bool | Detail view visible |
| `detail_finding_id` | Option<FindingId> | Finding in detail view |
| `ast10_view` | bool | AST10 coverage map visible |
| `suppressed_count` | usize | Count of suppressed findings |
| `scan_progress` | Option<ScanProgress> | Current scan state |

---

## Severity Filter Options

| Option | Shows |
|--------|-------|
| `all` | All unsuppressed findings |
| `critical` | Critical only |
| `high+` | High and above |
| `medium+` | Medium and above |
| `suppressed` | Suppressed findings (review mode) |

---

## Export Report Flow

`E` key opens export dialog:

```
┌ Export Audit Report ───────────────── ┐
│ Format: [json▼]  (json / text / sarif) │
│ Output: [stdout     ]                   │
│         [● stdout  ○ file]              │
│ File:   [                             ] │  (when file selected)
│ Include OWASP AST10 summary: [✓]        │
│                                         │
│  [ Export ]  [ Cancel ]                 │
└─────────────────────────────────────── ┘
```

---

## Suppress Finding Flow

`u` key on a finding opens suppression dialog:

```
┌ Suppress Finding ──────────────────────────────────── ┐
│ Finding: execute_command shell execution (SEC-001)     │
│                                                        │
│ Reason: [                                           ]  │
│                                                        │
│ Scope:  [● this finding  ○ this category  ○ all]       │
│                                                        │
│  [ Suppress ]  [ Cancel ]                              │
└──────────────────────────────────────────────────────── ┘
```

Suppressed findings are hidden from the default view but counted in the footer.

---

## Keyboard Shortcuts

| Key | Action | Notes |
|-----|--------|-------|
| `j` / `↓` | Next finding | |
| `k` / `↑` | Previous finding | |
| `Enter` / `i` | Show finding detail | |
| `Esc` | Close detail / return to list | |
| `s` | Start new audit | Requires active server |
| `f` | Cycle severity filter | All → Critical → High+ → Medium+ |
| `S` | Toggle sort direction | |
| `u` | Suppress selected finding | Opens suppress dialog |
| `E` | Export audit report | Opens export dialog |
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
| BC-7.18.001 | `E` key triggers structured report export |
| BC-7.18.002 | `u` key triggers finding suppression |
| BC-7.18.003 | `?` key shows OWASP AST10 coverage map |
| BC-3.08.005 | Severity uses badge text + color; no color-only |
