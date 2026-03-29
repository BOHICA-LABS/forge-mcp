---
document_type: ux-spec-screen
screen_id: SCR-004
screen_name: Traffic Inspector
version: "1.1"
status: draft
producer: ux-designer
timestamp: 2026-03-29T14:57:00
phase: 1c
complexity: complex
traces_to: UX-INDEX.md
prd_requirements: [BC-4.09.001, BC-4.09.002, BC-4.09.003, BC-4.10.001, BC-4.10.002, BC-3.08.001]
changelog:
  - version: "1.1"
    date: 2026-03-29
    change: "ADV-P2-004 — Added Replay Target Dialog (ELM-019/ELM-020), Replay Confirmation Dialog (ELM-021), and Replay Progress indicator (ELM-022) to enforce BC-4.10.003 PRE-002/PRE-004 (DI-007). Updated keyboard shortcuts, state definitions, and BC traceability accordingly."
---

# Screen: Traffic Inspector (SCR-004)

> Bottom section of the content area (or full-screen via `2` tab). Shows a
> scrollable, real-time log of JSON-RPC messages captured between client and
> MCP server. Syntax-highlighted JSON detail view. Filter bar. Request-response
> correlation. The "Wireshark for MCP" signature feature.

---

## Wireframe

### Message List View

```
╔══════════════════════════════════════════════════════════════════════════╗
║  TRAFFIC INSPECTOR — my-server    [● REC]  [47 msgs]  [Filter: off]     ║
║  ──────────────────────────────────────────────────────────────────────  ║
║  TIME        ID     DIR  METHOD                  STATUS   MS    SIZE     ║
║  ─────────── ────── ───  ────────────────────── ──────── ────  ──────   ║
║  14:02:31.1  001    ▶    initialize               OK       23   412B     ║
║  14:02:31.2  001    ◀    initialize (response)    OK        -   892B     ║
║  14:02:32.0  002    ▶    tools/list               OK        8   143B     ║
║  14:02:32.0  002    ◀    tools/list (response)    OK        -  3.2KB     ║
║  14:02:45.3  ▶ 003  ▶    tools/call               OK       67   298B     ║  ← selected (request)
║  14:02:45.4  ▶ 003  ◀    tools/call (response)    OK        -  1.1KB     ║  ← correlated pair
║  14:03:01.0  004    ▶    resources/list           OK        5   118B     ║
║  14:03:01.0  004    ◀    resources/list (response)OK        -   876B     ║
║  14:03:14.2  005    ▶    tools/call              ERR      234   298B     ║  ← error (red)
║                                                                          ║
║  ──────────────────────────────────────────────────────────────────────  ║
║  c:clear  f:filter  x:expand  R:replay  /: search                       ║
╚══════════════════════════════════════════════════════════════════════════╝
```

### Filter Bar Active

```
╔══════════════════════════════════════════════════════════════════════════╗
║  TRAFFIC INSPECTOR — my-server    [● REC]  [12 / 47 msgs]               ║
║  ┌ Filter ──────────────────────────────────────────────────────────── ┐ ║
║  │ Method: [tools/call      ] Dir: [both▼] Status: [all▼] Time: [__] │ ║
║  └───────────────────────────────────────────────────────────────────── ┘ ║
║  TIME        ID     DIR  METHOD                  STATUS   MS    SIZE     ║
║  ─────────── ────── ───  ────────────────────── ──────── ────  ──────   ║
║  14:02:45.3  ▶ 003  ▶    tools/call               OK       67   298B     ║
║  14:02:45.4  ▶ 003  ◀    tools/call (response)    OK        -  1.1KB     ║
║  14:03:14.2  005    ▶    tools/call              ERR      234   298B     ║
║                                                                          ║
║  3 of 47 messages                                                        ║
╚══════════════════════════════════════════════════════════════════════════╝
```

### Full-Screen Message Detail (expanded)

```
╔══════════════════════════════════════════════════════════════════════════╗
║  MESSAGE DETAIL — ID:003  tools/call  ▶ REQUEST                         ║
║  ──────────────────────────────────────────────────────────────────────  ║
║   1  {                                                                   ║
║   2    "jsonrpc": "2.0",                                                 ║
║   3    "id": 3,                                                          ║
║   4    "method": "tools/call",                                           ║
║   5    "params": {                                                        ║
║   6      "name": "read_file",                                            ║
║   7      "arguments": {                                                  ║
║   8        "path": "/workspace/README.md",                               ║
║   9        "encoding": "utf-8"                                           ║
║  10      }                                                               ║
║  11    }                                                                 ║
║  12  }                                                                   ║
║  ──────────────────────────────────────────────────────────────────────  ║
║  Size: 298B   Sent: 14:02:45.300ms                                       ║
║  [ ↔ Show Response Pair ]   [ R: Replay ]   [ y: Yank ]   [ Esc: Back ] ║
╚══════════════════════════════════════════════════════════════════════════╝
```

### Split Request/Response View

```
╔══════════════════════════════════════════════════════════════════════════╗
║  MESSAGE PAIR — ID:003  tools/call                                       ║
║  ╔════════════════════════════╗ ╔═══════════════════════════════════════╗ ║
║  ║ REQUEST (▶)                ║ ║ RESPONSE (◀)                          ║ ║
║  ║  {                         ║ ║  {                                    ║ ║
║  ║    "jsonrpc": "2.0",       ║ ║    "jsonrpc": "2.0",                 ║ ║
║  ║    "id": 3,                ║ ║    "id": 3,                          ║ ║
║  ║    "method": "tools/call", ║ ║    "result": {                       ║ ║
║  ║    "params": {             ║ ║      "content": [                    ║ ║
║  ║      "name": "read_file",  ║ ║        {                             ║ ║
║  ║      "arguments": { … }   ║ ║          "type": "text",             ║ ║
║  ║    }                       ║ ║          "text": "# Forge MCP…"     ║ ║
║  ║  }                         ║ ║        }                             ║ ║
║  ║                            ║ ║      ]                               ║ ║
║  ╚════════════════════════════╝ ╚═══════════════════════════════════════╝ ║
║  RTT: 134ms     [ Esc: Back ]  [ R: Replay ]  [ y: Yank both ]          ║
╚══════════════════════════════════════════════════════════════════════════╝
```

### Replay Target Dialog (ELM-019) — triggered by `R` from any detail/split/list view

```
╔══════════════════════════════════════════════════════════════════════════╗
║  TRAFFIC INSPECTOR — my-server    [● REC]  [47 msgs]                    ║
║  ┌ Replay Target ─────────────────────────────────────────────────────┐  ║
║  │ Select the server to replay this message to:                       │  ║
║  │                                                                    │  ║
║  │  Replaying:  tools/call  (ID:003)                                  │  ║
║  │                                                                    │  ║
║  │  ▶ staging-server   (localhost:9001)                               │  ║  ← highlighted
║  │    my-server        (localhost:8080)  [original]                   │  ║
║  │    prod-server      (api.example.com:443)                          │  ║
║  │                                                                    │  ║
║  │  j/k: navigate   Enter: select   Esc: cancel                       │  ║
║  └────────────────────────────────────────────────────────────────────┘  ║
╚══════════════════════════════════════════════════════════════════════════╝
```

> Note: The original server is labeled `[original]` but is NOT pre-highlighted.
> A non-original server is focused by default. This enforces DI-007: replay
> never defaults to the original server.

### Replay Confirmation Dialog (ELM-021) — after target is selected

```
╔══════════════════════════════════════════════════════════════════════════╗
║  TRAFFIC INSPECTOR — my-server    [● REC]  [47 msgs]                    ║
║  ┌ Confirm Replay ────────────────────────────────────────────────────┐  ║
║  │                                                                    │  ║
║  │  Replay this message?                                              │  ║
║  │                                                                    │  ║
║  │    Method:  tools/call                                             │  ║
║  │    ID:      003                                                    │  ║
║  │    Target:  staging-server  (localhost:9001)           ← prominent │  ║
║  │                                                                    │  ║
║  │  ┌────────────────────────────────────────────────────────────┐   │  ║
║  │  │ ⚠ WARNING: Replaying to original server — responses may   │   │  ║  ← shown only when
║  │  │   differ due to state changes.                             │   │  ║     target == original
║  │  └────────────────────────────────────────────────────────────┘   │  ║
║  │                                                                    │  ║
║  │            [ ✓ Confirm (Enter/y) ]   [ ✗ Cancel (Esc/n) ]        │  ║
║  └────────────────────────────────────────────────────────────────────┘  ║
╚══════════════════════════════════════════════════════════════════════════╝
```

> The warning block is **only shown** when the user explicitly selected the
> original server as the replay target (EC-007). It does not block the replay —
> the user confirmed intentional choice — but it must be visible at confirmation.
> "Confirm" button is focused by default.

### Replay Progress (ELM-022) — after confirmation

```
╔══════════════════════════════════════════════════════════════════════════╗
║  TRAFFIC INSPECTOR — my-server    [● REC]  [47 msgs]                    ║
║  ┌ Replay Progress ───────────────────────────────────────────────────┐  ║
║  │  Replaying to staging-server (localhost:9001)                      │  ║
║  │                                                                    │  ║
║  │  [████████████████████░░░░░░░░░░]  1 / 1 messages sent            │  ║
║  │                                                                    │  ║
║  │  Responses received: 0 / 1                                         │  ║
║  └────────────────────────────────────────────────────────────────────┘  ║
╚══════════════════════════════════════════════════════════════════════════╝
```

> Progress dismisses automatically on completion. On error, dialog stays open
> and shows error text with `[ Dismiss ]` button.

---

## Element Inventory

| ID | Widget | Type | Position | Description |
|----|--------|------|----------|-------------|
| ELM-001 | Panel | Panel | Full pane | Pane border + title |
| ELM-002 | Title bar | Text | Top | Server name + REC badge + msg count + filter status |
| ELM-003 | REC badge | Badge | Title | `[● REC]` when capture active, `[○ PAUSED]` when paused |
| ELM-004 | Filter bar | InlineForm | Below title | Filter fields (Method, Dir, Status, Time range) |
| ELM-005 | Column headers | Text | Row 3 | TIME, ID, DIR, METHOD, STATUS, MS, SIZE |
| ELM-006 | Message rows | Table | Data area | One row per message |
| ELM-007 | Timestamp | Text | Row col 1 | `HH:MM:SS.mmm` format |
| ELM-008 | Message ID | Text | Row col 2 | Correlation ID |
| ELM-009 | Direction glyph | Glyph | Row col 3 | `▶` = request, `◀` = response |
| ELM-010 | Method | Text | Row col 4 | JSON-RPC method name |
| ELM-011 | Status badge | Badge | Row col 5 | `OK` / `ERR` / `NOTIF` |
| ELM-012 | Latency | Text | Row col 6 | Request RTT in ms (response rows show `-`) |
| ELM-013 | Size | Text | Row col 7 | Message size (B / KB) |
| ELM-014 | Correlation marker | Glyph | Row prefix | `▶` on selected + its pair |
| ELM-015 | Key hint bar | Text | Last row | Context-sensitive shortcuts |
| ELM-016 | Detail view | FullPane | Modal-like | JSON viewer for expanded message |
| ELM-017 | Split view | SplitPane | Modal-like | Request+Response side-by-side |
| ELM-018 | Scrollbar | Scrollbar | Right edge | Position indicator |
| ELM-019 | Replay Target Dialog | Dialog | Overlay | Lists all registered servers; user selects replay target; original server labeled but NOT pre-highlighted (DI-007) |
| ELM-020 | Server list | SelectList | Inside ELM-019 | Navigable list of registered servers with name, address, and `[original]` label on the source server |
| ELM-021 | Replay Confirmation Dialog | Dialog | Overlay | Shows method, message ID, and **target server name + address** prominently; warning badge when target == original server |
| ELM-022 | Replay Progress indicator | ProgressBar | Inside overlay | Shows "N / M messages sent" and "responses received" while replay executes; auto-dismisses on success; shows error text on failure |

---

## Column Widths

| Column | Width | Notes |
|--------|-------|-------|
| TIME | 12 cols | `HH:MM:SS.mmm` |
| ID | 6 cols | Integer or `▶ NNN` when correlated |
| DIR | 3 cols | `▶` or `◀` |
| METHOD | 24 cols | Truncated with `…` |
| STATUS | 8 cols | `OK` / `ERR` / `NOTIF` |
| MS | 6 cols | Right-aligned |
| SIZE | 6 cols | Right-aligned |

On narrow panes (< 80 cols for the inspector), SIZE column hides; MS column hides below 60 cols.

---

## Request-Response Correlation

When a row is selected:
- The paired message (request↔response) is highlighted with the same background tint
- A `▶` prefix marker appears on both rows of the pair
- The pair highlight uses `color.bg.focused` tint (lighter than selection)
- RTT is computed as `response.timestamp - request.timestamp` in milliseconds
- If no response received (in-flight): status shows `WAIT` with elapsed time

---

## Direction Indicator Colors

| Direction | Glyph | Text | Color |
|-----------|-------|------|-------|
| Request (client→server) | `▶` | `▶ ` | `color.fg.accent` (blue) |
| Response (server→client) | `◀` | `◀ ` | `color.fg.primary` (white) |
| Notification | `↕` | `↕ ` | `color.fg.secondary` (dim) |

ASCII fallback: `>` / `<` / `^`

---

## State Definitions

| State Variable | Type | Description |
|----------------|------|-------------|
| `messages` | RingBuffer<CapturedMessage> | Bounded ring buffer (max 100MB) |
| `filtered_messages` | Vec<MessageRef> | References into buffer after filter |
| `selected_idx` | usize | Highlighted row |
| `scroll_offset` | usize | Scroll position |
| `capture_active` | bool | Capture running |
| `filter_state` | FilterSpec | Active filter (method, dir, status, time) |
| `filter_bar_open` | bool | Filter bar visible |
| `detail_open` | bool | Full detail view open |
| `split_view_open` | bool | Split request/response view open |
| `detail_message_id` | Option<MessageId> | Message in detail view |
| `search_text` | String | `/` search text |
| `search_matches` | Vec<usize> | Indices of matching rows |
| `search_cursor` | usize | Current search match position |
| `replay_target_dialog_open` | bool | Replay Target Dialog (ELM-019) visible |
| `replay_confirm_dialog_open` | bool | Replay Confirmation Dialog (ELM-021) visible |
| `replay_selected_target` | Option<ServerId> | Server chosen in ELM-019 (None until user selects) |
| `replay_in_progress` | bool | Replay executing; progress overlay (ELM-022) visible |
| `replay_progress` | (usize, usize) | (messages_sent, messages_total) for ELM-022 |

---

## Filter Specification

Filter bar fields:

| Field | Type | Options |
|-------|------|---------|
| Method | TextInput | Prefix match; e.g., `tools/` matches all tools methods |
| Direction | Dropdown | `both` / `requests` / `responses` / `notifications` |
| Status | Dropdown | `all` / `ok` / `error` / `notifications` |
| Time range | TextInput | `last:30s` / `last:5m` / `HH:MM:SS-HH:MM:SS` |

Filters are AND-combined. "0 of N" shown when all filtered out.

---

## Keyboard Shortcuts

| Key | Action | Notes |
|-----|--------|-------|
| `j` / `↓` | Next message | |
| `k` / `↑` | Previous message | |
| `g` / `Home` | Oldest message | |
| `G` / `End` | Newest message (live tail) | |
| `Enter` / `x` | Expand selected message | Full JSON detail view |
| `↔` (or `p`) | View request/response pair | Split view |
| `f` | Open/close filter bar | |
| `Esc` | Close filter / close detail / clear search | Priority: detail > filter > search |
| `c` | Clear capture buffer | Confirm dialog |
| `P` | Pause / resume capture | Toggles REC badge |
| `/` | Search within visible messages | Highlights matches |
| `n` | Next search match | |
| `N` | Previous search match | |
| `R` | Initiate replay | Opens Replay Target Dialog (ELM-019); does NOT immediately send any message |
| `Enter` | Select server (in ELM-019) / Confirm replay (in ELM-021) | Context-sensitive |
| `y` | Confirm replay (in ELM-021) / Yank message (in list/detail) | Context-sensitive |
| `n` | Cancel at ELM-021 confirmation | Dismisses dialog; no replay |
| `Esc` | Cancel any replay dialog / Close detail / Clear filter | Priority: replay dialog > detail > filter > search |
| `Tab` | Move focus to Capability Browser | |

---

## JSON Syntax Highlighting

Implemented per the JSON viewer widget contract. Color tokens:

```
{ "key": "value", "count": 42, "flag": true, "sub": { ... } }
   ^^^    ^^^^^^^   ^^^^^^^    ^^    ^^^^      ^^^   ^^^
  key    string    key       number bool     key   bracket
```

Token classes: `syntax.key`, `syntax.string`, `syntax.number`, `syntax.bool`, `syntax.null`, `syntax.bracket`, `syntax.error`.

Error highlighting: if a response has `isError: true`, the entire `result` value block is highlighted with `syntax.error` background.

---

## Buffer Management

Per BC-4.09.003:
- In-memory ring buffer maximum: 100MB (configurable via `[capture].max_memory_mb`)
- Oldest messages evicted when buffer full
- Status bar shows `[WRAP]` badge when eviction has occurred
- Optional disk spill when `rotate_to_disk = true` in config

---

## Accessibility Notes

- **Direction:** Glyph + text (`▶`/`◀`/`↕`) ensures non-color direction indication
- **Status:** `OK`/`ERR`/`NOTIF` text labels; not color-only
- **No auto-scroll lock:** When user navigates up (live tail mode), auto-scroll pauses; `G` resumes tail
- **Search:** `/` opens inline search; matches highlighted with `color.bg.selected` + `[n/N]` navigation
- **Focus order:** Title → Filter bar (if open) → Message table → Key hint bar
- **Detail view:** Opens in full-pane replace (not modal); `Esc` returns to list

---

## BC Traceability

| BC ID | How This Screen Satisfies It |
|-------|------------------------------|
| BC-4.09.001 | Message log shows all captured JSON-RPC messages transparently |
| BC-4.09.002 | Timing column (MS) shows per-message RTT; RTT in detail view |
| BC-4.09.003 | Buffer management; WRAP indicator; 100MB cap |
| BC-4.10.001 | Filter bar (method, direction, status, time range) |
| BC-4.10.002 | `/` search for full-text payload search |
| BC-4.10.003 | Replay workflow via `R` key, with: PRE-002 (DI-007) — ELM-019 Replay Target Dialog forces explicit server selection before replay proceeds; PRE-004 — ELM-021 Confirmation Dialog requires user confirmation; POST-003/POST-004 — comparison status badge on replayed message row; POST-005 — ELM-019 defaults focus to non-original server (original labeled but not pre-selected); POST-006 — ELM-022 shows real-time progress (N/M sent, responses received); EC-007 — warning badge in ELM-021 when user selects original server as target |
| BC-3.08.001 | JSON syntax highlighting in detail/split views |
| BC-3.08.005 | Direction/status as glyph + text, not color-only |
