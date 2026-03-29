---
document_type: ux-spec-screen
screen_id: SCR-010
screen_name: Server Comparison View
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-03-29T14:15:00
phase: 1c
complexity: complex
traces_to: UX-INDEX.md
prd_requirements: [BC-10.23.001, BC-10.24.001, BC-10.25.001]
priority: P2
---

# Screen: Server Comparison View (SCR-010)

> Full-width view for side-by-side comparison of two MCP servers: capability
> differences, tool schema diffs, and optional behavioral delta testing.
> Typically launched from `forge-mcp diff <server-a> <server-b>` CLI or
> from the `[ Compare ]` tab (P2 feature gate).

---

## Wireframe

### Capability Overview Diff

```
╔══════════════════════════════════════════════════════════════════════════╗
║  COMPARE — my-server  vs  staging                                        ║
║  ──────────────────────────────────────────────────────────────────────  ║
║  CAPABILITY       MY-SERVER           STAGING             DIFF          ║
║  ──────────────── ─────────────────── ─────────────────── ─────────────  ║
║  tools/list       ✓ 12 tools          ✓ 10 tools          ⚠ 2 added    ║
║  resources/list   ✓ 5 resources       ✓ 5 resources       ✓ same        ║
║  prompts/list     ✓ 3 prompts         ✗ not supported     ⚠ missing     ║
║  sampling         ✓ supported         ✓ supported         ✓ same        ║
║  elicitation      ✓ form + url        ✗ not supported     ⚠ missing     ║
║  logging          ✓ supported         ✓ supported         ✓ same        ║
║  completions      ✓ supported         ✗ not supported     ⚠ missing     ║
║                                                                          ║
║  Protocol version  2025-11-25         2024-11-05          ⚠ mismatch   ║
║                                                                          ║
║  t:tool-diff  c:capability-diff  b:behavior-test  Tab:switch-column     ║
╚══════════════════════════════════════════════════════════════════════════╝
```

### Tool Schema Diff (side-by-side)

```
╔══════════════════════════════════════════════════════════════════════════╗
║  TOOL SCHEMA DIFF — read_file                                            ║
║  ──────────────────────────────────────────────────────────────────────  ║
║  MY-SERVER                         │  STAGING                            ║
║  ─────────────────────────────── ─ │ ─ ─────────────────────────────── ║
║    "name": "read_file",             │   "name": "read_file",             ║
║    "description": "Read file…",     │   "description": "Read file…",     ║
║    "inputSchema": {                 │   "inputSchema": {                 ║
║      "type": "object",              │     "type": "object",              ║
║      "properties": {                │     "properties": {                ║
║  >>>   "path": {                    │ >>>   "path": {                    ║
║  >>>     "type": "string",          │ >>>     "type": "string",          ║
║  >>>     "description": "File       │ >>>     "description": "Path to    ║  ← description differs
║  >>>      path to read"             │ >>>      the file"                 ║
║  >>>   },                           │ >>>   },                           ║
║  +     "encoding": {                │                                    ║  ← only in my-server
║  +       "type": "string"           │                                    ║
║  +     }                            │                                    ║
║      }                              │     }                              ║
║    }                                │   }                                ║
║                                                                          ║
║  [ Esc: Back ]                                                           ║
╚══════════════════════════════════════════════════════════════════════════╝
```

### Tools List Diff

```
╔══════════════════════════════════════════════════════════════════════════╗
║  TOOL DIFF — my-server vs staging                                        ║
║  ──────────────────────────────────────────────────────────────────────  ║
║  TOOL NAME            MY-SERVER       STAGING         DIFF              ║
║  ──────────────────── ─────────────── ─────────────── ─────────────────  ║
║  read_file            ✓ present       ✓ present       ⚠ schema diff    ║
║  write_file           ✓ present       ✓ present       ✓ identical      ║
║  list_directory       ✓ present       ✓ present       ✓ identical      ║
║  get_env              ✓ present       ✓ present       ✓ identical      ║
║  execute_command      ✓ present       ✓ present       ✓ identical      ║
║  http_get             ✓ present       ✓ present       ✓ identical      ║
║  http_post            ✓ present       ✓ present       ✓ identical      ║
║  read_database        ✓ present       ✓ present       ✓ identical      ║
║  write_database       ✓ present       ✓ present       ✓ identical      ║
║  search_web           ✓ present       ✓ present       ✓ identical      ║
║  analyze_code         ✓ present       ✗ missing       ⚠ missing        ║
║  generate_report      ✓ present       ✗ missing       ⚠ missing        ║
║                                                                          ║
║  12 total  10 shared  2 only in my-server  0 only in staging            ║
║  i:schema-diff  Esc:back                                                 ║
╚══════════════════════════════════════════════════════════════════════════╝
```

---

## Element Inventory

| ID | Widget | Type | Position | Description |
|----|--------|------|----------|-------------|
| ELM-001 | Panel | Panel | Full content area | |
| ELM-002 | Title | Text | Top border | "COMPARE — <server-a> vs <server-b>" |
| ELM-003 | View tabs | TabBar | Row 1 | Overview / Tools / Schema / Behavior |
| ELM-004 | Comparison table | Table | Main area | 4-column: name, A, B, diff |
| ELM-005 | Diff indicator | Badge | Diff col | `✓ same` / `⚠ diff` / `⚠ missing` |
| ELM-006 | Split diff view | SplitPane | On detail | Side-by-side JSON schema diff |
| ELM-007 | Diff line marker | Text | Diff lines | `>>>` changed, `+` added, `-` removed |
| ELM-008 | Summary | Text | Below table | Count breakdown |
| ELM-009 | Key hint bar | Text | Last row | Context shortcuts |

---

## Diff Markers

| Marker | Meaning | Color token |
|--------|---------|-------------|
| `>>>` | Changed value (same key, different value) | `status.warning` |
| `+` | Present in server A only | `status.connected` (green) |
| `-` | Present in server B only | `status.error` (red) |
| ` ` (none) | Identical in both | normal |

ASCII mode: same markers (`>>>`, `+`, `-`) work without color changes.

---

## State Definitions

| State Variable | Type | Description |
|----------------|------|-------------|
| `server_a` | ServerId | First comparison server |
| `server_b` | ServerId | Second comparison server |
| `active_view` | Enum: Overview\|Tools\|Schema\|Behavior | Current view |
| `selected_tool` | Option<String> | Tool selected for schema diff |
| `diff_result` | CapabilityDiff | Computed diff |
| `schema_diffs` | HashMap<String, SchemaDiff> | Per-tool schema diffs |

---

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `j` / `↓` | Next row |
| `k` / `↑` | Previous row |
| `Tab` | Switch focus between server columns (in split view) |
| `t` | Switch to tool diff view |
| `c` | Switch to capability overview |
| `b` | Run behavioral delta test (P2) |
| `Enter` / `i` | Show schema diff for selected tool |
| `Esc` | Close detail / return to overview |
| `E` | Export comparison report |

---

## Accessibility Notes

- **Diff indicators:** `✓ same` / `⚠ diff` text + glyphs; not color-only
- **Diff markers:** `>>>`, `+`, `-` text prefixes work without color
- **Column focus:** Tab toggles which column scrolls in split view
- **Summary line:** Always shows counts

---

## BC Traceability

| BC ID | How This Screen Satisfies It |
|-------|------------------------------|
| BC-10.23.001 | Tool schema diff view (side-by-side schema comparison) |
| BC-10.24.001 | Capability overview diff (supported/missing per server) |
| BC-10.25.001 | Behavioral delta test trigger (`b` key) |
| BC-3.08.005 | Diff indicators use glyphs + text + color |
