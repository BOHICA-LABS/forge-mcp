---
document_type: design-system-tokens
product: forge-mcp
platform: TUI (ratatui + crossterm)
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-03-29T14:15:00
traces_to: ../UX-INDEX.md
---

# Forge MCP TUI Design Tokens

> Token definitions for the ratatui/crossterm terminal UI.
> Three color tiers: truecolor (24-bit), 256-color, 16-color ANSI.
> All tokens map across all tiers. The renderer selects the active tier
> at startup based on terminal capability detection (see UX-INDEX.md §Color System).

---

## Color Tokens

### Background Colors

| Token Name | Truecolor Hex | xterm-256 | ANSI 16 | Usage |
|-----------|--------------|-----------|---------|-------|
| `bg.primary` | `#1a1b26` | color 234 | Black | Main background |
| `bg.secondary` | `#24283b` | color 235 | Black + bold | Pane/panel background |
| `bg.selected` | `#364a82` | color 62 | Blue | Selected row highlight |
| `bg.focused` | `#2d3561` | color 61 | Blue + dim | Focused pane highlight |
| `bg.modal` | `#1a1b26` | color 234 | Black | Modal overlay background |
| `bg.tooltip` | `#1f2335` | color 235 | Black | Tooltip background |

### Foreground / Text Colors

| Token Name | Truecolor Hex | xterm-256 | ANSI 16 | Usage |
|-----------|--------------|-----------|---------|-------|
| `fg.primary` | `#c0caf5` | color 189 | White | Primary text |
| `fg.secondary` | `#787c99` | color 243 | White + dim | Dimmed/secondary text |
| `fg.accent` | `#7aa2f7` | color 111 | Blue + bold | Headings, active labels, tab names |
| `fg.muted` | `#414868` | color 238 | Black + bold | Borders (inactive), separators |
| `fg.highlight` | `#ffffff` | color 231 | White + bold | High-emphasis text in alerts |

### Border Colors

| Token Name | Truecolor Hex | xterm-256 | ANSI 16 | Usage |
|-----------|--------------|-----------|---------|-------|
| `border.normal` | `#414868` | color 238 | White + dim | Inactive pane borders |
| `border.focused` | `#7aa2f7` | color 111 | Blue + bold | Active/focused pane borders |
| `border.alert` | `#f7768e` | color 204 | Red | Alert-state pane borders |

### Status Colors (paired with glyphs — accessibility-safe)

| Token Name | Truecolor Hex | xterm-256 | ANSI 16 | Paired Glyph |
|-----------|--------------|-----------|---------|-------------|
| `status.connected` | `#9ece6a` | color 150 | Green | `●` (U+25CF) |
| `status.disconnected` | `#787c99` | color 243 | White + dim | `○` (U+25CB) |
| `status.connecting` | `#7aa2f7` | color 111 | Blue | `◌` (U+25CC) |
| `status.error` | `#f7768e` | color 204 | Red + bold | `✗` (U+2717) |
| `status.warning` | `#e0af68` | color 179 | Yellow + bold | `⚠` (U+26A0) |
| `status.unknown` | `#565f89` | color 61 | White + dim | `?` |

### Severity Colors (text badge always present)

| Token Name | Truecolor Hex | xterm-256 | ANSI 16 | Text Badge |
|-----------|--------------|-----------|---------|-----------|
| `severity.critical` | `#f7768e` | color 204 | Red + bold | `[CRIT]` |
| `severity.high` | `#ff9e64` | color 215 | Red | `[HIGH]` |
| `severity.medium` | `#e0af68` | color 179 | Yellow + bold | `[MED] ` |
| `severity.low` | `#9ece6a` | color 150 | Green | `[LOW] ` |
| `severity.info` | `#7aa2f7` | color 111 | Blue | `[INFO]` |

### Syntax Highlight Colors (JSON Viewer)

| Token Name | Truecolor Hex | xterm-256 | ANSI 16 | Usage |
|-----------|--------------|-----------|---------|-------|
| `syntax.key` | `#7aa2f7` | color 111 | Blue + bold | JSON object keys |
| `syntax.string` | `#9ece6a` | color 150 | Green | String values |
| `syntax.number` | `#ff9e64` | color 215 | Red | Numeric values |
| `syntax.bool` | `#bb9af7` | color 183 | Magenta | `true` / `false` |
| `syntax.null` | `#565f89` | color 61 | White + dim | `null` |
| `syntax.bracket` | `#89ddff` | color 117 | Cyan | `{`, `}`, `[`, `]` |
| `syntax.error` | `#f7768e` | color 204 | Red | Error-flagged content |

### Alert/Threshold Sparkline Colors

| Token Name | Truecolor Hex | xterm-256 | ANSI 16 | Usage |
|-----------|--------------|-----------|---------|-------|
| `sparkline.ok` | `#9ece6a` | color 150 | Green | Below threshold |
| `sparkline.warn` | `#e0af68` | color 179 | Yellow | 80–100% of threshold |
| `sparkline.breach` | `#f7768e` | color 204 | Red + bold | Above threshold |

---

## Light Terminal Variant

For terminals with light (white/cream) backgrounds. Applied when `$COLORFGBG` second value is < 8, or when `--theme light` is set.

| Token | Light Mode Hex | xterm-256 | Notes |
|-------|---------------|-----------|-------|
| `bg.primary` | `#f0f0f0` | color 255 | |
| `bg.secondary` | `#e0e0e0` | color 254 | |
| `bg.selected` | `#aaccff` | color 153 | |
| `fg.primary` | `#1a1b26` | color 234 | |
| `fg.accent` | `#2255cc` | color 26 | |
| `border.normal` | `#aaaacc` | color 146 | |
| `border.focused` | `#2255cc` | color 26 | |
| `status.connected` | `#336633` | color 22 | |
| `status.error` | `#cc1133` | color 160 | |
| `syntax.key` | `#2255cc` | color 26 | |
| `syntax.string` | `#336633` | color 22 | |

---

## Typography Tokens

In character-cell terminals, typography is expressed as modifiers on the monospace font provided by the terminal emulator.

| Token | Modifier | Usage |
|-------|---------|-------|
| `text.heading` | Bold | Pane titles, section headers |
| `text.body` | Normal | Regular content |
| `text.secondary` | Dim | Metadata, secondary labels |
| `text.code` | Normal | JSON content (monospace by definition) |
| `text.emphasis` | Bold | Key values, important labels |
| `text.error` | Bold + `severity.critical` color | Error messages |
| `text.success` | Bold + `status.connected` color | Success indicators |

---

## Spacing Tokens

| Token | Value (cells) | Usage |
|-------|--------------|-------|
| `space.0` | 0 | No gap |
| `space.1` | 1 | Glyph-to-label gap, icon spacing |
| `space.2` | 2 | Column separators in tables |
| `space.3` | 3 | Section separators |
| `space.4` | 4 | Major pane section gaps |
| `padding.panel.h` | 1 | Horizontal inner padding in panels |
| `padding.panel.v` | 0 | Vertical inner padding in panels |
| `padding.modal.all` | 1 | Uniform padding inside modals |

---

## Sizing Tokens

| Token | Value | Usage |
|-------|-------|-------|
| `terminal.min.cols` | 80 | Minimum terminal width |
| `terminal.min.rows` | 24 | Minimum terminal rows |
| `terminal.narrow.threshold` | 100 | Below this: collapse health pane |
| `terminal.wide.threshold` | 160 | Above this: expand panes |
| `pane.sidebar.width.default` | 22% of cols | Server sidebar |
| `pane.sidebar.width.min` | 18 cols | Fixed minimum |
| `pane.sidebar.width.wide` | 30 cols | Wide-terminal fixed |
| `pane.health.width.default` | 24% of cols | Health panel |
| `pane.health.width.wide` | 28 cols | Wide-terminal fixed |
| `modal.min.width` | 40 cols | Modal dialogs |
| `modal.min.height` | 12 rows | Modal dialogs |
| `sparkline.height` | 1 row | Sparkline character height |
| `sparkline.history` | 60 points | 60 seconds at 1Hz |
| `table.row.height` | 1 row | All table rows single-line |
| `status.bar.height` | 1 row | Global status bar |
| `header.bar.height` | 1 row | Global header bar |

---

## Widget Component Contracts Reference

> Full widget rendering contracts are specified in UX-INDEX.md §Widget Component Contracts.
> This section provides the token → widget mapping.

| Widget | Tokens Used |
|--------|------------|
| Table | `bg.selected`, `bg.secondary`, `fg.primary`, `fg.secondary`, `fg.accent` (headers), `border.normal`/`border.focused` |
| List | `bg.selected`, `fg.primary`, `border.normal`/`border.focused` |
| Panel/Pane | `bg.secondary`, `border.normal`/`border.focused`, `fg.accent` (title) |
| Sparkline | `sparkline.ok`/`sparkline.warn`/`sparkline.breach`, `fg.muted` (axis) |
| Progress bar | `fg.accent` (fill), `fg.muted` (empty), `fg.primary` (label) |
| JSON viewer | All `syntax.*` tokens |
| Tab bar | `fg.accent` (active), `fg.secondary` (inactive), `border.normal` |
| Modal | `bg.modal`, `border.focused`, `fg.primary` |
| Badge / status | `status.*` tokens + glyph |
| Severity badge | `severity.*` tokens + text badge |

---

## Box Drawing Character Set Reference

See UX-INDEX.md §Unicode Box Drawing & ASCII Fallback for the complete character set.

| Mode | Activated By |
|------|-------------|
| Unicode (double-line focus) | Default when locale is UTF-8 |
| Unicode (light-line unfocused) | Default when locale is UTF-8 |
| ASCII fallback | `$TERM == "linux"` or `--unicode false` or non-UTF-8 locale |
| Auto-detect | `--unicode auto` (default) — checks `$LANG`/`$LC_ALL` |

---

## Status Badge Character Reference

| State | Unicode Glyph | ASCII Fallback | Color Token |
|-------|--------------|---------------|-------------|
| Connected | `●` U+25CF | `*` | `status.connected` |
| Disconnected | `○` U+25CB | `o` | `status.disconnected` |
| Connecting | `◌` U+25CC | `.` | `status.connecting` |
| Error | `✗` U+2717 | `X` | `status.error` |
| Warning | `⚠` U+26A0 | `!` | `status.warning` |
| Pass | `✓` U+2713 | `+` | `status.connected` |
| Fail | `✗` U+2717 | `x` | `status.error` |
| Running | `◌` U+25CC | `.` | `status.connecting` |
| Pending | `○` U+25CB | `o` | `status.disconnected` |
| Selected | `▶` U+25B6 | `>` | `fg.accent` |
| Bullet | `•` U+2022 | `*` | `fg.secondary` |

---

## Animation Tokens

| Animation | Frames | Rate | Degradation |
|-----------|--------|------|-------------|
| Connecting spinner | `◌ ◎ ●` | 4Hz | Static `◌` in 16-color |
| Progress pulse | `████░░░░` scrolling | 4Hz | Static bar in monochrome |
| Sparkline tick | New data point appended | 1Hz | Always — no animation |

No animations require color to convey meaning — glyph changes are the primary indicator.
