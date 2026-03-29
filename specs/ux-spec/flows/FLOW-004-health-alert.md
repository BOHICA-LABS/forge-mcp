---
document_type: ux-spec-flow
flow_id: FLOW-004
flow_name: Health Alert
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-03-29T14:15:00
phase: 1c
traces_to: UX-INDEX.md
screens: [SCR-001, SCR-005]
prd_requirements: [BC-6.14.001, BC-6.14.002, BC-6.15.001, BC-6.13.001, BC-6.13.002]
---

# Flow: Health Alert (FLOW-004)

> A latency or error-rate threshold is breached → TUI shows alert indicators →
> user drills into health metrics to investigate → views histogram for depth →
> optionally adjusts thresholds.

---

## Flow Diagram (ASCII)

```
  [Dashboard: server connected, normal state]
       │
       │  (Background: latency rises above threshold)
       ▼
  SCR-005 Health panel border changes → double-line + ⚠ in title
  SCR-001 status bar: latency value turns amber/red
       │
       │  User notices alert indicator (any pane)
       ▼
  SCR-005 focused: ⚠ ALERT badge in title
                   Alert sparkline red
                   "⚠ LATENCY ALERT — p50 > 1000ms for 45s"
       │
       │  User presses 3 (Health tab) or Tab to Health pane
       ▼
  SCR-005: Full health metrics visible
           Red sparklines for breached metrics
           Alert summary at bottom of pane
       │
       │  User presses h (histogram)
       ▼
  SCR-005: Histogram view: latency distribution last 5 min
           p50, p95, p99 percentile markers
       │
       │  User presses Esc (back to sparklines)
       ▼
  SCR-005: Sparklines view
       │
       │  User presses t (thresholds)
       ▼
  SCR-005: Threshold editor overlay
           User adjusts values, presses Enter to save
       │
       │  (Background: latency recovers below threshold)
       ▼
  SCR-005: Alert state → Recovered
           Title: "✓ Recovered (Ns ago)"
           Sparklines return to green
  SCR-001 status bar: latency value returns to normal color
```

---

## Step-by-Step Sequence

| Step | Screen | User Action | System Response |
|------|--------|-------------|----------------|
| 1 | SCR-001 | (Passive — user may be on any screen) | Latency rises; forge-health computes breach |
| 2 | SCR-001 | (Automatic) | Status bar latency value changes color; `[Latency: 1243ms]` highlighted |
| 3 | SCR-005 | (Automatic) | Health panel border goes bold; `⚠ ALERT` badge appears in title |
| 4 | SCR-001 | User notices alert (status bar or health panel) | Decides to investigate |
| 5 | SCR-005 | `3` or `Tab` to focus Health pane | Full health pane visible; red sparklines; alert summary text |
| 6 | SCR-005 | User reads alert summary text | "⚠ LATENCY ALERT — p50 > 1000ms for 45s" |
| 7 | SCR-005 | `h` to open histogram | Latency histogram opens; percentile distribution shown |
| 8 | SCR-005 | `j`/`k` to scroll histogram (if needed) | Histogram adjusts view |
| 9 | SCR-005 | `Esc` to return to sparklines | Sparkline view restored |
| 10 | SCR-005 | (Optional) `t` to open threshold editor | Threshold overlay opens; current values pre-filled |
| 11 | SCR-005 | Adjust threshold values; `Tab` to next; `Enter` to save | Thresholds updated; alert re-evaluated |
| 12 | SCR-005 | (Automatic, when latency recovers) | Alert → Recovered; "✓ Recovered" shown |

---

## Success Path

After alert recovery:
- Sparklines return to green
- Alert summary replaced with "✓ Recovered (Ns ago)"
- Status bar latency value returns to normal color
- Health panel border returns to normal weight

---

## Error Paths

### Metrics Stop Updating (Connection Lost)

| Trigger | Server disconnects while collecting metrics |
|---------|------|
| Display | Health panel sparklines show gap (no new data points); last value faded |
| Display | Alert summary: "⚠ METRICS STALE — No data for Ns" |
| Recovery | Reconnect in sidebar (FLOW-001); metrics resume |

### Threshold Edit Invalid

| Trigger | User enters non-numeric value in threshold editor |
|---------|------|
| Display | `⚠ Expected numeric value` below field |
| Recovery | Correct value; Enter to save; Esc to cancel without saving |

### No Metrics Yet

| Trigger | Server just connected, no data points yet |
|---------|------|
| Display | Sparklines empty: "Collecting…"; percentile markers show `—` |
| Recovery | Wait a few seconds; sparklines populate |

---

## Screen Transitions

| From | To | Trigger |
|------|----|---------|
| Any pane | SCR-005 focused | `3` key or Tab cycle to Health pane |
| SCR-005 sparklines | SCR-005 histogram | `h` key |
| SCR-005 histogram | SCR-005 sparklines | `Esc` |
| SCR-005 sparklines | SCR-005 threshold editor | `t` key |
| SCR-005 threshold editor | SCR-005 sparklines | `Enter` (save) or `Esc` (cancel) |

---

## Alert Visibility From All Panes

The alert is designed to be visible even when the user is on another screen:

| Location | Alert Indicator |
|----------|----------------|
| Status bar (SCR-001) | `[Latency: NNNNms]` color changes (amber/red) |
| Health pane border | Border weight changes (light → double-line) |
| Health pane title | `⚠ ALERT` badge appears |
| (Future) Desktop notification | Out of scope |

All indicators use glyph + text + color per BC-3.08.005.

---

## Keyboard-Only Operation Path

1. No user action required to receive alert — visible passively
2. `3` to focus Health pane
3. Read alert summary text in pane
4. `h` to view histogram
5. `Esc` to return to sparklines
6. `t` to edit thresholds if desired
7. `Tab`/`Shift+F` through threshold fields
8. `Enter` to save

No mouse required.

---

## BC Traceability

| BC ID | Coverage |
|-------|---------|
| BC-6.14.001 | Threshold editor in step 10 |
| BC-6.14.002 | Alert state machine: Normal → Breached → Recovered |
| BC-6.15.001 | Time-series sparkline visualization |
| BC-6.13.001 | Passive latency and throughput metric collection |
| BC-6.13.002 | Error rate trend (same alert logic applies) |
| BC-3.08.002 | Sparkline + histogram visualization |
| BC-3.08.005 | Alert uses `⚠` glyph + text + color |
