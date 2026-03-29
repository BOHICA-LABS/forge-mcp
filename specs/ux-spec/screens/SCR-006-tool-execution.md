---
document_type: ux-spec-screen
screen_id: SCR-006
screen_name: Tool Execution Dialog
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-03-29T14:15:00
phase: 1c
complexity: complex
traces_to: UX-INDEX.md
prd_requirements: [BC-2.05.001, BC-2.05.009, BC-2.05.010, BC-3.07.001]
---

# Screen: Tool Execution Dialog (SCR-006)

> Modal dialog overlay for invoking an MCP tool. Input fields are driven by
> the tool's JSON Schema. Supports argument entry, execution, progress display,
> result rendering with syntax highlighting, and error display. Opened from
> SCR-003 Capability Browser via `e` key.

---

## Wireframe

### Argument Entry State

```
╔══════════════════════════════════════════════════════════════════════╗
║  EXECUTE TOOL — read_file @ my-server                                ║
║  ──────────────────────────────────────────────────────────────────  ║
║  Read file contents from the specified path.                         ║
║                                                                      ║
║  Arguments:                                                          ║
║                                                                      ║
║  path *         (string, required)                                   ║
║  ┌────────────────────────────────────────────────────────────────┐  ║
║  │ /workspace/README.md                                           │  ║
║  └────────────────────────────────────────────────────────────────┘  ║
║                                                                      ║
║  encoding       (string, optional)  [default: utf-8]                ║
║  ┌────────────────────────────────────────────────────────────────┐  ║
║  │ utf-8                                                          │  ║
║  └────────────────────────────────────────────────────────────────┘  ║
║                                                                      ║
║  ──────────────────────────────────────────────────────────────────  ║
║           [ Execute (Ctrl+Enter) ]      [ Cancel (Esc) ]            ║
╚══════════════════════════════════════════════════════════════════════╝
```

### Executing State (In-Progress)

```
╔══════════════════════════════════════════════════════════════════════╗
║  EXECUTE TOOL — read_file @ my-server                                ║
║  ──────────────────────────────────────────────────────────────────  ║
║  Read file contents from the specified path.                         ║
║                                                                      ║
║  Arguments:                                                          ║
║  path:     /workspace/README.md                                      ║
║  encoding: utf-8                                                     ║
║                                                                      ║
║  ──────────────────────────────────────────────────────────────────  ║
║  Executing…  ◌  67ms                                                 ║
║  ████████████████████░░░░░░░░░░░░░░░░░░░░  (waiting for response)   ║
║                                                                      ║
║                          [ Cancel (Esc) ]                            ║
╚══════════════════════════════════════════════════════════════════════╝
```

### Success Result State

```
╔══════════════════════════════════════════════════════════════════════╗
║  RESULT — read_file @ my-server       ✓ OK  │  134ms               ║
║  ──────────────────────────────────────────────────────────────────  ║
║  Content: (text/plain)                                               ║
║                                                                      ║
║   1  # Forge MCP                                                     ║
║   2                                                                  ║
║   3  A unified MCP inspection, debugging, monitoring, and security  ║
║   4  auditing tool for AI platform teams.                            ║
║   5                                                                  ║
║   6  ## Installation                                                 ║
║   7                                                                  ║
║   8  ```bash                                                         ║
║   9  cargo install forge-mcp                                         ║
║  10  ```                                                             ║
║  ──────────────────────────────────────────────────────────────────  ║
║  1 content block  │  2.3KB                                           ║
║  [ y: Yank ]  [ r: Re-run ]  [ e: Edit args ]  [ Esc: Close ]       ║
╚══════════════════════════════════════════════════════════════════════╝
```

### Tool Error State

```
╔══════════════════════════════════════════════════════════════════════╗
║  RESULT — read_file @ my-server       ✗ TOOL ERROR  │  89ms        ║
║  ──────────────────────────────────────────────────────────────────  ║
║  ✗ Tool returned an error:                                           ║
║                                                                      ║
║  Error: No such file or directory (os error 2)                       ║
║  Path:  /workspace/README.md                                         ║
║                                                                      ║
║  ──────────────────────────────────────────────────────────────────  ║
║  isError: true   │  Content type: text                               ║
║  [ r: Re-run ]  [ e: Edit args ]  [ Esc: Close ]                     ║
╚══════════════════════════════════════════════════════════════════════╝
```

### Protocol Error State

```
╔══════════════════════════════════════════════════════════════════════╗
║  RESULT — read_file @ my-server       ✗ PROTOCOL ERROR  │  31ms    ║
║  ──────────────────────────────────────────────────────────────────  ║
║  ✗ Protocol error:                                                   ║
║                                                                      ║
║  Code:    -32602                                                     ║
║  Message: Invalid params: missing required field 'path'              ║
║                                                                      ║
║  ──────────────────────────────────────────────────────────────────  ║
║  JSON-RPC error code: -32602 (Invalid params)                        ║
║  [ e: Edit args ]  [ Esc: Close ]                                    ║
╚══════════════════════════════════════════════════════════════════════╝
```

### JSON Object Argument (multi-line editor)

```
╔══════════════════════════════════════════════════════════════════════╗
║  EXECUTE TOOL — configure @ my-server                                ║
║  ──────────────────────────────────────────────────────────────────  ║
║  Configure server options.                                           ║
║                                                                      ║
║  options *  (object, required)                                       ║
║  ┌────────────────────────────────────────────────────────────────┐  ║
║  │ {                                                              │  ║
║  │   "timeout": 30,                                               │  ║
║  │   "retries": 3                                                 │  ║
║  │ }                                                              │  ║
║  └────────────────────────────────────────────────────────────────┘  ║
║  ⚠ JSON must be valid                                                 ║
║                                                                      ║
║           [ Execute (Ctrl+Enter) ]      [ Cancel (Esc) ]            ║
╚══════════════════════════════════════════════════════════════════════╝
```

---

## Element Inventory

| ID | Widget | Type | Position | Description |
|----|--------|------|----------|-------------|
| ELM-001 | Modal border | Panel | Full overlay | Double-line modal border |
| ELM-002 | Title | Text | Top border | "EXECUTE TOOL — <tool-name> @ <server>" |
| ELM-003 | Tool description | Text | Row 2 | Short description from schema |
| ELM-004 | Arguments section | Section | Main area | Label + input field per parameter |
| ELM-005 | Arg label | Text | Above each input | `<name> *` (required) or `<name>` (optional) + type |
| ELM-006 | Arg description | Text | Right of label | Inline description if space allows |
| ELM-007 | Arg default | Text | Right of label | `[default: value]` if applicable |
| ELM-008 | Text input | TextInput | Below label | Single-line for scalar types |
| ELM-009 | Multi-line editor | TextArea | Below label | Multi-line for object/array types |
| ELM-010 | Validation hint | Text | Below input | `⚠ <message>` if invalid |
| ELM-011 | Execute button | Button | Bottom | `[ Execute (Ctrl+Enter) ]` |
| ELM-012 | Cancel button | Button | Bottom | `[ Cancel (Esc) ]` |
| ELM-013 | Progress bar | ProgressBar | Executing state | Indeterminate pulse |
| ELM-014 | Elapsed timer | Text | Executing state | `NNNms` updating live |
| ELM-015 | Result title | Text | Result state | "RESULT — <tool> @ <server>  ✓/✗ STATUS │ NNms" |
| ELM-016 | Content type | Text | Result state | MIME type of returned content |
| ELM-017 | Result body | ScrollableText | Result state | Tool output, line-numbered |
| ELM-018 | Error display | Text | Error state | Error message + details |
| ELM-019 | Result footer | Text | Result state | Block count, size, action hints |
| ELM-020 | Scrollbar | Scrollbar | Result pane | Right edge |

---

## Argument Input Field Types

Input fields are rendered based on JSON Schema type:

| Schema Type | Widget | Notes |
|-------------|--------|-------|
| `string` | TextInput (1 line) | Auto-focus first required field |
| `number` / `integer` | TextInput (1 line) | Validated as numeric |
| `boolean` | Toggle: `[true]` / `[false]` | Space/Enter toggles |
| `object` | TextArea (multi-line) | JSON editor; validated on blur |
| `array` | TextArea (multi-line) | JSON array editor |
| `enum` | Dropdown list | Options from `enum:` schema field |

Required fields marked with `*` suffix on label.
Optional fields with defaults: `[default: value]` shown inline.

---

## Validation

- **Required field empty:** Cannot execute; hint `⚠ Required field` shown
- **Type mismatch:** `⚠ Expected <type>` shown below field
- **JSON syntax error:** `⚠ Invalid JSON: <parse error>` shown below textarea
- **Schema validation:** Performed before sending; all errors shown before execute
- Execute button is disabled (dimmed) when any required field is invalid

---

## State Definitions

| State Variable | Type | Description |
|----------------|------|-------------|
| `tool_def` | ToolDefinition | Schema driving the form |
| `arg_values` | HashMap<String, Value> | Current field values |
| `validation_errors` | HashMap<String, String> | Field → error message |
| `executing` | bool | Request in flight |
| `elapsed_ms` | u64 | Elapsed time since execute |
| `result` | Option<ToolResult> | Completed result |
| `error` | Option<ToolError> | Protocol or tool error |
| `result_scroll` | usize | Scroll offset in result view |
| `cancellation_token` | Option<CancelToken> | For progress cancellation |

---

## Progress Cancellation

Per BC-2.05.009:
- While executing: `Esc` or `Cancel` button sends `notifications/cancelled` to server
- Progress bar shows indeterminate pulse (scrolling `████`) 
- After cancel: result area shows "Cancelled" with elapsed time

---

## Tool Error vs. Protocol Error Display

Per BC-2.05.010:

| Error Type | Indicator | Body Content |
|-----------|-----------|-------------|
| Tool error (`isError: true` in result) | `✗ TOOL ERROR` in title | Content array from result |
| Protocol error (JSON-RPC error object) | `✗ PROTOCOL ERROR` in title | `code:` + `message:` from error object |

Tool errors indicate the tool ran but reported a domain error. Protocol errors indicate the server rejected the request itself.

---

## Multi-Content Block Results

Some tools return multiple content blocks. Display:
```
Content: (3 blocks)

[Block 1 — text/plain]
...text content...

[Block 2 — application/json]
{ ... }  (syntax highlighted)

[Block 3 — image/png]
[Binary image: 4.2KB — save with y]
```

---

## Keyboard Shortcuts

| Key | Action | Notes |
|-----|--------|-------|
| `Tab` | Next input field | Cycles through args → Execute → Cancel |
| `Shift+Tab` | Previous field | |
| `Ctrl+Enter` | Execute tool | When form is valid |
| `Esc` | Cancel (input mode) / Close (result mode) / Cancel execution | |
| `r` | Re-run with same args | From result state |
| `e` | Edit args (return to form) | From result state |
| `y` | Yank result to clipboard | From result state |
| `j` / `↓` | Scroll result down | From result state |
| `k` / `↑` | Scroll result up | From result state |

---

## Accessibility Notes

- **Focus trap:** Tab cycling stays within dialog; `Esc` closes
- **Focus auto-placement:** Opens with focus on first required field
- **Validation:** All errors shown as text; not color-only
- **Error distinction:** Tool error vs. protocol error clearly labeled in title and body
- **Scrollable result:** Long results scrollable with j/k; scrollbar shown
- **Required marker:** `*` glyph + "required" text in field label
- **Execution state:** Text "Executing…" + spinner glyph + elapsed timer; not animation-only

---

## BC Traceability

| BC ID | How This Screen Satisfies It |
|-------|------------------------------|
| BC-2.05.001 | Tool invocation via tools/call |
| BC-2.05.009 | Progress cancellation (Esc while executing) |
| BC-2.05.010 | Tool error vs. protocol error distinction |
| BC-3.07.001 | Keyboard-only navigation of form fields |
| BC-3.08.001 | Syntax-highlighted JSON result display |
| BC-3.08.005 | Error states use glyph + text, not color-only |
