---
document_type: ux-spec-index
level: L3
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-03-29T14:15:00
phase: 1c
inputs: [prd.md, behavioral-contracts/BC-INDEX.md, prd-supplements/interface-definitions.md, architecture/module-decomposition.md, planning/product-brief.md]
traces_to: prd.md
prd_version: "1.0"
design_system_version: "1.0"
---

# UX Specification: Forge MCP TUI

> **Sharded artifact (DF-021).** This is a terminal-native TUI application built
> with ratatui + crossterm. There is NO web UI or GUI. All wireframes use ASCII/
> Unicode box drawing. All layouts use the character-cell grid. All interactions
> are keyboard-first with optional mouse supplementary input.
>
> This index contains global TUI settings (design system, color system, keybinding
> scheme, terminal requirements, a11y). Per-screen and per-flow details live in
> `screens/` and `flows/`.

---

## Screen Inventory

| SCR ID | Name | Purpose | Priority | File |
|--------|------|---------|----------|------|
| SCR-001 | Main Dashboard Layout | Root multi-pane container; adapts from 80×24 to ultra-wide | P0 | screens/SCR-001-main-dashboard.md |
| SCR-002 | Server Sidebar | Server list with connection status; keyboard nav; search | P0 | screens/SCR-002-server-sidebar.md |
| SCR-003 | Capability Browser | Tab-based Tools/Resources/Prompts explorer with tables | P0 | screens/SCR-003-capability-browser.md |
| SCR-004 | Traffic Inspector | Scrollable JSON-RPC message log; filter; syntax highlight | P0 | screens/SCR-004-traffic-inspector.md |
| SCR-005 | Health Metrics Panel | Sparklines, histograms, alert threshold indicators | P0 | screens/SCR-005-health-metrics.md |
| SCR-006 | Tool Execution Dialog | Modal: JSON-schema-driven args, output, error display | P0 | screens/SCR-006-tool-execution.md |
| SCR-007 | Security Audit View | Findings list, severity indicators, compliance export | P1 | screens/SCR-007-security-audit.md |
| SCR-008 | Conformance Test Runner | Test suite progress, pass/fail, detail view | P1 | screens/SCR-008-conformance-runner.md |
| SCR-009 | Config Drift View | Side-by-side config diff with highlighted changes | P1 | screens/SCR-009-config-drift.md |
| SCR-010 | Server Comparison View | Side-by-side capability diff between two servers | P2 | screens/SCR-010-server-comparison.md |

---

## Flow Inventory

| FLOW ID | Name | Screens Involved | Steps | File |
|---------|------|-----------------|-------|------|
| FLOW-001 | Server Connection | SCR-001, SCR-002 | 6 | flows/FLOW-001-server-connection.md |
| FLOW-002 | Tool Discovery & Execution | SCR-002, SCR-003, SCR-006 | 8 | flows/FLOW-002-tool-execution.md |
| FLOW-003 | Traffic Inspection | SCR-001, SCR-004 | 7 | flows/FLOW-003-traffic-inspection.md |
| FLOW-004 | Health Alert | SCR-001, SCR-005 | 5 | flows/FLOW-004-health-alert.md |
| FLOW-005 | Security Audit Workflow | SCR-002, SCR-007 | 7 | flows/FLOW-005-security-audit.md |

---

## Cross-References

| If you need... | Read these together |
|----------------|-------------------|
| Implement a specific screen | UX-INDEX.md (globals) + screens/SCR-NNN-*.md |
| Write E2E tests for a flow | flows/FLOW-NNN-*.md + referenced screen files |
| Accessibility audit | UX-INDEX.md (a11y checklist) + screens/SCR-NNN-*.md |
| Design system reference | UX-INDEX.md §Design System + design-system/ directory |
| Full UX review | UX-INDEX.md + all screen and flow files |

---

## Terminal Requirements

| Requirement | Value | Notes |
|-------------|-------|-------|
| Minimum terminal size | 80×24 | Below this: show "Terminal too small" message |
| Recommended size | 120×40 | Enables full multi-pane layout |
| Ultra-wide support | Up to 220×60 | Panes expand proportionally |
| Color depth minimum | 16-color ANSI | All indicators work at 16-color |
| Color depth preferred | Truecolor (24-bit) | Full syntax highlighting palette |
| Box drawing | Unicode preferred, ASCII fallback | Auto-detected via `$TERM` / `$COLORTERM` |
| Encoding | UTF-8 required | |
| Mouse | Optional supplementary | Disabled via `--mouse false` |

---

## TUI Design System

### Color System

#### Auto-Detection Strategy

```
Detection order:
1. $COLORTERM == "truecolor" | "24bit"  → Truecolor mode
2. $TERM contains "256color"             → 256-color mode  
3. $TERM == "xterm" | "screen" | etc.   → 16-color mode
4. $NO_COLOR set                         → Monochrome fallback
5. --color never CLI flag               → Monochrome fallback
```

#### Truecolor Palette (Primary)

> Designed for dark terminal backgrounds (default). Light-background variants
> use `color.light.*` token aliases — same structure, inverted luminance.

**Semantic Color Tokens:**

| Token | Dark Mode (hex) | Light Mode (hex) | Usage |
|-------|----------------|-----------------|-------|
| `color.bg.primary` | `#1a1b26` | `#f0f0f0` | Main background |
| `color.bg.secondary` | `#24283b` | `#e0e0e0` | Pane backgrounds |
| `color.bg.selected` | `#364a82` | `#aaccff` | Selected row/item highlight |
| `color.bg.focused` | `#2d3561` | `#bbd4ff` | Focused pane border |
| `color.fg.primary` | `#c0caf5` | `#1a1b26` | Primary text |
| `color.fg.secondary` | `#787c99` | `#555577` | Dimmed/secondary text |
| `color.fg.accent` | `#7aa2f7` | `#2255cc` | Headings, active labels |
| `color.fg.muted` | `#414868` | `#aaaacc` | Borders, separators |
| `color.border.normal` | `#414868` | `#aaaacc` | Inactive pane borders |
| `color.border.focused` | `#7aa2f7` | `#2255cc` | Active/focused pane borders |

**Syntax Highlight Tokens (JSON viewer):**

| Token | Dark Mode | Light Mode | Usage |
|-------|-----------|-----------|-------|
| `syntax.key` | `#7aa2f7` | `#2255cc` | JSON object keys |
| `syntax.string` | `#9ece6a` | `#336633` | String values |
| `syntax.number` | `#ff9e64` | `#993300` | Numeric values |
| `syntax.bool` | `#bb9af7` | `#6633cc` | true/false |
| `syntax.null` | `#565f89` | `#888899` | null values |
| `syntax.bracket` | `#89ddff` | `#005577` | `{}[]` |
| `syntax.error` | `#f7768e` | `#cc1133` | Error highlights |

**Status/Severity Color Tokens:**

| Token | Dark Mode | Light Mode | Paired Glyph | Usage |
|-------|-----------|-----------|-------------|-------|
| `status.connected` | `#9ece6a` | `#336633` | `●` (U+25CF) | Server connected |
| `status.disconnected` | `#787c99` | `#555577` | `○` (U+25CB) | Server disconnected |
| `status.error` | `#f7768e` | `#cc1133` | `✗` (U+2717) | Connection error |
| `status.warning` | `#e0af68` | `#aa6600` | `⚠` (U+26A0) | Warning state |
| `status.connecting` | `#7aa2f7` | `#2255cc` | `◌` (U+25CC) | Connecting/pending |
| `severity.critical` | `#f7768e` | `#cc1133` | `[CRIT]` | Critical severity |
| `severity.high` | `#ff9e64` | `#993300` | `[HIGH]` | High severity |
| `severity.medium` | `#e0af68` | `#aa6600` | `[MED]` | Medium severity |
| `severity.low` | `#9ece6a` | `#336633` | `[LOW]` | Low severity |
| `severity.info` | `#7aa2f7` | `#2255cc` | `[INFO]` | Informational |

> **Accessibility rule:** Every color-coded indicator MUST have a paired text label
> or glyph. Color is supplementary, never the sole indicator. (BC-3.08.005)

#### 256-Color Palette (Fallback)

Maps semantic tokens to xterm-256 color indices:

| Token | 256-color index | Notes |
|-------|----------------|-------|
| `status.connected` | `color 2` (green) | xterm-256 standard green |
| `status.disconnected` | `color 8` (dark gray) | |
| `status.error` | `color 1` (red) | |
| `status.warning` | `color 3` (yellow) | |
| `severity.critical` | `color 196` (bright red) | |
| `severity.high` | `color 208` (orange) | |
| `severity.medium` | `color 220` (yellow) | |
| `severity.low` | `color 46` (bright green) | |
| `syntax.key` | `color 75` (cornflower blue) | |
| `syntax.string` | `color 114` (light green) | |
| `syntax.number` | `color 215` (light orange) | |

#### 16-Color Palette (Minimum)

| Token | ANSI color | Modifier |
|-------|-----------|---------|
| `status.connected` | Green | Bold |
| `status.disconnected` | White | Dim |
| `status.error` | Red | Bold |
| `status.warning` | Yellow | Bold |
| `severity.critical` | Red | Bold |
| `severity.high` | Red | Normal |
| `severity.medium` | Yellow | Bold |
| `severity.low` | Green | Normal |
| `syntax.key` | Blue | Bold |
| `syntax.string` | Green | Normal |
| Border focused | Blue | Bold |
| Border normal | White | Dim |

---

### Character-Cell Spacing Grid

```
Base unit: 1 character cell (1 col × 1 row)

Spacing scale (in cells):
  space.0  = 0   (no gap)
  space.1  = 1   (tight — used between icon and label)
  space.2  = 2   (normal — used between columns in tables)
  space.4  = 4   (loose — used between major pane sections)

Padding conventions:
  Panel inner padding: 1 cell left/right, 0 top/bottom
  Table row height: 1 cell (single-line rows only)
  Section headers: 1 blank row above, 0 below
  Modal dialog: 1 cell padding all sides inside border
```

---

### Unicode Box Drawing & ASCII Fallback

| Element | Unicode | ASCII fallback |
|---------|---------|---------------|
| Top-left corner | `╔` (U+2554) | `+` |
| Top-right corner | `╗` (U+2557) | `+` |
| Bottom-left corner | `╚` (U+255A) | `+` |
| Bottom-right corner | `╝` (U+255D) | `+` |
| Horizontal line | `═` (U+2550) | `-` |
| Vertical line | `║` (U+2551) | `|` |
| T-junction (down) | `╦` (U+2566) | `+` |
| T-junction (up) | `╩` (U+2569) | `+` |
| T-junction (right) | `╠` (U+2560) | `+` |
| T-junction (left) | `╣` (U+2563) | `+` |
| Cross | `╬` (U+256C) | `+` |
| Light horizontal | `─` (U+2500) | `-` |
| Light vertical | `│` (U+2502) | `|` |
| Light top-left | `┌` (U+250C) | `+` |
| Light top-right | `┐` (U+2510) | `+` |
| Light bottom-left | `└` (U+2514) | `+` |
| Light bottom-right | `┘` (U+2518) | `+` |
| Focused pane border | Double-line (`╔═╗`) | `+--+` |
| Unfocused pane border | Light-line (`┌─┐`) | `+--+` |
| Scrollbar track | `░` (U+2591) | `:` |
| Scrollbar thumb | `█` (U+2588) | `#` |
| Selected marker | `▶` (U+25B6) | `>` |
| Bullet | `•` (U+2022) | `*` |
| Check | `✓` (U+2713) | `+` |
| Cross/Fail | `✗` (U+2717) | `x` |

> **ASCII fallback detection:** Check `$TERM` for `linux` console or if locale
> does not include `UTF-8`. Set `--unicode false` to force ASCII.

---

### Status Badge Definitions

Status badges appear inline in the server list and status bar.
Each badge has a color (see color tokens above), a glyph, and a text label.
Glyph + text label ensures accessibility when color rendering is disabled.

| State | Glyph | Text | Color Token | Context |
|-------|-------|------|-------------|---------|
| Connected | `●` | `CONN` | `status.connected` | Server is active and responding |
| Disconnected | `○` | `DISC` | `status.disconnected` | Server is known but not connected |
| Connecting | `◌` | `BUSY` | `status.connecting` | Connection in progress |
| Error | `✗` | `ERR ` | `status.error` | Connection failed / protocol error |
| Warning | `⚠` | `WARN` | `status.warning` | Degraded but functional |
| Unknown | `?` | `UNK ` | `color.fg.muted` | Status not yet determined |

ASCII fallback glyphs: `*` / `o` / `.` / `X` / `!` / `?`

---

### Severity Indicator Definitions

Used in security audit view (SCR-007) and traffic inspector (SCR-004).

| Severity | Badge | Color Token | Sort Order |
|----------|-------|-------------|-----------|
| Critical | `[CRIT]` | `severity.critical` | 1 (highest) |
| High | `[HIGH]` | `severity.high` | 2 |
| Medium | `[MED] ` | `severity.medium` | 3 |
| Low | `[LOW] ` | `severity.low` | 4 |
| Info | `[INFO]` | `severity.info` | 5 |

---

### Widget Component Contracts

#### Table Widget

```
Appearance:
  Header row: bold text, color.fg.accent, 1 blank row separator below
  Data rows: alternating bg (primary / secondary) OR plain, 1-cell height
  Selected row: bg color.bg.selected, fg color.fg.primary, bold
  Column separator: " │ " (space, light vertical, space) = 3 cells
  Scrollbar: right edge, track = ░, thumb = █

Keyboard:
  j / ↓       Next row
  k / ↑       Previous row
  g / Home    First row
  G / End     Last row
  /           Enter inline search/filter
  Enter       Select/expand row

Mouse (supplementary):
  Click       Select row
  Scroll      Scroll rows
```

#### List Widget

```
Appearance:
  Items: 1-cell height, left-aligned
  Selected: ▶ prefix + bg color.bg.selected
  Focused: double-line border (╔═╗)
  Unfocused: light border (┌─┐)

Keyboard:
  j/k         Navigate items
  Enter       Select/activate item
  /           Search/filter
```

#### Panel / Pane Widget

```
Appearance:
  Border: double-line when focused, light-line when unfocused
  Title: displayed in top border, centered, wrapped in " [ title ] "
  Padding: 1 cell left/right inside border
  Focus indicator: border color changes + title bold

Keyboard:
  Tab / Shift+Tab   Cycle focus between panes
  h/l               Horizontal pane focus shift (where applicable)
```

#### Sparkline Widget

```
Appearance:
  Height: 3-6 rows
  Characters: ▁▂▃▄▅▆▇█ (braille or block elements)
  Label: above or left, text
  Current value: right-aligned after sparkline
  Alert threshold: dashed line marker (if threshold set)
  Color: green below threshold, red above

Updates: 1Hz refresh tick
```

#### Progress Bar Widget

```
Appearance:
  Width: fills parent pane
  Fill: ████░░░░ (block + light shade)
  Label: "nn% (n/N)" right-aligned
  Colors: fg accent for fill, fg muted for empty
```

#### JSON Viewer Widget

```
Appearance:
  Line-by-line rendering with syntax tokens
  Indentation: 2-space standard
  Key/value coloring per syntax.* tokens
  Truncation: long values truncated with "…" (U+2026), expandable
  Line numbers: optional (toggleable)

Keyboard:
  j/k         Scroll lines
  Enter / →   Expand collapsed object/array
  ←           Collapse current level
  /           Search within viewer
  y           Yank (copy) current line or selection to clipboard
```

#### Tab Bar Widget

```
Appearance:
  Tabs: " Tab Name " with padding, separated by │
  Active tab: bold + underline OR color.fg.accent background
  Inactive tabs: color.fg.secondary

Keyboard:
  1-9         Jump to tab by number
  [ / ]       Cycle tabs left/right
  Tab key     Next tab (when tab bar has focus)
```

#### Modal Dialog Widget

```
Appearance:
  Centered overlay, minimum 40×12
  Double-line border (╔═╗)
  Dimmed background (color.fg.muted overlay on bg)
  Title in top border
  Buttons at bottom: [ Confirm ] [ Cancel ]

Keyboard:
  Escape      Dismiss/cancel
  Enter       Confirm default action
  Tab         Cycle between inputs and buttons
  Shift+Tab   Reverse cycle
```

---

## Global Keybinding Scheme

### Mode Model

The TUI operates in two global modes, inspired by vi:

| Mode | Trigger | Description |
|------|---------|-------------|
| **Normal mode** | Default; `Esc` exits other modes | Navigation, pane focus, selection |
| **Search mode** | `/` | Inline filter/search within current pane |
| **Command mode** | `:` | Command palette for less-frequent actions |
| **Input mode** | Specific dialogs (SCR-006) | Text/form input |

### Universal Keybindings (All Screens)

| Key | Action | Notes |
|-----|--------|-------|
| `q` / `Q` | Quit TUI (confirm if unsaved) | |
| `?` | Show help overlay | Lists all current keybindings |
| `Tab` | Next pane (cycle forward) | |
| `Shift+Tab` | Previous pane (cycle backward) | |
| `h` / `←` | Focus left pane | Where layout permits |
| `l` / `→` | Focus right pane | Where layout permits |
| `j` / `↓` | Move down in focused widget | |
| `k` / `↑` | Move up in focused widget | |
| `g` / `Home` | Jump to first item | |
| `G` / `End` | Jump to last item | |
| `Enter` | Select / expand / activate | Context-dependent |
| `Esc` | Cancel / close modal / exit mode | |
| `/` | Open search/filter bar | Within current pane |
| `:` | Open command mode | Global action palette |
| `r` | Refresh / reload current view | |
| `Ctrl+C` | Interrupt / quit (no confirm) | Emergency exit |
| `Ctrl+L` | Redraw / clear and repaint | For terminal glitches |
| `F1` | Help overlay | Alias for `?` |

### Screen-Specific Keybindings

Defined in each screen's individual file. Summary:

| Context | Key | Action |
|---------|-----|--------|
| Server sidebar | `Enter` | Connect/disconnect selected server |
| Server sidebar | `n` | Add new server connection |
| Server sidebar | `d` | Delete server from registry |
| Capability browser | `[` / `]` | Cycle tabs (Tools/Resources/Prompts) |
| Capability browser | `e` | Execute selected tool (→ SCR-006) |
| Traffic inspector | `c` | Clear capture buffer |
| Traffic inspector | `f` | Open filter dialog |
| Traffic inspector | `x` | Expand selected message |
| Traffic inspector | `R` | Replay selected message |
| Health metrics | `t` | Edit alert thresholds |
| Security audit | `s` | Start new audit |
| Security audit | `E` | Export audit report |
| Conformance runner | `s` | Start test suite |
| Conformance runner | `f` | Filter by fail/pass |
| Config drift | `s` | Source selector |
| Server comparison | `Tab` | Switch focus between server columns |

---

## Layout System

### Main Dashboard Layout (80×24 minimum)

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Forge MCP │ server-name │ CONN ●  │  [Tab: Capabilities] [Health] [Traffic] │
├──────────────┬──────────────────────────────────────┬────────────────────────┤
│ Servers      │ Content Area                         │ Health / Metrics       │
│ (22% width)  │ (54% width)                          │ (24% width)            │
│              │                                      │                        │
│              │                                      │                        │
├──────────────┴──────────────────────────────────────┴────────────────────────┤
│ Status Bar: [Daemon: OK] [Capture: ON] [nn msgs] [latency: NNms] [q:quit ?:help]│
└─────────────────────────────────────────────────────────────────────────────┘
```

### Narrow Layout (80–100 cols)

At < 100 columns, the right Health panel collapses into a tab within the content area.

```
Cols < 100:
  Left sidebar: 20 cols fixed
  Content area: remaining cols
  Health: tab within content, not side panel
```

### Wide Layout (> 160 cols)

Sidebar expands to 30 cols. Content area gets larger. Health panel expands to 28 cols.

---

## Accessibility Checklist (TUI-Specific)

- [x] All interactive elements keyboard-accessible (hjkl + Enter + Tab)
- [x] Focus indicators visible on all focusable panes (double-line border)
- [x] No color-only indicators — every status/severity uses glyph + text + color (BC-3.08.005)
- [x] Full keyboard-only operation without mouse (BC-3.07.001)
- [x] Focus management: modal open traps focus; Escape returns focus to prior pane
- [x] Search mode (`/`) accessible from any pane, exits cleanly with `Esc`
- [x] Screen reader note: ratatui renders to terminal — standard terminal a11y applies
- [x] No animations that cannot be disabled (`$NO_COLOR` and monochrome mode)
- [x] ASCII fallback for all Unicode box drawing and glyphs
- [x] Status bar always visible providing global context
- [x] Minimum 4.5:1 contrast ratio maintained in 16-color mode (dark bg + bright fg)

---

## Performance Targets (TUI)

| Metric | Target | Source |
|--------|--------|--------|
| Sustained render frame rate | ≥ 60fps at 100 events/sec | NFR from product brief |
| Memory footprint (RSS) | < 5MB | NFR from product brief |
| Event-to-render latency | < 16ms | Required for 60fps |
| Capture buffer memory | < 100MB max | BC-4.09.003 |
| Color detection startup | < 1ms | Synchronous env-var read |
| Terminal size detection | On SIGWINCH | OS signal handler |

---

## BC Traceability (Global)

| BC ID | Addressed By |
|-------|-------------|
| BC-3.06.001 | Layout system; min 80×24; proportional expansion |
| BC-3.06.002 | Color system auto-detection (truecolor→256→16→mono) |
| BC-3.07.001 | Global keybinding scheme; hjkl + Tab navigation |
| BC-3.07.002 | Search mode (`/`) and command mode (`:`) |
| BC-3.07.003 | Mouse supplementary input; `--mouse` flag |
| BC-3.08.001 | JSON viewer widget contract; syntax highlight tokens |
| BC-3.08.002 | Sparkline + histogram widget contracts |
| BC-3.08.003 | Status badge definitions (connected/error/warning) |
| BC-3.08.004 | Tab-based capability explorer in SCR-003 |
| BC-3.08.005 | Accessibility rule: glyph+text+color for every indicator |
