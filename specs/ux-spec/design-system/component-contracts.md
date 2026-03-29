---
document_type: design-system-component-contracts
product: forge-mcp
platform: TUI (ratatui + crossterm)
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-03-29T14:15:00
traces_to: ../UX-INDEX.md
---

# Forge MCP TUI Component Contracts

> Detailed rendering specifications for each reusable widget.
> Implementers (forge-tui crate) MUST follow these contracts.
> All contracts use character-cell grid units.

---

## 1. Table Widget

### Usage
Capability Browser (SCR-003), Traffic Inspector (SCR-004), Security Audit (SCR-007),
Conformance Runner (SCR-008), Config Drift (SCR-009), Server Comparison (SCR-010).

### Visual Specification

```
HEADER ROW:
  Modifier: Bold, fg.accent color
  Separator: 1 blank row OR horizontal line (─) between header and data

DATA ROW:
  Height: 1 cell (single-line, truncated with … if overflow)
  Selected row: bg.selected background, fg.primary foreground, Bold modifier
  Alternating rows: optional bg.secondary tint (can be disabled)
  Column separator: " │ " (1 space + │ + 1 space = 3 cells)
  Left padding: 1 cell minimum

SCROLLBAR (right edge):
  Track character: ░ (U+2591) | ASCII: :
  Thumb character: █ (U+2588) | ASCII: #
  Visible when content height > pane height
  Width: 1 cell

SELECTED MARKER:
  Prefix on selected row: ▶ (U+25B6) | ASCII: >
  1 cell wide, 1 space gap before first column
```

### Keyboard Contract

| Key | Action |
|-----|--------|
| `j` / ↓ | Next row (does NOT wrap at bottom) |
| `k` / ↑ | Previous row (does NOT wrap at top) |
| `g` / Home | Jump to first row |
| `G` / End | Jump to last row |
| Page Down | Advance by pane height - 2 |
| Page Up | Retreat by pane height - 2 |
| `/` | Enter inline search/filter |

### Sort Contract

- Sortable columns indicated by sort glyphs in header: `▲` (asc) / `▼` (desc) / ` ` (unsorted)
- `s` key cycles: unsorted → ascending → descending → unsorted
- Mouse: click header cell to sort (supplementary)
- Sort state persists while pane is open; resets on screen change

---

## 2. List Widget

### Usage
Server Sidebar (SCR-002).

### Visual Specification

```
ITEM:
  Height: 1 cell
  Prefix space: 2 cells (for ▶ marker + gap)
  Selected: ▶ marker + bg.selected background
  Unselected: 2 spaces + bg.secondary (or primary) background
  Group header (non-selectable): fg.secondary color, no prefix space, dim modifier

BORDER:
  Focused: double-line (╔═╗)
  Unfocused: light-line (┌─┐)
  Title: displayed in top border center: " [ TITLE ] "
```

### Keyboard Contract (same as Table)

---

## 3. Panel / Pane Widget

### Usage
All panes in SCR-001 layout, modals in SCR-006.

### Visual Specification

```
BORDER FOCUSED:
  Top-left: ╔  Top-right: ╗  Bottom-left: ╚  Bottom-right: ╝
  Horizontal: ═  Vertical: ║
  Color: border.focused (blue)
  Title: ║ ═ ═  TITLE  ═ ═ ║ (centered in top border)

BORDER UNFOCUSED:
  Characters: ┌─┐└─┘│
  Color: border.normal (dim)

ALERT STATE BORDER:
  Characters: same as focused
  Color: border.alert (red) + Bold modifier

PADDING:
  Inner: 1 cell left, 1 cell right
  Top/bottom: 0 (or 1 for modals)
```

### Focus Indicator Contract

Focus is exclusively indicated by:
1. Double-line vs light-line border characters
2. Border color change (blue vs dim)

NOT by background color change alone (accessibility).

---

## 4. Sparkline Widget

### Usage
Health Metrics Panel (SCR-005).

### Visual Specification

```
RENDERING:
  Characters: ▁▂▃▄▅▆▇█ (8-level Unicode blocks)
  ASCII fallback: ........::::::::########
  Height: 1 character row (single-row sparkline)
  Width: pane_width - label_width (3) - value_width (8) - 2 (padding)

LAYOUT (per sparkline line):
  "p50 ▁▂▃▄▅▆▇█▇█  42ms"
   ^^^  ←─────→    ^^^^^
  label  sparkline  value (right-aligned, 8 chars)

COLORS:
  Value < threshold: sparkline.ok (green)
  Value 80-100% of threshold: sparkline.warn (amber)
  Value > threshold: sparkline.breach (red)
  In 16-color: use ANSI green/yellow/red

THRESHOLD LINE:
  Normal: "── threshold: Nms ──" (fg.muted)
  Breached: "══ threshold: Nms ══" (border.alert, Bold)
  Position: 1 row below sparkline group
```

### Update Contract

- Data source: forge-health tick events (1Hz)
- History length: 60 data points (1 minute at 1Hz)
- On tick: shift left, append new value
- Thread safety: ring buffer read; no locking in render path

---

## 5. Progress Bar Widget

### Usage
Conformance Runner (SCR-008), Tool Execution Dialog (SCR-006 executing state), Security Audit (SCR-007 scanning).

### Visual Specification

```
CHARACTERS:
  Fill: █ (U+2588) | ASCII: #
  Empty: ░ (U+2591) | ASCII: .
  Width: pane_width - label_width - 2

LAYOUT:
  "██████████████████░░░░░  78%  (47 / 60)"
   ←── fill ──→←empty→   ^^^   ←───────→
                          pct   count

COLORS:
  Fill: fg.accent
  Empty: fg.muted
  Label: fg.primary

INDETERMINATE (pulsing):
  Pattern: scrolling block "████░░░░" moves right at 4Hz
  ASCII: "####...." scrolling
  Label: "Executing… NNNms" (elapsed timer)
```

---

## 6. JSON Viewer Widget

### Usage
Traffic Inspector detail view (SCR-004), Tool Execution result (SCR-006), Finding evidence (SCR-007).

### Visual Specification

```
RENDERING:
  Line-by-line. Each line independently tokenized.
  Indentation: 2 spaces per level
  Key coloring: syntax.key
  Value coloring: per type (syntax.string, .number, .bool, .null)
  Bracket coloring: syntax.bracket
  Line numbers: left-aligned, fg.muted, right-padded to 4 chars + " "

LINE TRUNCATION:
  Lines longer than pane_width - 1: truncated with "…" (U+2026) | ASCII: "..."
  Hover or Enter: expands truncated line to scrollable sub-view (optional)

SCROLLBAR: right-edge, same contract as Table scrollbar

COLLAPSE/EXPAND:
  Objects/arrays: show "{ … }" when collapsed
  → key expands, ← key collapses current level
  Default: all levels expanded to depth 2; deeper levels collapsed
```

### Keyboard Contract

| Key | Action |
|-----|--------|
| `j` / ↓ | Scroll down 1 line |
| `k` / ↑ | Scroll up 1 line |
| `→` / `Enter` | Expand object/array at cursor |
| `←` | Collapse current level |
| `/` | Search within viewer (highlights all matches) |
| `n` / `N` | Next / previous match |
| `y` | Yank current line or selection to clipboard |
| `g` / `G` | Jump to top / bottom |

---

## 7. Tab Bar Widget

### Usage
Capability Browser (SCR-003), Main Dashboard tabs (SCR-001 header).

### Visual Specification

```
LAYOUT:
  "[ Tools(12) │ Resources(5) │ Prompts(3) ]"
   ^─── tab ──^│^─── tab ────^│^─── tab ──^

ACTIVE TAB:
  Bold modifier + fg.accent color
  Optional: underline modifier

INACTIVE TAB:
  fg.secondary color, normal modifier

TAB SEPARATOR:
  " │ " (space + light vertical + space)

BADGE:
  "(N)" suffix on tab label showing item count
  Color: fg.secondary when inactive, fg.accent when active
```

### Keyboard Contract

| Key | Action |
|-----|--------|
| `[` | Previous tab (wraps) |
| `]` | Next tab (wraps) |
| `1`–`9` | Jump to tab N |
| `Tab` (within pane) | Next tab |

---

## 8. Modal Dialog Widget

### Usage
Tool Execution Dialog (SCR-006), Suppress Dialog, Export Dialog, Threshold Editor.

### Visual Specification

```
OVERLAY:
  Centers on terminal
  Background dims: fg.muted color applied to all background characters
  Minimum size: 40 cols × 12 rows
  Maximum size: 80% of terminal width, 80% of terminal height

BORDER:
  Double-line focused (╔═╗╚═╝║)
  Color: border.focused
  Title: centered in top border

BUTTONS (bottom row):
  "  [ Confirm ]    [ Cancel ]  "
  Selected button: bg.selected + fg.primary + Bold
  Unselected: fg.secondary
  Separator: 4 spaces between buttons
```

### Focus Contract

- On open: focus trapped inside dialog
- Focus order: first input field → ... → last input → Confirm button → Cancel button → cycle
- `Tab` cycles forward, `Shift+Tab` cycles backward
- `Escape` always maps to Cancel/Close
- `Enter` in last field OR on Confirm button: submits form
- On close: focus returns to element that triggered open

---

## 9. Filter Bar Widget (Inline)

### Usage
Traffic Inspector filter (SCR-004), Security Audit filter (SCR-007).

### Visual Specification

```
LAYOUT:
  "┌ Filter ──────────────────────────────────── ┐"
  "│ Method: [tools/call  ] Dir: [both▼] ... │"
  "└─────────────────────────────────────────────── ┘"

FIELDS:
  TextInput: single-line, bg.secondary background, cursor visible
  Dropdown: current value + ▼ indicator; Enter opens option list
  Active field: border.focused border

VISIBILITY:
  Opens on `f` key; closes on second `f` or `Esc`
  When active: displaces message table by 3 rows
  Filter persists when bar is closed; "Filter: active (N/M)" shown in title
```

---

## 10. Status Bar Widget

### Usage
SCR-001 bottom bar.

### Visual Specification

```
LAYOUT (1 row, full width):
  " [Daemon: OK]  [Capture: ON | 47 msgs]  [Latency: 42ms]  v0.1.0   q:quit ?:help "
   ^───────────^  ^──────────────────────^ ^──────────────^  ^──────^ ^────────────^
   segment 1      segment 2                segment 3          version  key hints

SEGMENT FORMATTING:
  Normal segment: "[ label: value ]" — fg.secondary brackets, fg.primary value
  Alert segment:  "[ ⚠ label: value ]" — status.warning color, Bold modifier
  Error segment:  "[ ✗ label: value ]" — status.error color, Bold modifier

SEPARATOR:
  2 spaces between segments

KEY HINTS (right-aligned):
  Context-sensitive. Always include: "q:quit ?:help"
  Additional hints added by active pane
```

---

## Accessibility Contract (All Widgets)

1. **No color-only information.** Every piece of information conveyed by color MUST also be conveyed by at least one of: glyph symbol, text label, or positional indicator.

2. **Focus always visible.** Every focusable widget MUST have a visually distinct focused state that does not rely solely on color (border character change required).

3. **Motion is non-essential.** All animations (spinner, sparkline, progress) are non-essential. The UI is fully functional when animations show static state.

4. **Text labels on all status indicators.** Status badges always include the text portion (CONN/DISC/ERR/etc.) even when glyph is present.

5. **Keyboard complete.** Every user action achievable by keyboard alone. Mouse is supplementary only.
