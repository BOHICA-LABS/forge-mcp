---
document_type: ux-spec-flow
flow_id: FLOW-003
flow_name: Traffic Inspection
version: "1.1"
status: draft
producer: ux-designer
timestamp: 2026-03-29T14:57:00
phase: 1c
traces_to: UX-INDEX.md
screens: [SCR-001, SCR-004]
prd_requirements: [BC-4.09.001, BC-4.09.002, BC-4.10.001, BC-4.10.002, BC-4.10.003, BC-3.08.001]
changelog:
  - version: "1.1"
    date: 2026-03-29
    change: "ADV-P2-004 — Added explicit replay target-selection dialog and confirmation step to satisfy BC-4.10.003 PRE-002/PRE-004 (DI-007). Replay is no longer a direct one-keystroke action."
---

# Flow: Traffic Inspection (FLOW-003)

> Navigate to the traffic inspector, apply filters to isolate messages of
> interest, drill into a specific message for full JSON detail, view the
> correlated request/response pair, and optionally replay a request.

---

## Flow Diagram (ASCII)

```
  [Dashboard: server connected, traffic capturing]
       │
       │  User presses 2 (Traffic tab) or Tab to content area
       ▼
  SCR-004: Traffic Inspector active, messages scrolling in
           [● REC] badge visible, message count incrementing
       │
       │  User presses f (filter bar)
       ▼
  SCR-004: Filter bar opens above message table
       │
       │  User types method filter (e.g., "tools/") and presses Enter
       ▼
  SCR-004: Message list filters to matching messages
           "N of M messages" shown in title
       │
       │  User navigates j/k to desired message
       ▼
  SCR-004: Message row highlighted, correlated pair highlighted
       │
       │  User presses Enter or x (expand)
       ▼
  SCR-004: Full-pane JSON detail view
           Syntax-highlighted JSON, line numbers, timing shown
       │
       │  User presses p (pair view)
       ▼
  SCR-004: Split view — request left, response right
           RTT shown below
       │
       │  User presses R (replay)
       ▼
  SCR-004: REPLAY TARGET DIALOG opens (ELM-019)
           Title: "Replay Target"
           Dropdown lists all registered servers (ELM-020)
           Currently active server pre-selected BUT NOT confirmed
       │
       ├─ User selects a server from dropdown, presses Enter
       │
       ▼
  SCR-004: REPLAY CONFIRMATION DIALOG (ELM-021)
           Shows: method name, message ID, target server name/address
           Warning badge if target == original server (EC-007)
           [ Confirm ] focused    [ Cancel ]
       │
       ├─ User presses Enter / "y" (Confirm)
       │
       ▼
  SCR-004: Replay in progress — PROGRESS VIEW (ELM-022)
           "Replaying 1 message to <server>…  [0/1]"
       │
       ▼
  SCR-004: REPLAY RESULT — comparison row appears in message list
           Status: identical / equivalent / divergent / error
           New request/response pair visible in live stream
       │
       │  User presses Esc to return to list
       ▼
  [Return to SCR-004 message list]

  ── Cancel path ──────────────────────────────────────────────
  At REPLAY TARGET DIALOG or CONFIRMATION DIALOG:
       │  User presses Esc or selects [ Cancel ]
       ▼
  SCR-004: Dialog dismissed; returns to detail/list view unchanged
```

---

## Step-by-Step Sequence

| Step | Screen | User Action | System Response |
|------|--------|-------------|----------------|
| 1 | SCR-001 | `2` or `Tab` to Traffic tab | Traffic Inspector (SCR-004) becomes active |
| 2 | SCR-004 | Observe live message stream | Messages appear with timestamp, direction, method, status, ms |
| 3 | SCR-004 | `G` to jump to newest message (or `P` to pause) | Auto-tail mode (newest at bottom) |
| 4 | SCR-004 | `f` to open filter bar | Filter bar slides in above message table |
| 5 | SCR-004 | Type in Method field (e.g., `tools/`) + `Enter` | Message list filtered; count updates |
| 6 | SCR-004 | `j`/`k` to navigate filtered messages | Correlated pair highlighted when request selected |
| 7 | SCR-004 | `Enter` or `x` to expand message | Full JSON detail view opens |
| 8 | SCR-004 | `j`/`k` to scroll JSON | Syntax-highlighted JSON scrolls |
| 9 | SCR-004 | `p` to view request/response pair | Split view: request left, response right; RTT displayed |
| 10 | SCR-004 | `Tab` to focus right pane (response) | Scroll independently in response pane |
| 11 | SCR-004 | `R` to initiate replay | **Replay Target Dialog** opens (ELM-019); all registered servers listed; currently active server pre-highlighted but not auto-selected |
| 11a | SCR-004 | `j`/`k` to navigate server list, `Enter` to select target | Target server locked in; **Replay Confirmation Dialog** opens (ELM-021) showing method, message ID, and target server address |
| 11b | SCR-004 | `Enter` / `y` to confirm (or `Esc`/`n` to cancel) | If confirmed: replay executes; progress bar shows `[N/M]`; on completion, new request/response pair appears in message list with comparison status. If cancelled: dialogs dismiss, no replay. |
| 12 | SCR-004 | `y` to yank message | Message JSON copied to clipboard |
| 13 | SCR-004 | `Esc` to close detail/split, return to list | Back to message list |

---

## Success Path

After step 13:
- Filter remains active (persists between detail/list transitions)
- Replay (steps 11–11b) generates a new correlated pair in the live stream with a comparison status badge (`identical` / `equivalent` / `divergent`)
- `Esc` in detail → returns to list at same scroll position
- `Esc` from filter bar → clears filter, restores full list
- Replay dialogs dismissed by `Esc` or `n` leave the inspector state unchanged

---

## Error Paths

### No Messages Captured

| Trigger | Server connected but no traffic yet |
|---------|------|
| Display | Empty list: "No messages captured. Waiting for traffic…" |
| Recovery | Invoke tools via SCR-006 to generate traffic |

### Filter Returns Zero Results

| Trigger | Filter text matches no messages |
|---------|------|
| Display | "0 of N messages" shown; table empty; hint: "Press Esc to clear filter" |
| Recovery | `Esc` clears filter |

### Replay Fails — No Target Designated

| Trigger | User presses `R` but no servers registered (edge case) |
|---------|------|
| Display | Replay Target Dialog shows empty list with message: "No servers registered. Add a server to replay." |
| Recovery | Cancel dialog; configure a server in sidebar |

### Replay Fails — Target Unreachable

| Trigger | Confirmed target server unreachable when replay executes |
|---------|------|
| Display | Progress dismissed; error row appears: `✗ ERR` — "E-RPL-001: Target server unreachable: {address}" |
| Recovery | Verify server address; retry `R` |

### Replay Fails — Connection Lost Mid-Replay

| Trigger | Target server disconnects after some messages sent |
|---------|------|
| Display | Progress shows partial count; error: "E-RPL-002: Connection lost after {N} of {M} messages"; partial comparison report shown |
| Recovery | Reconnect; replay again with same or different target |

### Replay Cancelled by User

| Trigger | User presses `Esc` or `n` at either dialog |
|---------|------|
| Display | Dialogs close; inspector returns to prior state unchanged |
| Recovery | n/a (intentional cancel) |

### Buffer Full (WRAP)

| Trigger | Capture buffer reaches 100MB limit |
|---------|------|
| Display | `[WRAP]` badge appears in title bar; oldest messages evicted |
| Recovery | `c` to clear buffer; or configure higher limit in config |

---

## Screen Transitions

| From | To | Trigger |
|------|----|---------|
| SCR-001 (any tab) | SCR-004 (list view) | `2` key or Tab cycle |
| SCR-004 (list) | SCR-004 (filter bar active) | `f` key |
| SCR-004 (list) | SCR-004 (detail view) | `Enter`/`x` |
| SCR-004 (detail) | SCR-004 (split view) | `p` key |
| SCR-004 (detail/split) | SCR-004 (replay target dialog) | `R` key |
| SCR-004 (replay target dialog) | SCR-004 (replay confirmation dialog) | `Enter` (server selected) |
| SCR-004 (replay confirmation dialog) | SCR-004 (replay progress) | `Enter`/`y` (confirmed) |
| SCR-004 (replay progress) | SCR-004 (list with new row) | Replay completes |
| SCR-004 (any replay dialog) | SCR-004 (prior view) | `Esc`/`n` (cancel) |
| SCR-004 (detail/split) | SCR-004 (list) | `Esc` |

All transitions are within SCR-004; the outer dashboard shell (SCR-001) remains visible at all times.

---

## Keyboard-Only Operation Path

1. `2` to activate Traffic tab
2. `G` to jump to most recent message (or `g` for oldest)
3. `f` to open filter bar
4. Type filter text (e.g., `tools/`) in Method field
5. `Tab` within filter bar to move to other filter fields
6. `Enter` to apply filter, `Esc` to clear
7. `j`/`k` to navigate filtered messages
8. `Enter` to expand message detail
9. `p` to split request/response
10. `Tab` to switch focus between split panes
11. `R` to open Replay Target Dialog
12. `j`/`k` to navigate server list; `Enter` to select target
13. Confirm: `Enter` or `y` — Cancel: `Esc` or `n`
14. `y` to yank; `Esc` to return to list

No mouse required at any step.

---

## BC Traceability

| BC ID | Coverage |
|-------|---------|
| BC-4.09.001 | Messages shown for all captured JSON-RPC messages |
| BC-4.09.002 | MS column + RTT in detail view |
| BC-4.10.001 | Filter bar (method, direction, status, time range) |
| BC-4.10.002 | `/` search within visible messages |
| BC-4.10.003 | `R` key initiates replay; PRE-002 (explicit target designation) covered by Replay Target Dialog (ELM-019/ELM-020); PRE-004 (user confirmation) covered by Replay Confirmation Dialog (ELM-021); POST-003/POST-004 covered by comparison status badge; POST-005 covered by default non-original-server pre-selection; POST-006 covered by progress indicator (ELM-022); INV-001/DI-007 enforced — system will not replay without explicit target selection |
| BC-3.08.001 | Syntax-highlighted JSON in detail and split views |
