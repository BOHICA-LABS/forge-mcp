---
document_type: ux-spec-flow
flow_id: FLOW-003
flow_name: Traffic Inspection
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-03-29T14:15:00
phase: 1c
traces_to: UX-INDEX.md
screens: [SCR-001, SCR-004]
prd_requirements: [BC-4.09.001, BC-4.09.002, BC-4.10.001, BC-4.10.002, BC-4.10.003, BC-3.08.001]
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
  SCR-004: Message replayed to server
           New request/response pair appears in message list
       │
       │  User presses Esc to return to list
       ▼
  [Return to SCR-004 message list]
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
| 11 | SCR-004 | `R` to replay request | New request sent to server; appears as new message in list |
| 12 | SCR-004 | `y` to yank message | Message JSON copied to clipboard |
| 13 | SCR-004 | `Esc` to close detail/split, return to list | Back to message list |

---

## Success Path

After step 13:
- Filter remains active (persists between detail/list transitions)
- Replay (step 11) generates a new correlated pair at the top of the live stream
- `Esc` in detail → returns to list at same scroll position
- `Esc` from filter bar → clears filter, restores full list

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

### Replay Fails

| Trigger | Server disconnected before replay completes |
|---------|------|
| Display | New message row shows `✗ ERR` status; tooltip: "Replay failed: disconnected" |
| Recovery | Reconnect server in sidebar; retry `R` |

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
11. `R` to replay, `y` to yank
12. `Esc` to return to list

No mouse required at any step.

---

## BC Traceability

| BC ID | Coverage |
|-------|---------|
| BC-4.09.001 | Messages shown for all captured JSON-RPC messages |
| BC-4.09.002 | MS column + RTT in detail view |
| BC-4.10.001 | Filter bar (method, direction, status, time range) |
| BC-4.10.002 | `/` search within visible messages |
| BC-4.10.003 | `R` key replay of selected request |
| BC-3.08.001 | Syntax-highlighted JSON in detail and split views |
