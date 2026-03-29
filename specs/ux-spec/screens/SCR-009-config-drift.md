---
document_type: ux-spec-screen
screen_id: SCR-009
screen_name: Config Drift View
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-03-29T14:15:00
phase: 1c
complexity: complex
traces_to: UX-INDEX.md
prd_requirements: [BC-9.21.001, BC-9.21.002, BC-9.22.001]
priority: P1
---

# Screen: Config Drift View (SCR-009)

> Full content-area tab showing configuration differences across editor MCP
> configs (Claude Desktop, Cursor, VS Code, Windsurf). Side-by-side diff view
> with highlighted changes. Presents actionable reconciliation guidance.

---

## Wireframe

### Summary / Diff Overview

```
╔══════════════════════════════════════════════════════════════════════════╗
║  CONFIG DRIFT — 3 editors  │  ⚠ 4 diffs detected                        ║
║  ──────────────────────────────────────────────────────────────────────  ║
║  Sources:  [● claude]  [● cursor]  [● vscode]  [○ windsurf (not found)] ║
║                                                                          ║
║  SERVER           CLAUDE         CURSOR         VSCODE         DRIFT    ║
║  ──────────────── ────────────── ────────────── ────────────── ──────── ║
║  filesystem       ✓ present      ✓ present      ✗ missing      ⚠ DRIFT  ║
║  web-search       ✓ present      ✗ missing      ✓ present      ⚠ DRIFT  ║
║  database         ✓ v1.2.0       ✓ v1.1.0       ✓ v1.2.0      ⚠ DRIFT  ║  ← version diff
║  lsp-bridge       ✓ present      ✓ present      ✓ present      ✓ OK     ║
║  git-tools        ✓ present      ✓ present      ✓ present      ✓ OK     ║
║                                                                          ║
║  4 differences found   2 missing   1 version mismatch   1 arg diff      ║
║  s:source-select  i:detail  r:reconcile  E:export                       ║
╚══════════════════════════════════════════════════════════════════════════╝
```

### Side-by-Side Detail (server selected)

```
╔══════════════════════════════════════════════════════════════════════════╗
║  CONFIG DETAIL — database server                                         ║
║  ──────────────────────────────────────────────────────────────────────  ║
║  CLAUDE DESKTOP                    │  CURSOR                            ║
║  ──────────────────────────────── │ ─────────────────────────────────  ║
║    "database": {                   │   "database": {                    ║
║      "command": "npx",             │     "command": "npx",              ║
║      "args": [                     │     "args": [                      ║
║        "-y",                       │       "-y",                        ║
║  >>>   "@company/db-mcp@1.2.0",    │  >>>  "@company/db-mcp@1.1.0",    ║  ← diff line
║        "--port", "5432"            │       "--port", "5432"             ║
║      ],                            │     ],                             ║
║      "env": {                      │     "env": {                       ║
║        "DB_HOST": "localhost"      │       "DB_HOST": "localhost"       ║
║      }                             │     }                              ║
║    }                               │   }                                ║
║                                                                          ║
║  ⚠ Version mismatch: claude has 1.2.0, cursor has 1.1.0                 ║
║  Recommendation: Update cursor config to use @1.2.0                      ║
║  [ Esc: Back ]                                                           ║
╚══════════════════════════════════════════════════════════════════════════╝
```

### Reconciliation Guidance

```
╔══════════════════════════════════════════════════════════════════════════╗
║  RECONCILIATION GUIDE — 4 issues                                         ║
║  ──────────────────────────────────────────────────────────────────────  ║
║  Issue 1 of 4: filesystem — missing in vscode                            ║
║  ──────────────────────────────────────────────────────────────────────  ║
║  Action: Add filesystem server to VS Code config                         ║
║                                                                          ║
║  Add to ~/.config/Code/User/settings.json:                               ║
║  ┌────────────────────────────────────────────────────────────────────┐  ║
║  │   "mcp": {                                                         │  ║
║  │     "servers": {                                                   │  ║
║  │       "filesystem": {                                              │  ║
║  │         "command": "npx",                                         │  ║
║  │         "args": ["-y", "@company/filesystem-mcp"]                 │  ║
║  │       }                                                            │  ║
║  │     }                                                              │  ║
║  │   }                                                                │  ║
║  └────────────────────────────────────────────────────────────────────┘  ║
║  [y: Copy snippet]  [n: Next issue]  [p: Prev issue]  [Esc: Back]        ║
╚══════════════════════════════════════════════════════════════════════════╝
```

---

## Element Inventory

| ID | Widget | Type | Position | Description |
|----|--------|------|----------|-------------|
| ELM-001 | Panel | Panel | Full content area | Pane border |
| ELM-002 | Title | Text | Top border | "CONFIG DRIFT — N editors │ ⚠ N diffs" |
| ELM-003 | Source badges | Badges | Row 2 | Per-editor presence indicator |
| ELM-004 | Server table | Table | Main area | Server name + per-editor status column |
| ELM-005 | Server name | Text | Col 1 | Server name |
| ELM-006 | Editor columns | Text | Col 2-5 | Per-editor status (present/missing/version) |
| ELM-007 | Drift indicator | Badge | Last col | `⚠ DRIFT` or `✓ OK` |
| ELM-008 | Summary line | Text | Below table | Count breakdown |
| ELM-009 | Detail view | SplitPane | On `i` | Side-by-side JSON diff |
| ELM-010 | Reconcile view | FullPane | On `r` | Step-through reconciliation guide |
| ELM-011 | Diff highlight | Text | Detail view | `>>>` prefix on changed lines |
| ELM-012 | Recommendation | Text | Detail view | Actionable guidance below diff |
| ELM-013 | Code snippet | Block | Reconcile view | Config JSON to add/change |
| ELM-014 | Key hint bar | Text | Last row | Context shortcuts |

---

## State Definitions

| State Variable | Type | Description |
|----------------|------|-------------|
| `sources` | Vec<ConfigSource> | Discovered editor configs |
| `servers` | Vec<ServerDriftEntry> | Per-server cross-editor status |
| `selected_server` | usize | Highlighted row |
| `detail_open` | bool | Detail view visible |
| `reconcile_open` | bool | Reconciliation guide visible |
| `reconcile_issue_idx` | usize | Current issue in guide |
| `filter_drifted` | bool | Show drifted only |

---

## Diff Highlighting

Changed lines are marked with `>>>` prefix (3 chars).
- `>>>` lines: `syntax.string` color (yellow/amber) + bold
- Unchanged lines: normal text
- Context lines: 3 lines above/below each change shown

---

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `j` / `↓` | Next server row |
| `k` / `↑` | Previous server row |
| `Enter` / `i` | Show detail diff |
| `r` | Open reconciliation guide |
| `n` | Next reconciliation issue |
| `p` | Previous reconciliation issue |
| `y` | Copy reconciliation snippet |
| `s` | Source selector (toggle which editors shown) |
| `f` | Filter: show drifted only |
| `E` | Export drift report |
| `Esc` | Close detail / close reconcile |
| `Tab` | Move focus to next pane |

---

## Accessibility Notes

- **Drift status:** `✓ OK` / `⚠ DRIFT` glyphs + text; not color-only
- **Editor presence:** `✓ present` / `✗ missing` in each cell
- **Diff lines:** `>>>` prefix ensures visibility without color
- **Reconciliation:** Code snippets shown as text; `y` copies to clipboard
- **Focus order:** Source badges (display only) → Server table → Key hint bar

---

## BC Traceability

| BC ID | How This Screen Satisfies It |
|-------|------------------------------|
| BC-9.21.001 | Cross-editor config comparison table |
| BC-9.21.002 | Drift report with actionable detail (diff view + recommendations) |
| BC-9.22.001 | Reconciliation guide with step-through and copyable snippets (P2) |
| BC-3.08.005 | Drift indicators use glyphs + text + color |
