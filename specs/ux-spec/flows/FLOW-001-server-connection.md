---
document_type: ux-spec-flow
flow_id: FLOW-001
flow_name: Server Connection
version: "1.0"
status: draft
producer: ux-designer
timestamp: 2026-03-29T14:15:00
phase: 1c
traces_to: UX-INDEX.md
screens: [SCR-001, SCR-002, SCR-003]
prd_requirements: [BC-1.02.001, BC-1.02.002, BC-1.02.003, BC-2.04.001, BC-3.08.003]
---

# Flow: Server Connection (FLOW-001)

> The entry-point flow for using the TUI. User selects a server, triggers
> connection, watches capability negotiation complete, and arrives at a
> fully populated dashboard. Covers both stdio and HTTP transports.

---

## Flow Diagram (ASCII)

```
  [TUI launch]
       │
       ▼
  SCR-001 + SCR-002: Server list shown
  All servers in DISC/UNK state
       │
       │  User navigates j/k to select a server
       ▼
  SCR-002: Server highlighted, tooltip available (i)
       │
       │  User presses Enter
       ▼
  SCR-002: Server status → BUSY (◌)
  Status bar: "Connecting…"
       │
       ├──[Connection failed]──────────────────────────────► Error path (see below)
       │
       │  Connection established
       ▼
  SCR-002: Status → CONN (●)
  SCR-001 header: server name + ● badge updated
  SCR-003: "Loading capabilities…" spinner shown
       │
       │  Capability negotiation completes (tools/list, resources/list, prompts/list)
       ▼
  SCR-003: Capability browser populated with tools/resources/prompts
  SCR-005: Health metrics panel begins collecting
  SCR-004: Traffic capture starts (initialize + list messages appear)
       │
       ▼
  [Dashboard fully populated — nominal state]
```

---

## Step-by-Step Sequence

| Step | Screen | User Action | System Response |
|------|--------|-------------|----------------|
| 1 | SCR-001 | TUI launches (or `Tab` to server sidebar) | Server list shown; servers in DISC/UNK state; focus on sidebar |
| 2 | SCR-002 | Press `j`/`k` to navigate server list | Highlighted server row updates |
| 3 | SCR-002 | (Optional) Press `i` to view tooltip | Detail tooltip shown: transport, source, last error |
| 4 | SCR-002 | Press `Enter` to connect selected server | Status badge → `◌ BUSY`; connecting animation starts |
| 5 | SCR-001 | (Waiting) | Status bar shows "Connecting to <server>…" |
| 6 | SCR-002 | (Automatic) Connection established | Status → `● CONN`; header badge updates |
| 7 | SCR-001/003 | (Automatic) Capability negotiation | `initialize` message appears in traffic; negotiated version shown |
| 8 | SCR-003 | (Automatic) Tools/Resources/Prompts listed | Capability browser populates; tab counts updated (e.g., `Tools(12)`) |
| 9 | SCR-005 | (Automatic) First metrics arrive | Sparklines begin rendering |
| 10 | SCR-004 | (Automatic) Traffic appears | `initialize` + list method messages in inspector |

---

## Success Path

After step 10: all three panes are populated.
- Server sidebar: `● CONN` badge with server name
- Capability browser: tabs show `Tools(N)`, `Resources(N)`, `Prompts(N)`
- Traffic inspector: 3–5 messages from connection setup
- Health panel: sparklines active, no alerts
- Status bar: `[Daemon: OK] [Capture: ON | N msgs] [Latency: Nms]`

---

## Error Paths

### Connection Refused

| Trigger | Connection attempt fails: server process not running |
|---------|------|
| Step | Step 5–6 |
| Display | Status badge → `✗ ERR ` in sidebar; error tooltip auto-opens |
| Tooltip | "Connection refused: server process not found" |
| Status bar | "Connection failed: <server>" |
| Recovery | User can press `r` to retry, or `n` to add corrected config |

### Timeout

| Trigger | No response from server within 30s |
|---------|------|
| Display | `✗ ERR ` badge; tooltip "Timeout after 30s" |
| Recovery | `r` to retry; check server status manually |

### Protocol Version Mismatch

| Trigger | Server only supports MCP 2024-11-05 (older spec) |
|---------|------|
| Display | `● CONN` (connection succeeds); tooltip notes "Proto: 2024-11-05 (degraded)" |
| Behavior | Capability browser only shows capabilities supported by 2024-11-05; newer capabilities dimmed |
| Per BC | BC-2.04.003: graceful degradation |

### No Servers Discovered

| Trigger | Config scan finds no servers |
|---------|------|
| Display | Server sidebar: "No servers found. Run: forge-mcp list" |
| Recovery | User runs `forge-mcp list` in CLI, or adds server manually via `n` |

### Already Connected (Re-select)

| Trigger | User presses Enter on an already-connected server |
|---------|------|
| Behavior | Active server switches to the selected server; capabilities re-loaded |

---

## Screen Transitions

| From | To | Trigger |
|------|----|---------|
| SCR-001 (initial) | SCR-002 focused | TUI launch (sidebar auto-focused) |
| SCR-002 (disconnected) | SCR-002 (connecting) | Enter key |
| SCR-002 (connecting) | SCR-002 (connected) | Connection success event |
| SCR-002 (connecting) | SCR-002 (error) | Connection failure event |
| SCR-002 (connected) | SCR-003 populated | Capability negotiation complete |

---

## Keyboard-Only Operation Path

1. TUI launches → focus auto-placed on server sidebar (SCR-002)
2. `j`/`k` to navigate server list
3. (Optional) `i` to preview tooltip, `Esc` to close
4. `Enter` to connect
5. Wait (no key press needed) — connection events are async
6. Once connected: `Tab` to move focus to capability browser (SCR-003)
7. Browse capabilities with `j`/`k`, switch tabs with `[`/`]`

No mouse required at any step.

---

## BC Traceability

| BC ID | Coverage |
|-------|---------|
| BC-1.02.001 | Stdio transport connection establishment (Enter on stdio server) |
| BC-1.02.002 | HTTP transport connection (same flow, different transport) |
| BC-1.02.003 | Connection lifecycle: connecting → connected → error handling |
| BC-2.04.001 | Capability negotiation (step 7) |
| BC-3.08.003 | Status badge updates (DISC → BUSY → CONN/ERR) |
