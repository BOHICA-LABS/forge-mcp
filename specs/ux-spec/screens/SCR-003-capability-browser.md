---
document_type: ux-spec-screen
screen_id: SCR-003
screen_name: Capability Browser
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-03-29T14:15:00
phase: 1c
complexity: complex
traces_to: UX-INDEX.md
prd_requirements: [BC-3.08.004, BC-2.05.001, BC-2.05.002, BC-2.05.003, BC-3.07.001]
---

# Screen: Capability Browser (SCR-003)

> Top section of the content area. Tab-based view showing Tools, Resources, and
> Prompts advertised by the selected MCP server. Supports table navigation,
> sorting, column selection, and detail expansion. Gateway to SCR-006 (tool execution).

---

## Wireframe

### Tools Tab (active)

```
╔══════════════════════════════════════════════════════════════════════╗
║  CAPABILITIES — my-server                                            ║
║  [ Tools(12) │ Resources(5) │ Prompts(3) ]                          ║
║  ────────────────────────────────────────────────────────────────── ║
║  NAME                 TYPE    ARGS  DESCRIPTION                      ║
║  ─────────────────── ─────── ───── ─────────────────────────────── ║
║  ▶ read_file          tool      2   Read file contents from path     ║
║    write_file         tool      3   Write text content to file path  ║
║    list_directory     tool      1   List files in directory          ║
║    get_env            tool      1   Get environment variable value   ║
║    execute_command    tool      2   Run shell command                ║
║    http_get           tool      1   Make HTTP GET request            ║
║    http_post          tool      2   Make HTTP POST request           ║
║    read_database      tool      3   Read from SQLite database        ║
║    write_database     tool      3   Write to SQLite database         ║
║    search_web         tool      2   Search the web                   ║
║                                                                      ║
║  12 tools     Sort: name▲    [ e: execute  i: info  /: filter ]     ║
╚══════════════════════════════════════════════════════════════════════╝
```

### Detail Expansion (selected tool)

```
╔══════════════════════════════════════════════════════════════════════╗
║  CAPABILITIES — my-server                                            ║
║  [ Tools(12) │ Resources(5) │ Prompts(3) ]                          ║
║  ────────────────────────────────────────────────────────────────── ║
║  ▶ read_file          tool      2   Read file contents from path     ║
║  ┌────────────────────────────────────────────────────────────────┐ ║
║  │ Tool: read_file                                                │ ║
║  │ Description: Read file contents from the specified path.       │ ║
║  │                                                                │ ║
║  │ Input Schema:                                                  │ ║
║  │   path     : string (required)  — File path to read           │ ║
║  │   encoding : string (optional)  — Encoding [default: utf-8]   │ ║
║  │                                                                │ ║
║  │ Annotations: readOnly=true, destructive=false                 │ ║
║  │                                                                │ ║
║  │  [ e: Execute ]  [ i: Close ]                                  │ ║
║  └────────────────────────────────────────────────────────────────┘ ║
╚══════════════════════════════════════════════════════════════════════╝
```

### Resources Tab

```
╔══════════════════════════════════════════════════════════════════════╗
║  CAPABILITIES — my-server                                            ║
║  [ Tools(12) │ Resources(5) │ Prompts(3) ]                          ║
║  ────────────────────────────────────────────────────────────────── ║
║  URI                              MIME TYPE        DESCRIPTION       ║
║  ──────────────────────────────── ──────────────── ──────────────── ║
║  ▶ file:///workspace/README.md    text/plain       Project readme    ║
║    file:///workspace/config.json  application/json Config file       ║
║    db://main/users                application/json Users table       ║
║    env://PATH                     text/plain       PATH variable     ║
║    http://api/status              application/json API status        ║
║                                                                      ║
║  5 resources    [ i: info  s: subscribe  /: filter ]                ║
╚══════════════════════════════════════════════════════════════════════╝
```

### Prompts Tab

```
╔══════════════════════════════════════════════════════════════════════╗
║  CAPABILITIES — my-server                                            ║
║  [ Tools(12) │ Resources(5) │ Prompts(3) ]                          ║
║  ────────────────────────────────────────────────────────────────── ║
║  NAME                    ARGS  DESCRIPTION                           ║
║  ──────────────────────  ────  ─────────────────────────────────── ║
║  ▶ summarize_file           1   Summarize a file's contents         ║
║    analyze_code             2   Analyze code for quality issues     ║
║    generate_docs            1   Generate documentation              ║
║                                                                      ║
║  3 prompts    [ i: info  e: get  /: filter ]                        ║
╚══════════════════════════════════════════════════════════════════════╝
```

### No Server Selected State

```
╔══════════════════════════════════════════════════════════════════════╗
║  CAPABILITIES                                                        ║
║  ────────────────────────────────────────────────────────────────── ║
║                                                                      ║
║                  No server selected.                                 ║
║                                                                      ║
║                  Select a server in the sidebar                      ║
║                  and press Enter to connect.                         ║
║                                                                      ║
╚══════════════════════════════════════════════════════════════════════╝
```

---

## Element Inventory

| ID | Widget | Type | Position | Description |
|----|--------|------|----------|-------------|
| ELM-001 | Panel | Panel | Full pane | Focused/unfocused border |
| ELM-002 | Title | Text | Top border | "CAPABILITIES — <server-name>" |
| ELM-003 | Tab bar | TabBar | Row 1 | `[ Tools(N) │ Resources(N) │ Prompts(N) ]` |
| ELM-004 | Column headers | Text | Row 3 | Sortable column labels |
| ELM-005 | Table rows | Table | Data area | One row per capability |
| ELM-006 | Selected marker | Glyph | Row prefix | `▶` |
| ELM-007 | Name column | Text | Row | Capability name |
| ELM-008 | Type column | Text | Row | `tool` / `resource` / `prompt` |
| ELM-009 | Args count column | Text | Row | Count of input parameters |
| ELM-010 | Description column | Text | Row | Truncated description |
| ELM-011 | Detail panel | Popup | Over rows | Expanded detail on `i`/`Enter` |
| ELM-012 | Footer bar | Text | Last row | Count + sort indicator + key hints |
| ELM-013 | Filter input | TextInput | Inline (on `/`) | Filter by name/description |
| ELM-014 | Sort indicator | Text | Header | `▲`/`▼` on sorted column |

---

## Column Definitions

### Tools Table

| Column | Width | Sortable | Description |
|--------|-------|---------|-------------|
| Name | 22 cols | ✓ default | Tool name |
| Type | 7 cols | — | Always `tool` (shown for consistency) |
| Args | 5 cols | ✓ | Input parameter count |
| Description | remaining | — | Truncated description |

### Resources Table

| Column | Width | Sortable | Description |
|--------|-------|---------|-------------|
| URI | 34 cols | ✓ default | Resource URI |
| MIME Type | 18 cols | ✓ | MIME type string |
| Description | remaining | — | Truncated description |

### Prompts Table

| Column | Width | Sortable | Description |
|--------|-------|---------|-------------|
| Name | 24 cols | ✓ default | Prompt name |
| Args | 5 cols | ✓ | Argument count |
| Description | remaining | — | Truncated description |

---

## State Definitions

| State Variable | Type | Description |
|----------------|------|-------------|
| `active_tab` | Enum: Tools\|Resources\|Prompts | Current tab |
| `selected_row` | usize | Highlighted row index |
| `scroll_offset` | usize | Vertical scroll position |
| `detail_open` | bool | Detail expansion visible |
| `detail_item_id` | Option<CapabilityId> | Currently expanded item |
| `sort_column` | Option<ColumnId> | Active sort column |
| `sort_direction` | Enum: Asc\|Desc | Sort direction |
| `filter_text` | String | Active filter |
| `filter_active` | bool | Filter input has focus |
| `tools` | Vec<Tool> | Loaded tools |
| `resources` | Vec<Resource> | Loaded resources |
| `prompts` | Vec<Prompt> | Loaded prompts |
| `loading` | bool | Awaiting list response |

---

## Detail Panel Content

When `i` is pressed or `Enter` on a tool row, detail panel expands in-place:

**For Tools:**
- Tool name (bold)
- Description (full, word-wrapped)
- Input schema: each parameter on own line with name, type, required/optional, description
- Annotations: `readOnly`, `destructive`, `idempotent` if present
- Action buttons: `[ e: Execute ]  [ i: Close ]`

**For Resources:**
- URI (full, no truncation)
- MIME type
- Description (full)
- Subscribe status if subscriptions supported
- Action buttons: `[ r: Read ]  [ s: Subscribe ]  [ i: Close ]`

**For Prompts:**
- Name (bold)
- Description (full)
- Arguments: each arg with name, type, required/optional, description
- Action buttons: `[ e: Get Prompt ]  [ i: Close ]`

---

## Sorting

Default sort: name ascending (`▲`).
Click column header (mouse) or `s` key to cycle sort on focused column.
Sort indicator: `▲` = ascending, `▼` = descending, no indicator = unsorted.

---

## Keyboard Shortcuts

| Key | Action | Notes |
|-----|--------|-------|
| `j` / `↓` | Next row | |
| `k` / `↑` | Previous row | |
| `g` / `Home` | First row | |
| `G` / `End` | Last row | |
| `[` | Previous tab | Tools → (wraps from Tools) |
| `]` | Next tab | → Resources → Prompts → (wraps) |
| `1` | Switch to Tools tab | |
| `2` | Switch to Resources tab | |
| `3` | Switch to Prompts tab | |
| `Enter` / `i` | Toggle detail expansion | |
| `e` | Execute selected tool / Get prompt | Opens SCR-006 |
| `r` | Read selected resource | Opens resource content popup |
| `s` | Subscribe to resource | Resources tab only |
| `S` | Sort by focused column | Cycles asc/desc/none |
| `/` | Open filter | Case-insensitive name+desc match |
| `Esc` | Close detail / clear filter | |
| `Tab` | Move focus to Traffic Inspector | |
| `Shift+Tab` | Move focus to Server Sidebar | |

---

## Loading States

When a server is connected and capabilities are being fetched:

```
║  CAPABILITIES — my-server                                            ║
║  [ Tools(?) │ Resources(?) │ Prompts(?) ]                           ║
║  ────────────────────────────────────────────────────────────────── ║
║                                                                      ║
║              Loading capabilities…  ◌                               ║
║                                                                      ║
```

Spinner: `◌` rotates through `◌ ◎ ●` at 4Hz (or static in 16-color mode).

---

## Accessibility Notes

- **Tab count:** Badge `(N)` on each tab shows count — avoids needing to switch tabs to see if content exists
- **No color-only:** Type column uses text label always; detail panel lists schema textually
- **Keyboard-only:** All tabs reachable via `[`/`]`/`1`/`2`/`3`; no mouse required
- **Focus order within pane:** Tab bar → Table rows → Detail panel (when open)
- **Detail panel:** Opens inline (not modal) to preserve context; `Esc` closes
- **Sort state:** Indicated by glyph in column header (`▲`/`▼`) not color alone

---

## BC Traceability

| BC ID | How This Screen Satisfies It |
|-------|------------------------------|
| BC-3.08.004 | Tab-based capability explorer with tools/resources/prompts |
| BC-3.07.001 | j/k navigation, tab switching via `[`/`]` |
| BC-3.08.005 | No color-only; type column text, sort by glyph |
| BC-2.05.001 | Tool list display (paginated list from tools/list) |
| BC-2.05.002 | Resource list display and subscribe action |
| BC-2.05.003 | Prompt list display and get action |
