---
document_type: ux-spec-flow
flow_id: FLOW-005
flow_name: Security Audit Workflow
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-03-29T14:15:00
phase: 1c
traces_to: UX-INDEX.md
screens: [SCR-001, SCR-002, SCR-004, SCR-007]
prd_requirements: [BC-7.16.001, BC-7.16.002, BC-7.16.004, BC-7.17.001, BC-7.18.001, BC-7.18.002, BC-7.18.003]
priority: P1
---

# Flow: Security Audit Workflow (FLOW-005)

> User connects to a server, navigates to the Security Audit tab, starts
> a runtime audit, monitors findings as they accumulate, drills into a
> critical finding for evidence, suppresses a false positive, and exports
> the audit report.

---

## Flow Diagram (ASCII)

```
  [Dashboard: server connected, traffic flowing]
       │
       │  User presses 4 (Audit tab)
       ▼
  SCR-007: "No audit run yet. Press s to start."
       │
       │  User presses s
       ▼
  SCR-007: Audit In Progress
           Progress bar + check list (✓ / ◌ / ○)
           Findings accumulate in real time
       │
       ├──[User presses Esc]──────────────────────────────► Cancelled; partial results shown
       │
       │  (All checks complete)
       ▼
  SCR-007: Findings list shown
           [✗ 2 CRITICAL] [⚠ 5 HIGH] badges in title
       │
       │  User navigates j/k to CRITICAL finding
       ▼
  SCR-007: Finding highlighted
       │
       │  User presses i or Enter
       ▼
  SCR-007: Finding detail view
           Severity, confidence, category, AST10 ref
           Evidence (message excerpt)
           Recommendation text
       │
       │  User decides finding is a false positive; presses u
       ▼
  SCR-007: Suppress dialog
           User enters reason, selects scope
           Presses Enter to confirm
       │
       ▼
  SCR-007: Finding suppressed; removed from default view
           "(N suppressed)" count in footer
       │
       │  User presses ? (AST10 map)
       ▼
  SCR-007: OWASP AST10 coverage view
           Per-category: ✓ Covered / ○ Not Applicable
           Coverage percentage shown
       │
       │  User presses Esc
       ▼
  SCR-007: Findings list
       │
       │  User presses E
       ▼
  SCR-007: Export dialog
           Select format (json/text/sarif), stdout/file
           Enter to export
       │
       ▼
  [Report exported; user returns to findings list]
```

---

## Step-by-Step Sequence

| Step | Screen | User Action | System Response |
|------|--------|-------------|----------------|
| 1 | SCR-001 | `4` key (or Tab cycle to Audit tab) | Security Audit view (SCR-007) active |
| 2 | SCR-007 | View "no audit run" state | Message: "Press s to start a security audit. Requires active traffic capture." |
| 3 | SCR-007 | `s` to start audit | Audit begins; progress bar shown; check list appears |
| 4 | SCR-007 | (Observe) | Checks tick off: ✓ Pattern analysis → ✓ SSRF detection → ◌ Permission check… |
| 5 | SCR-007 | (Observe) | Findings appear in list as detected; severity summary badges update live |
| 6 | SCR-007 | Audit completes | Full findings list shown; `[✗ N CRITICAL]` etc. in title |
| 7 | SCR-007 | `j`/`k` to navigate to critical finding | Critical row highlighted |
| 8 | SCR-007 | `i` or `Enter` to view detail | Finding detail: severity, confidence, category, evidence, recommendation |
| 9 | SCR-007 | `Esc` to return to list | List view restored |
| 10 | SCR-007 | Navigate to a false-positive finding | Row highlighted |
| 11 | SCR-007 | `u` to suppress | Suppress dialog opens |
| 12 | SCR-007 | Enter reason; select scope; `Enter` to confirm | Finding suppressed; removed from list; footer count updates |
| 13 | SCR-007 | `?` to view AST10 map | OWASP coverage map shown |
| 14 | SCR-007 | `Esc` to return | Findings list |
| 15 | SCR-007 | `E` to export | Export dialog opens |
| 16 | SCR-007 | Select format; `Enter` | Report written to stdout or file |

---

## Success Path

After step 16:
- Report exported to chosen output
- Findings list remains visible for continued investigation
- Suppressed findings are gone from default list; accessible via filter `suppressed`
- Status bar shows last audit timestamp

---

## Error Paths

### No Traffic Captured (Cannot Audit)

| Trigger | User presses `s` before any traffic captured |
|---------|------|
| Display | Warning dialog: "⚠ No traffic captured. Generate traffic first by connecting a server and invoking tools." |
| Recovery | Connect server; invoke tools via SCR-006; then `s` |

### Audit Cancelled (Partial Results)

| Trigger | User presses `Esc` during audit |
|---------|------|
| Display | `[CANCELLED]` badge in title; partial findings shown |
| Recovery | `s` to re-run from scratch; partial results still browseable |

### No Findings

| Trigger | Audit completes with zero findings |
|---------|------|
| Display | `[✓ CLEAN]` badge in title; "No findings detected." message |
| Body | AST10 coverage still shown with `?` |

### Export Fails

| Trigger | File path not writable |
|---------|------|
| Display | Export dialog: `⚠ Cannot write to path: Permission denied` |
| Recovery | Change output to stdout; or fix file path |

---

## Screen Transitions

| From | To | Trigger |
|------|----|---------|
| SCR-001 (any tab) | SCR-007 (not-run state) | `4` key |
| SCR-007 (not-run) | SCR-007 (scanning) | `s` key |
| SCR-007 (scanning) | SCR-007 (findings list) | Audit completes |
| SCR-007 (scanning) | SCR-007 (findings list, partial) | `Esc` cancel |
| SCR-007 (findings list) | SCR-007 (detail view) | `i`/`Enter` |
| SCR-007 (detail view) | SCR-007 (findings list) | `Esc` |
| SCR-007 (findings list) | SCR-007 (suppress dialog) | `u` |
| SCR-007 (findings list) | SCR-007 (AST10 map) | `?` |
| SCR-007 (AST10 map) | SCR-007 (findings list) | `Esc` |
| SCR-007 (findings list) | SCR-007 (export dialog) | `E` |

---

## Prerequisite: Traffic Capture

The security audit operates on captured traffic (from forge-traffic). The auditor analyzes:
- Tool schemas and names (for pattern detection)
- Actual arguments passed to tools (for SSRF, execution detection)
- Response content (for schema drift detection)
- Auth headers in HTTP transport (for auth validation)

User flow prerequisite: server must be connected and traffic must have been captured (even a few tool invocations are sufficient for pattern detection).

---

## Keyboard-Only Operation Path

1. `4` to activate Audit tab
2. `s` to start audit
3. Wait for completion
4. `j`/`k` to navigate findings
5. `i` to view detail
6. `Esc` to return to list
7. `u` to suppress a finding; fill form fields with Tab; `Enter` to confirm
8. `?` to view AST10 map; `Esc` to return
9. `E` to export; select format with arrow keys; `Enter`

No mouse required at any step.

---

## BC Traceability

| BC ID | Coverage |
|-------|---------|
| BC-7.16.001 | Dangerous pattern findings in list |
| BC-7.16.002 | SSRF findings displayed |
| BC-7.16.004 | Confidence score shown per finding |
| BC-7.17.001 | Permission escalation findings |
| BC-7.18.001 | `E` key exports structured report |
| BC-7.18.002 | `u` key suppresses findings |
| BC-7.18.003 | `?` key shows AST10 coverage map |
| BC-3.08.005 | All severity indicators use glyph + text |
