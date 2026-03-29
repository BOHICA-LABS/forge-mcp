---
document_type: ux-spec-screen
screen_id: SCR-005
screen_name: Health Metrics Panel
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-03-29T14:15:00
phase: 1c
complexity: complex
traces_to: UX-INDEX.md
prd_requirements: [BC-6.15.001, BC-6.13.001, BC-6.13.002, BC-6.14.001, BC-6.14.002, BC-3.08.002]
---

# Screen: Health Metrics Panel (SCR-005)

> Right panel of the main dashboard (or "Health" tab in narrow mode). Displays
> real-time health metrics for the active server: latency sparklines, error rate
> trend, throughput counter, and alert state indicators. Updates at 1Hz.

---

## Wireframe

### Normal State (All OK)

```
╔══════════════════════════════╗
║  HEALTH — my-server          ║
║  ────────────────────────── ║
║  Latency (p50/p95/p99)       ║
║  p50 ▁▂▂▃▂▁▂▃▂▃  42ms       ║
║  p95 ▂▃▄▄▃▂▃▄▃▄  89ms       ║
║  p99 ▃▄▅▆▅▄▅▅▄▅ 134ms       ║
║  ── threshold: 1000ms ──     ║
║                              ║
║  Error Rate (1m window)      ║
║  ▁▁▁▁▁▁▁▁▁▁▁▁▁▁  0.0%       ║
║  ── threshold:  5.0% ──      ║
║                              ║
║  Throughput (req/s)          ║
║  ▂▃▄▃▂▃▄▃▂▃▄▃▂▃  12/s       ║
║                              ║
║  ● All metrics OK            ║
║                              ║
║  [ t: thresholds  r: reset ] ║
╚══════════════════════════════╝
```

### Alert State (Latency Breach)

```
╔══════════════════════════════╗
║  HEALTH — my-server  ⚠ ALERT ║   ← alert badge in title
║  ────────────────────────── ║
║  Latency (p50/p95/p99)  ⚠   ║   ← metric section marker
║  p50 ▁▂▃▄▅▆▇▇▇▇ 1243ms ⚠   ║   ← red sparkline + value
║  p95 ▂▃▄▅▆▇█████ 2891ms ⚠   ║
║  p99 ▃▄▅▆▇██████ 4102ms ⚠   ║
║  ══ threshold: 1000ms ══     ║   ← threshold line highlighted
║                              ║
║  Error Rate (1m window)      ║
║  ▁▁▁▁▁▁▁▁▁▁▁▁▁▁  0.0%       ║
║  ── threshold:  5.0% ──      ║
║                              ║
║  Throughput (req/s)          ║
║  ▂▃▄▃▂▃▄▃▂▃▄▃▂▃  12/s       ║
║                              ║
║  ⚠ LATENCY ALERT             ║   ← alert summary line
║    p50 > 1000ms for 45s      ║
║                              ║
║  [ t: thresholds  r: reset ] ║
╚══════════════════════════════╝
```

### Expanded Histogram View (h key)

```
╔══════════════════════════════╗
║  LATENCY HISTOGRAM           ║
║  ────────────────────────── ║
║  Distribution (last 5 min)   ║
║                              ║
║     │▓                        ║
║  60 │▓▓                       ║
║     │▓▓▓▓                     ║
║  40 │▓▓▓▓▓▓                   ║
║     │▓▓▓▓▓▓▓▓░                ║
║  20 │▓▓▓▓▓▓▓▓░░░              ║
║     │▓▓▓▓▓▓▓▓░░░░░░░          ║
║   0 └────────────────────     ║
║     10 25 50 100 250 500 1000 ║
║     ← ms →                   ║
║                              ║
║  p50:  42ms  p95: 89ms       ║
║  p99: 134ms  max: 287ms      ║
║                              ║
║  [ Esc: back ]               ║
╚══════════════════════════════╝
```

### No Server Connected State

```
╔══════════════════════════════╗
║  HEALTH                      ║
║  ────────────────────────── ║
║                              ║
║  No server connected.        ║
║  Select a server to view     ║
║  health metrics.             ║
║                              ║
╚══════════════════════════════╝
```

---

## Element Inventory

| ID | Widget | Type | Position | Description |
|----|--------|------|----------|-------------|
| ELM-001 | Panel | Panel | Full pane | Pane border; alert state changes border color |
| ELM-002 | Title | Text | Top border | "HEALTH — <server>" + alert badge if alerting |
| ELM-003 | Latency section header | Text | Row 2 | "Latency (p50/p95/p99)" |
| ELM-004 | Latency alert marker | Glyph | Section header right | `⚠` when breached |
| ELM-005 | p50 sparkline | Sparkline | Row 3 | 3-char label + sparkline + current value |
| ELM-006 | p95 sparkline | Sparkline | Row 4 | " |
| ELM-007 | p99 sparkline | Sparkline | Row 5 | " |
| ELM-008 | Latency threshold line | Text | Row 6 | `── threshold: Nms ──` (double when alert) |
| ELM-009 | Error rate section header | Text | Row 8 | "Error Rate (1m window)" |
| ELM-010 | Error rate sparkline | Sparkline | Row 9 | Single sparkline + percentage |
| ELM-011 | Error threshold line | Text | Row 10 | `── threshold: N.N% ──` |
| ELM-012 | Throughput section header | Text | Row 12 | "Throughput (req/s)" |
| ELM-013 | Throughput sparkline | Sparkline | Row 13 | Single sparkline + req/s |
| ELM-014 | Alert summary | Text | Row 15+ | Alert state + description |
| ELM-015 | Status line | Text | Row N-2 | `● All metrics OK` or `⚠ ALERT` |
| ELM-016 | Key hint bar | Text | Last row | `t: thresholds  r: reset  h: histogram` |

---

## Sparkline Specification

Each sparkline:
- **Width:** fills pane width minus label (3 chars) and value (8 chars)
- **History:** last 60 seconds of data points (60 samples at 1Hz)
- **Characters:** `▁▂▃▄▅▆▇█` (8-level block elements) or braille `⣀⡄⡆⣆⣇⣧⣿`
- **Color:** green (`status.connected`) below threshold; amber (`status.warning`) approaching threshold (>80%); red (`status.error`) above threshold
- **ASCII fallback:** simple numeric bars `....::::####`

Alert threshold visualization:
- A threshold marker line is drawn below the sparkline
- Normal: `──` (dimmed) 
- Breached: `══` (bright/bold) — visual weight increase, not color alone

---

## Alert State Machine Display

Per BC-6.14.002:

| Alert State | Panel Border | Title Badge | Status Line | Sparkline Color |
|-------------|-------------|-------------|-------------|----------------|
| Normal | Normal (unfocused: light) | none | `● All metrics OK` (green) | Green |
| Approaching (>80% threshold) | Normal | none | `○ Approaching threshold` (amber) | Amber |
| Breached | Bold/bright | `⚠ ALERT` | `⚠ <METRIC> ALERT` (red) | Red |
| Recovered | Normal | none | `✓ Recovered (Ns ago)` (green) | Green for 30s |

**Non-color indicator for alerts:** Border character changes from `─` to `═` (double horizontal) when alert is active. Alert summary text always present.

---

## Metric Definitions

| Metric | Source | Update | Notes |
|--------|--------|--------|-------|
| Latency p50 | forge-health | 1Hz | 50th percentile of last 60s window |
| Latency p95 | forge-health | 1Hz | 95th percentile |
| Latency p99 | forge-health | 1Hz | 99th percentile |
| Error rate | forge-health | 1Hz | Error count / total count in 1m window |
| Throughput | forge-health | 1Hz | Requests per second (1m average) |

Displayed alongside sparklines:
- Current value (rightmost point of sparkline)
- Alert threshold (from configuration)

---

## State Definitions

| State Variable | Type | Description |
|----------------|------|-------------|
| `latency_history` | RingBuffer<LatencySample> | 60-point history for sparklines |
| `error_rate_history` | RingBuffer<f64> | 60-point error rate history |
| `throughput_history` | RingBuffer<f64> | 60-point throughput history |
| `alert_state` | AlertState | Normal/Approaching/Breached/Recovered per metric |
| `alert_triggered_at` | Option<Instant> | When current alert started |
| `thresholds` | ThresholdConfig | Latency/error/throughput thresholds |
| `histogram_view` | bool | Histogram expanded view active |
| `histogram_data` | Vec<(BucketMs, Count)> | Bucketed latency distribution |

---

## Keyboard Shortcuts

| Key | Action | Notes |
|-----|--------|-------|
| `j` / `↓` | Scroll view down | If content overflows |
| `k` / `↑` | Scroll view up | |
| `h` | Show latency histogram | Full-pane histogram view |
| `Esc` | Return from histogram to sparklines | |
| `t` | Open threshold editor | Inline form overlay |
| `r` | Reset metrics (clear history) | Confirm dialog |
| `Tab` | Move focus to Server Sidebar | |

---

## Threshold Editor Overlay

Accessed via `t`. Inline overlay within the health panel:

```
┌ Alert Thresholds ────────────────── ┐
│ Latency p50 (ms): [ 1000           ]│
│ Latency p95 (ms): [ 2000           ]│
│ Error Rate (%):   [  5.0           ]│
│ Throughput min:   [    0           ]│
│                                     │
│  [ Save ]  [ Cancel ]               │
└───────────────────────────────────── ┘
```

Tab order: fields → Save → Cancel. Enter on Save applies and closes. Escape cancels.

---

## Auto-Refresh Behavior

- Sparklines update every 1 second via timer tick event
- Alert state changes trigger immediate re-render (no wait for next tick)
- When pane is not focused, updates continue (background refresh)
- When health tab is hidden (narrow mode), metrics still collected; display updates when tab is activated

---

## Accessibility Notes

- **Alert:** Always paired with text (`⚠ ALERT` badge + summary text); not color-only
- **Sparklines:** Current value shown as text number alongside sparkline
- **Threshold:** Text label `threshold: Nms` always shown; border style changes (not color alone)
- **No animations:** Sparkline scroll is data-driven; no CSS-like animations
- **Focus order:** Section headers → sparklines (read-only) → key hint bar
- **Keyboard-only:** All actions accessible without mouse

---

## BC Traceability

| BC ID | How This Screen Satisfies It |
|-------|------------------------------|
| BC-6.15.001 | Time-series sparklines for latency, error rate, throughput |
| BC-6.13.001 | Passive metric display: latency p50/p95/p99, throughput |
| BC-6.13.002 | Error rate trend tracking and display |
| BC-6.14.001 | Threshold editor (`t` key) |
| BC-6.14.002 | Alert state machine display (Normal→Approaching→Breached→Recovered) |
| BC-3.08.002 | Sparkline and histogram visualization widgets |
| BC-3.08.005 | Alerts use glyph (`⚠`) + text badge + summary, not color-only |
