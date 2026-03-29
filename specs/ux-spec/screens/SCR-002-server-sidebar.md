---
document_type: ux-spec-screen
screen_id: SCR-002
screen_name: Server Sidebar
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-03-29T14:15:00
phase: 1c
complexity: complex
traces_to: UX-INDEX.md
prd_requirements: [BC-3.07.001, BC-3.08.003, BC-1.02.001, BC-1.02.002, BC-1.02.003]
---

# Screen: Server Sidebar (SCR-002)

> Left pane of the main dashboard. Displays all discovered MCP servers with
> connection status. Supports keyboard navigation, search/filter, and connect/
> disconnect actions. The authoritative server selection control for the entire TUI.

---

## Wireframe

### Normal State (22 cols wide)

```
╔══════════════════════╗
║  SERVERS          + ║
║  ──────────────────  ║
║  Filter: [        ]  ║
║                      ║
║  claude-desktop      ║
║  ▶ ● my-server  CONN ║  ← selected + connected
║    ○ staging    DISC ║  ← disconnected
║    ✗ old-api    ERR  ║  ← error
║    ◌ local-dev  BUSY ║  ← connecting
║    ○ test-srv   DISC ║
║                      ║
║  cursor              ║
║    ○ cursor-mcp DISC ║
║                      ║
║  ──────────────────  ║
║  5 servers  1 conn   ║
╚══════════════════════╝
```

### Focused State (double border)

```
╔══════════════════════╗   ← double-line border = focused
║  SERVERS          +  ║
║  ══════════════════  ║
 ...
```

### Search Active State

```
╔══════════════════════╗
║  SERVERS     /search ║   ← mode indicator in title
║  ──────────────────  ║
║  Filter: [my-s    ]  ║   ← active input cursor
║                      ║
║  ▶ ● my-server  CONN ║   ← filtered results only
║                      ║
║  1 match             ║
╚══════════════════════╝
```

### Detail Tooltip (on selected server)

```
╔══════════════════════╗
║  SERVERS             ║
║  ▶ ● my-server  CONN ║
║  ┌────────────────┐  ║
║  │ Transport: stdio│  ║
║  │ Source: claude  │  ║
║  │ Proto: 2025-11  │  ║
║  │ Caps: T R P S L │  ║
║  │ Enter:connect   │  ║
║  └────────────────┘  ║
╚══════════════════════╝
```

---

## Element Inventory

| ID | Widget | Type | Position | Description |
|----|--------|------|----------|-------------|
| ELM-001 | Panel border | Panel | Full pane | Focused/unfocused border |
| ELM-002 | Title | Text | Top border | "SERVERS" |
| ELM-003 | Add button | Button | Top border, right | `+` — add server |
| ELM-004 | Filter input | TextInput | Row 2 | Inline search field |
| ELM-005 | Source group label | Text | Before each group | Config source name (e.g., `claude-desktop`) |
| ELM-006 | Server row | ListItem | Data rows | Per-server row (see row anatomy below) |
| ELM-007 | Selected marker | Glyph | Row prefix | `▶` on selected, ` ` otherwise |
| ELM-008 | Status glyph | StatusGlyph | Row col 2 | `●`/`○`/`✗`/`◌`/`?` |
| ELM-009 | Server name | Text | Row col 3-N | Name, truncated with `…` if too long |
| ELM-010 | Status badge | Badge | Row right | `CONN`/`DISC`/`ERR `/`BUSY`/`UNK ` |
| ELM-011 | Footer summary | Text | Last row | "N servers  M conn" |
| ELM-012 | Detail tooltip | Popup | Over/beside row | Shown on `i` key or hover |

### Server Row Anatomy

```
  <marker> <glyph> <name…………………> <badge>
   1 col    1 col   N cols          4 cols

Examples:
  ▶ ● my-server         CONN    (selected, connected)
    ○ staging           DISC    (unselected, disconnected)
    ✗ old-api           ERR     (unselected, error)
    ◌ local-dev         BUSY    (unselected, connecting)
```

Name truncation: if `name.len() > (pane_width - 9)`, truncate with `…`.

---

## Grouping by Config Source

Servers are grouped by their originating config source:
- `claude-desktop` → Claude Desktop config
- `cursor` → Cursor config
- `vscode` → VS Code config
- `windsurf` → Windsurf config
- `manual` → manually added servers

Group headers are dimmed (color.fg.secondary), non-selectable.
If only one source, header is hidden.

---

## State Definitions

| State Variable | Type | Description |
|----------------|------|-------------|
| `servers` | Vec<ServerEntry> | All discovered servers |
| `selected_idx` | usize | Currently highlighted server index |
| `selected_server_id` | Option<ServerId> | ID of highlighted server |
| `filter_text` | String | Current filter input content |
| `filter_active` | bool | Filter input has keyboard focus |
| `grouped_view` | bool | Group by config source (default: true) |
| `tooltip_visible` | bool | Detail tooltip shown |
| `scroll_offset` | usize | Scroll position for long server lists |

### ServerEntry Data

| Field | Type | Description |
|-------|------|-------------|
| `id` | ServerId | Unique identifier |
| `name` | String | Display name |
| `transport` | Enum: stdio\|http | Connection transport |
| `source` | Enum: claude\|cursor\|vscode\|windsurf\|manual | Origin config |
| `status` | Enum: connected\|disconnected\|connecting\|error\|unknown | Live status |
| `protocol_version` | Option<String> | Negotiated MCP version |
| `capabilities` | CapabilitySet | tools/resources/prompts/etc. |
| `error_message` | Option<String> | Last error if status=error |

---

## Keyboard Shortcuts

| Key | Action | Notes |
|-----|--------|-------|
| `j` / `↓` | Move selection down | Wraps at bottom |
| `k` / `↑` | Move selection up | Wraps at top |
| `g` / `Home` | Jump to first server | |
| `G` / `End` | Jump to last server | |
| `Enter` | Connect selected (if disconnected) / Show detail | |
| `Enter` (connected) | Switch active server + populate content area | |
| `Space` | Disconnect selected server | Confirm if active |
| `/` | Activate filter input | |
| `Esc` | Clear filter / exit filter mode | |
| `n` | Add new server (→ add-server dialog) | P1 |
| `d` | Delete selected server from registry | Confirm dialog |
| `i` | Toggle detail tooltip | |
| `r` | Refresh server list (re-scan configs) | |
| `Tab` | Move focus to Content Area | |

---

## Connection Lifecycle Display

| Transition | Visual | Duration |
|-----------|--------|---------|
| Disconnected → Connecting | Glyph changes to `◌` (U+25CC), badge to `BUSY`, pulsing (if color avail) | Until result |
| Connecting → Connected | Glyph → `●`, badge → `CONN`, green color | Instant |
| Connecting → Error | Glyph → `✗`, badge → `ERR `, red color, error tooltip | Instant |
| Connected → Disconnected | Glyph → `○`, badge → `DISC` | Instant |

Pulsing animation: `◌` rotates through `◌ ◎ ●` at 4Hz if truecolor/256 color available. In 16-color mode, static `◌` glyph only.

---

## Filter Behavior

- Filter input appears in row 2 of the pane (always visible, not modal)
- Filter is case-insensitive substring match on server name
- Filtered results hide non-matching servers; group headers hide if group is empty
- `Esc` clears filter and returns to normal navigation
- Filter text preserved across pane focus changes
- `Enter` while filter is active: selects the top visible result

---

## Accessibility Notes

- **No color-only:** Each status uses glyph + text badge + color. Status = error shows `✗` glyph + `ERR ` text + red color (BC-3.08.005)
- **Focus indicator:** Focused pane has double-line border
- **Keyboard-only:** All actions reachable without mouse
- **Screen orientation:** Footer shows summary count ("5 servers  1 conn") for quick overview
- **Focus order within pane:** Filter input → Server list items → Footer (Tab within pane)
- **Tooltip focus:** When tooltip is open, `Esc` closes it; focus returns to list item

---

## Error States

| Error | Display |
|-------|---------|
| Config parse error | Server shown with `?` glyph + `UNK ` badge + tooltip with error message |
| Connection refused | `✗` glyph + `ERR ` badge + tooltip: "Connection refused: <detail>" |
| Timeout | `✗` glyph + `ERR ` badge + tooltip: "Timeout after 30s" |
| No servers discovered | Empty list + centered message: "No servers found.\nRun: forge-mcp list" |

---

## BC Traceability

| BC ID | How This Screen Satisfies It |
|-------|------------------------------|
| BC-3.07.001 | j/k navigation, Enter to connect |
| BC-3.07.002 | `/` opens filter/search input |
| BC-3.08.003 | Status badges (glyph + text + color) for all connection states |
| BC-3.08.005 | No color-only indicators; glyph + text always present |
| BC-1.02.001 | Connect action triggers stdio transport connection |
| BC-1.02.002 | Connect action supports HTTP transport |
| BC-1.02.003 | Disconnect action triggers graceful lifecycle management |
| BC-1.01.001 | Servers grouped by config source (discovery paths shown in tooltip) |
