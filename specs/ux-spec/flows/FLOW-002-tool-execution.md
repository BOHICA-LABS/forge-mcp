---
document_type: ux-spec-flow
flow_id: FLOW-002
flow_name: Tool Discovery & Execution
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-03-29T14:15:00
phase: 1c
traces_to: UX-INDEX.md
screens: [SCR-001, SCR-002, SCR-003, SCR-006]
prd_requirements: [BC-2.05.001, BC-2.05.009, BC-2.05.010, BC-3.08.004]
---

# Flow: Tool Discovery & Execution (FLOW-002)

> Browse available tools for the connected server, select one, fill in
> arguments using the JSON schema-driven form, execute it, and view the result.
> Covers success, tool error, protocol error, and cancellation paths.

---

## Flow Diagram (ASCII)

```
  [Dashboard: server connected, SCR-003 focused on Tools tab]
       │
       │  User browses tools with j/k
       ▼
  SCR-003: Tool highlighted
       │
       │  User presses i or Enter (optional: view detail first)
       ▼
  SCR-003: Detail panel expands (schema, annotations, description)
       │
       │  User presses e (Execute)
       ▼
  SCR-006: Tool Execution Dialog opens
           Auto-focus on first required field
       │
       │  User fills in arguments
       │  Tab to advance between fields
       ▼
  SCR-006: All required fields filled, form valid
       │
       │  User presses Ctrl+Enter
       ▼
  SCR-006: Executing state
           Progress bar shown, elapsed timer ticking
       │
       ├──[User presses Esc]────────────────────────────────► Cancel path
       ├──[Server returns error]──────────────────────────► Error paths
       │
       │  Tool returns result
       ▼
  SCR-006: Result view shown (syntax highlighted content)
  SCR-004: New tools/call message pair appears in traffic inspector
       │
       │  User presses Esc or e (edit) or r (re-run)
       ▼
  [Return to SCR-003 capability browser]
```

---

## Step-by-Step Sequence

| Step | Screen | User Action | System Response |
|------|--------|-------------|----------------|
| 1 | SCR-003 | Ensure Tools tab is active (`1` key or `]`) | Tools tab shown with tool list |
| 2 | SCR-003 | `j`/`k` to navigate to desired tool | Tool row highlighted |
| 3 | SCR-003 | (Optional) `i` to view tool detail | Schema, description, annotations shown in detail panel |
| 4 | SCR-003 | `e` to execute | Tool Execution Dialog (SCR-006) opens |
| 5 | SCR-006 | Focus auto-placed on first required field | Dialog shows tool name, description, argument fields |
| 6 | SCR-006 | Fill in required arguments; `Tab` to advance | Validation runs on blur; hints shown for errors |
| 7 | SCR-006 | (Optional) fill optional arguments | Default values pre-filled |
| 8 | SCR-006 | `Ctrl+Enter` to execute | Dialog enters executing state; progress bar + timer |
| 9 | SCR-006 | (Wait — no action needed) | Request sent via tools/call; traffic captured |
| 10 | SCR-006 | (Automatic) Result received | Result view shown: ✓ OK badge + content |
| 11 | SCR-006 | Browse result with `j`/`k`; `y` to yank | Content scrollable; syntax-highlighted JSON if applicable |
| 12 | SCR-006 | `Esc` to close / `r` to re-run / `e` to edit | Returns to SCR-003 or re-opens form |

---

## Success Path

After step 10:
- Dialog title: "RESULT — <tool> @ <server>  ✓ OK │ NNms"
- Content blocks rendered (text/JSON/other MIME types)
- SCR-004 traffic inspector: new request+response pair visible
- `y` key: yanks result to clipboard
- `r` key: re-runs immediately with same args
- `e` key: returns to edit form with fields pre-filled

---

## Error Paths

### Validation Error (Cannot Execute)

| Trigger | Required field left empty, or type mismatch |
|---------|------|
| Step | Step 6–7 (before Ctrl+Enter) |
| Display | `⚠ Required field` / `⚠ Expected string` shown below field |
| Execute button | Dimmed and non-functional |
| Recovery | Fill required fields; validation clears on valid input |

### Tool Error (isError: true)

| Trigger | Tool runs but reports domain error |
|---------|------|
| Step | Step 9–10 |
| Display | Dialog title: "RESULT — <tool>  ✗ TOOL ERROR │ NNms" |
| Body | Error message text from content array |
| Recovery | `e` to edit args and retry; `Esc` to abandon |
| Per BC | BC-2.05.010: tool error distinguished from protocol error |

### Protocol Error (JSON-RPC error code)

| Trigger | Server rejects request at protocol level |
|---------|------|
| Display | Dialog title: "RESULT — <tool>  ✗ PROTOCOL ERROR │ NNms" |
| Body | `Code: -32602` + `Message: Invalid params…` |
| Recovery | `e` to edit args; check schema |
| Per BC | BC-2.05.010 |

### Cancellation

| Trigger | User presses `Esc` while executing |
|---------|------|
| Step | Step 8–9 |
| Display | `notifications/cancelled` sent to server |
| Result | "Cancelled after Nms" shown in result area |
| Recovery | `r` to re-run; `Esc` to close |
| Per BC | BC-2.05.009 |

### Server Disconnected During Execution

| Trigger | Connection drops after request sent |
|---------|------|
| Display | Dialog: "✗ CONNECTION LOST │ <Ns>" |
| Recovery | Return to sidebar (Esc); reconnect; retry |

---

## Screen Transitions

| From | To | Trigger |
|------|----|---------|
| SCR-003 (tools tab) | SCR-003 (detail expanded) | `i`/`Enter` key |
| SCR-003 (tools tab) | SCR-006 (arg entry) | `e` key |
| SCR-006 (arg entry) | SCR-006 (executing) | `Ctrl+Enter` |
| SCR-006 (executing) | SCR-006 (result) | Response received |
| SCR-006 (executing) | SCR-006 (result/error) | Error received |
| SCR-006 (result) | SCR-003 | `Esc` |
| SCR-006 (result) | SCR-006 (arg entry) | `e` (edit args) |
| SCR-006 (result) | SCR-006 (executing) | `r` (re-run) |

---

## Keyboard-Only Operation Path

1. `Tab` to focus capability browser (SCR-003)
2. `1` to ensure Tools tab is active
3. `j`/`k` to select tool
4. `i` (optional) to read detail, `Esc` to close
5. `e` to open execution dialog
6. Dialog auto-focuses first field — type value
7. `Tab` to advance to next field
8. `Ctrl+Enter` to execute
9. `j`/`k` to scroll result
10. `y` to yank, `Esc` to close
11. `Tab` returns to capability browser

No mouse required at any step.

---

## BC Traceability

| BC ID | Coverage |
|-------|---------|
| BC-2.05.001 | Tool list display (step 1) + invocation (step 8) |
| BC-2.05.009 | Cancellation via Esc during execution |
| BC-2.05.010 | Tool error vs. protocol error display |
| BC-3.08.004 | Capability explorer tab-based browsing |
| BC-3.07.001 | Keyboard-only operation throughout |
