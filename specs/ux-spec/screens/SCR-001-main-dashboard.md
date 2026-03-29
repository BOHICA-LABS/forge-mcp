---
document_type: ux-spec-screen
screen_id: SCR-001
screen_name: Main Dashboard Layout
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-03-29T14:15:00
phase: 1c
complexity: complex
traces_to: UX-INDEX.md
prd_requirements: [BC-3.06.001, BC-3.06.002, BC-3.07.001, BC-3.08.003]
---

# Screen: Main Dashboard Layout (SCR-001)

> Root container screen. Hosts the multi-pane layout. All other panes
> (SCR-002 through SCR-005) render as children within this layout.
> This screen defines the structural skeleton; content is delegated to child screens.

---

## Wireframe

### Default Layout (120×40, truecolor)

```
╔══════════════════════════════════════════════════════════════════════════════════════════╗
║ ⚒ Forge MCP  │  my-server ● CONN  │  [ Capabilities ]  [ Traffic ]  [ Health ]  [ Audit ]║
╠════════════════╦═════════════════════════════════════════════════════╦═══════════════════╣
║  SERVERS      ║  ▶ CAPABILITIES — my-server                        ║  HEALTH           ║
║ ──────────────║  ─────────────────────────────────────────────────  ║  ─────────────── ║
║  ● my-server  ║  [ Tools │ Resources │ Prompts ]                   ║  Latency (p50)    ║
║  ○ staging    ║                                                     ║  ▁▂▃▄▃▂▄▅ 42ms   ║
║  ✗ old-api    ║  NAME              TYPE        DESCRIPTION          ║                   ║
║  ○ local-dev  ║  ──────────────── ─────────── ─────────────────── ║  Error Rate       ║
║               ║  read_file         tool        Read file contents   ║  ▁▁▁▁▁▁▁▁  0.0%  ║
║               ║  write_file        tool        Write to file        ║                   ║
║               ║  list_directory    tool        List directory       ║  Throughput       ║
║               ║  get_env           tool        Get env variable     ║  ▂▃▄▃▂▃▄▃ 12/s   ║
║               ║                                                     ║                   ║
║               ║                                                     ║  ● Alert OK       ║
║               ╠═════════════════════════════════════════════════════╣                   ║
║               ║  TRAFFIC INSPECTOR                                  ║                   ║
║               ║  ─────────────────────────────────────────────────  ║                   ║
║               ║  TIME     DIR  METHOD              STATUS   MS      ║                   ║
║               ║  14:02:31  ▶  tools/list           OK      23ms    ║                   ║
║               ║  14:02:32  ◀  tools/list (resp)    OK       -      ║                   ║
║               ║  14:02:45  ▶  tools/call           OK      67ms    ║                   ║
╠════════════════╩═════════════════════════════════════════════════════╩═══════════════════╣
║ [Daemon: OK] [Capture: ON | 47 msgs] [Latency: 42ms] [forge-mcp v0.1.0]  q:quit ?:help ║
╚══════════════════════════════════════════════════════════════════════════════════════════╝
```

### Minimum Layout (80×24)

```
╔══════════════════════════════════════════════════════════════════════════════╗
║ ⚒ Forge MCP  │  my-server ● CONN  │  [Caps] [Traffic] [Health]              ║
╠══════════════╦═════════════════════════════════════════════════════════════ ╣
║  SERVERS     ║  ▶ CAPABILITIES — my-server                                   ║
║  ──────────  ║  ─────────────────────────────────────────────────────────── ║
║  ● my-server ║  [ Tools │ Resources │ Prompts ]                              ║
║  ○ staging   ║                                                               ║
║  ✗ old-api   ║  NAME              TYPE        DESCRIPTION                    ║
║              ║  read_file         tool        Read file contents              ║
║              ║  write_file        tool        Write to file                   ║
║              ║  list_directory    tool        List directory                  ║
║              ╠═════════════════════════════════════════════════════════════ ╣
║              ║  TRAFFIC (3 msgs) — press t to expand                         ║
╠══════════════╩═════════════════════════════════════════════════════════════ ╣
║ [Daemon: OK] [Capture: ON] [42ms] [q:quit ?:help]                            ║
╚══════════════════════════════════════════════════════════════════════════════╝
```

### Terminal Too Small (< 80×24)

```
┌──────────────────────────────────────┐
│                                      │
│   ⚠ Terminal too small               │
│                                      │
│   Current:  72 × 18                  │
│   Required: 80 × 24 minimum          │
│                                      │
│   Please resize your terminal.       │
│                                      │
│   Press Ctrl+C to quit.              │
│                                      │
└──────────────────────────────────────┘
```

---

## Layout Specification

### Pane Proportions (character cells)

| Pane | Min Width | Default | Wide (>160) | Role |
|------|-----------|---------|-------------|------|
| Server Sidebar (SCR-002) | 18 cols | 22% | 30 cols fixed | Left panel |
| Content Area | 40 cols | 54% | remaining | Center panel (hosts SCR-003, SCR-004) |
| Health Panel (SCR-005) | — | 24% | 28 cols fixed | Right panel; collapses < 100 cols |
| Header bar | full width | 1 row | 1 row | Top chrome |
| Status bar | full width | 1 row | 1 row | Bottom chrome |

### Content Area Split

The content area is vertically split between Capability Browser (top) and Traffic Inspector (bottom):

| State | Top (Capabilities) | Bottom (Traffic) |
|-------|-------------------|-----------------|
| Default | 60% | 40% |
| Traffic focused | 30% | 70% |
| Capabilities focused | 70% | 30% |
| Traffic maximized | hidden | 100% |

### Health Panel Collapse (< 100 cols)

When terminal width < 100:
- Health panel disappears from right column
- Health content moves into a tab within the content area: `[ Capabilities │ Traffic │ Health ]`
- Status bar retains last latency value

### Resize Behavior

- `SIGWINCH` triggers immediate layout recalculation
- Minimum enforced: 80×24; below that, "too small" overlay renders
- Pane proportions preserved as percentages on resize
- Scroll positions preserved across resize events

---

## Element Inventory

| ID | Widget | Type | Position | Source Screen |
|----|--------|------|----------|--------------|
| ELM-001 | Header bar | Panel | Row 0, full width | This screen |
| ELM-002 | App title | Text | Header, left | Static label |
| ELM-003 | Active server badge | StatusBadge | Header, center-left | SCR-002 state |
| ELM-004 | Tab bar | TabBar | Header, center-right | This screen |
| ELM-005 | Server sidebar pane | Panel | Left column | SCR-002 |
| ELM-006 | Content area pane | Panel | Center column | SCR-003 / SCR-004 |
| ELM-007 | Health metrics pane | Panel | Right column | SCR-005 |
| ELM-008 | Status bar | Panel | Row last, full width | This screen |
| ELM-009 | Daemon status | Text | Status bar, left | Daemon health |
| ELM-010 | Capture status | Text | Status bar, center | Traffic capture state |
| ELM-011 | Latency indicator | Text | Status bar, center-right | Health metrics |
| ELM-012 | Version / keyhint | Text | Status bar, right | Static + context |

---

## State Definitions

| State Variable | Type | Description |
|----------------|------|-------------|
| `focused_pane` | Enum: Sidebar\|Content\|Health | Which pane has keyboard focus |
| `active_tab` | Enum: Capabilities\|Traffic\|Health\|Audit | Content area active tab |
| `active_server` | Option<ServerId> | Currently selected server |
| `daemon_status` | Enum: Ok\|Degraded\|Offline | Daemon connection state |
| `capture_active` | bool | Traffic capture running |
| `capture_count` | u64 | Messages captured |
| `terminal_size` | (u16, u16) | Current cols × rows |
| `health_collapsed` | bool | Health panel in tab mode |
| `resize_pending` | bool | SIGWINCH received, awaiting recalc |

---

## Header Bar

The header bar (1 row) contains:

```
 ⚒ Forge MCP  │  <server-name> <status-badge>  │  [Tab1] [Tab2] [Tab3] [Tab4]
```

| Element | Content | Update |
|---------|---------|--------|
| App logo | `⚒ Forge MCP` | Static |
| Separator | `│` | Static |
| Active server | `<name> <glyph> <badge>` | On server change |
| Tabs | `[ Capabilities ]  [ Traffic ]  [ Health ]  [ Audit ]` | On tab change |

Tab labels and number:
- `[ Capabilities ]` → SCR-003
- `[ Traffic ]` → SCR-004
- `[ Health ]` → SCR-005
- `[ Audit ]` → SCR-007 (P1)
- `[ Tests ]` → SCR-008 (P1)
- `[ Drift ]` → SCR-009 (P1)
- `[ Compare ]` → SCR-010 (P2)

Tabs 5–7 hidden until corresponding feature is active (P1/P2 gates).

---

## Status Bar

The status bar (1 row) contains persistent global state:

```
 [Daemon: OK]  [Capture: ON | 47 msgs]  [Latency: 42ms]  forge-mcp v0.1.0   q:quit ?:help
```

| Segment | Content | Alert condition |
|---------|---------|----------------|
| Daemon | `[Daemon: OK]` / `[Daemon: ERR]` | Red + `✗` on error |
| Capture | `[Capture: ON \| NNN msgs]` / `[Capture: OFF]` | — |
| Latency | `[Latency: NNms]` | Amber if > threshold |
| Version | `forge-mcp vX.Y.Z` | Static |
| Key hints | `q:quit ?:help r:refresh` | Context-sensitive |

---

## Keyboard Shortcuts (Dashboard-Level)

| Key | Action |
|-----|--------|
| `Tab` | Cycle focus: Sidebar → Content → Health → Sidebar |
| `Shift+Tab` | Reverse cycle |
| `h` / `l` | Focus left / right pane |
| `1` | Switch content tab to Capabilities |
| `2` | Switch content tab to Traffic |
| `3` | Switch content tab to Health |
| `4` | Switch content tab to Audit (P1) |
| `[` / `]` | Cycle content tabs left/right |
| `q` | Quit TUI (confirm dialog if capture active) |
| `?` | Help overlay |
| `Ctrl+L` | Force redraw |

---

## Accessibility Notes

- **Focus indicator:** Focused pane uses double-line border (`╔═╗`); unfocused uses light border (`┌─┐`)
- **No color-only:** Active server status uses glyph (`●`/`○`/`✗`) + text badge + color
- **Keyboard-only:** All panes reachable via `Tab` / `h` / `l` without mouse
- **Terminal too small:** Graceful degradation renders a text-only error message — no partial/corrupt UI
- **Status bar:** Always visible, providing global orientation context
- **Focus order:** Header → Server Sidebar → Content Area → Health Panel → Status Bar (Tab cycle)

---

## BC Traceability

| BC ID | How This Screen Satisfies It |
|-------|------------------------------|
| BC-3.06.001 | Multi-pane layout adapts from 80×24 to ultra-wide; proportional columns |
| BC-3.06.002 | Color detection at startup; tokens applied globally |
| BC-3.07.001 | hjkl navigation, Tab pane cycling |
| BC-3.07.003 | Mouse click selects pane focus (supplementary) |
| BC-3.08.003 | Server status badge with glyph + text + color in header |
| BC-3.08.005 | All status indicators use glyph + color |
