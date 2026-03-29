---
document_type: prd-supplement-error-taxonomy
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-03-29T11:25:00
phase: 1a
inputs: [prd.md]
input-hash: ""
traces_to: prd.md
---

# Error Taxonomy: Forge MCP

> PRD supplement — extracted from PRD Section 5.
> Referenced by: implementer, test-writer.

## Error Categories

| Category Code | Category | Description |
|--------------|----------|-------------|
| CON | Connection | Transport, DNS, TLS, session errors |
| CFG | Configuration | Config file parsing, schema, path errors |
| PRO | Protocol | MCP/JSON-RPC protocol violations |
| TUI | Terminal UI | Rendering, terminal capability errors |
| SEC | Security | Security auditing rule and report errors |
| CAP | Capture | Traffic capture buffer and replay errors |
| MON | Monitoring | Metric collection and alerting errors |

## Severity Definitions

| Severity | Meaning | Exit Code Impact |
|----------|---------|-----------------|
| broken | Cannot continue — operation fails | Non-zero exit (2, 3, or 100) |
| degraded | Partial result possible — some data missing | Zero exit with stderr warning |
| cosmetic | Display/formatting issue — data correct | Zero exit |

## Error Catalog

### Connection Errors (CON)

| Error Code | Severity | Exit Code | Message Format | Traces To |
|-----------|----------|-----------|---------------|-----------|
| E-CON-001 | broken | 2 | `Connection failed: DNS resolution failed for <hostname>` | FM-001, CAP-002 |
| E-CON-002 | broken | 2 | `Connection failed: server process exited with code <code>` | FM-002, DEC-001 |
| E-CON-003 | broken | 2 | `Connection failed: timeout after <seconds>s waiting for server response` | FM-003, DEC-005 |
| E-CON-004 | broken | 2 | `Connection failed: TLS error — <detail> for <hostname>` | FM-004 |
| E-CON-005 | broken | 2 | `Connection failed: server rejected capability negotiation — <reason>` | DI-001 |
| E-CON-006 | degraded | 0 | `Connection warning: server protocol version <version> differs from expected <expected>` | BC-2.04.003 |
| E-CON-007 | broken | 2 | `Connection failed: Streamable HTTP session lost — <reason>` | FM-021, R-015 |
| E-CON-008 | degraded | 0 | `Connection recovered: session re-established after <event>` | FM-021 |
| E-CON-009 | broken | 2 | `Connection failed: authentication failed — HTTP <status> for <url>` | BC-1.02.002 |
| E-CON-010 | degraded | 0 | `Connection warning: insecure HTTP connection to <url>` | BC-1.02.002 |
| E-CON-011 | broken | 2 | `Connection failed: server unavailable — HTTP <status> for <url>` | BC-1.02.002 |

### Configuration Errors (CFG)

| Error Code | Severity | Exit Code | Message Format | Traces To |
|-----------|----------|-----------|---------------|-----------|
| E-CFG-001 | degraded | 0 | `Config source <editor> not found at <path> — skipped` | FM-009, DEC-007 |
| E-CFG-002 | degraded | 0 | `Config parse error in <path> at line <line>, column <col>: <detail>` | FM-007, DEC-008 |
| E-CFG-003 | degraded | 0 | `Config schema mismatch in <path>: expected <expected_key>, found <actual_key>` | FM-008 |
| E-CFG-004 | degraded | 0 | `Config permission denied: cannot read <path>` | FM-006 |
| E-CFG-005 | broken | 3 | `No config sources found — provide --config or install a supported editor` | CAP-001 |
| E-CFG-006 | degraded | 0 | `Config conflict: server "<name>" defined differently in <source_a> and <source_b>` | DEC-010 |

### Protocol Errors (PRO)

| Error Code | Severity | Exit Code | Message Format | Traces To |
|-----------|----------|-----------|---------------|-----------|
| E-PRO-001 | broken | 100 | `Protocol error: malformed JSON-RPC from server — <detail>` | DEC-002, ASM-003 |
| E-PRO-002 | degraded | 0 | `Protocol warning: server advertised <capability> but <method> returned error` | DEC-003, DEC-012 |
| E-PRO-003 | degraded | 0 | `Protocol warning: method <method> not available — server lacks <capability> capability` | DI-002 |
| E-PRO-004 | degraded | 0 | `Protocol warning: pagination cursor loop detected after <n> pages for <method>` | DEC-020, FM-019 |
| E-PRO-005 | degraded | 0 | `Protocol warning: tool execution error (isError=true) for <tool>: <message>` | DI-020, DEC-023 |
| E-PRO-006 | degraded | 0 | `Protocol warning: ignoring progress notification for cancelled request <id>` | DEC-024 |
| E-PRO-007 | broken | 2 | `Protocol error: sampling request received but no LLM provider configured` | DEC-016, FM-016 |
| E-PRO-008 | broken | 2 | `Protocol error: elicitation request received in non-interactive mode` | DEC-017, FM-017 |
| E-PRO-009 | degraded | 0 | `Protocol warning: batch response reordered — matched <n> responses by id` | DEC-022 |

### Terminal UI Errors (TUI)

| Error Code | Severity | Exit Code | Message Format | Traces To |
|-----------|----------|-----------|---------------|-----------|
| E-TUI-001 | degraded | 0 | `TUI degraded: terminal lacks truecolor — falling back to <mode>` | FM-013 |
| E-TUI-002 | cosmetic | 0 | `TUI: terminal resize detected (<old_w>×<old_h> → <new_w>×<new_h>)` | FM-014 |
| E-TUI-003 | degraded | 0 | `TUI degraded: Unicode not supported — falling back to ASCII box drawing` | FM-013 |
| E-TUI-004 | degraded | 0 | `TUI degraded: terminal size <w>×<h> below minimum 80×24 — some panels hidden` | BC-3.06.001 |

### Security Errors (SEC)

| Error Code | Severity | Exit Code | Message Format | Traces To |
|-----------|----------|-----------|---------------|-----------|
| E-SEC-001 | degraded | 0 | `Security rule load failed: <rule_file> — <reason>` | FM-015 |
| E-SEC-002 | degraded | 0 | `Security auditing disabled: no rules loaded` | FM-015 |
| E-SEC-003 | degraded | 0 | `Security finding suppressed: <finding_id> — <reason>` | DI-012, BC-7.18.002 |
| E-SEC-004 | degraded | 0 | `Security auto-suppress suggested: <n> repeated findings for pattern <pattern>` | FM-018 |
| E-SEC-005 | degraded | 0 | `Schema drift detected: tool "<tool>" metadata changed since last connection` | FM-020, R-014 |

### Capture Errors (CAP)

| Error Code | Severity | Exit Code | Message Format | Traces To |
|-----------|----------|-----------|---------------|-----------|
| E-CAP-001 | degraded | 0 | `Capture buffer at <percent>% capacity (<current_mb>/<max_mb> MB) — oldest messages will be evicted` | FM-010, R-010 |
| E-CAP-002 | broken | 100 | `Capture failed: replay target server "<name>" not connected` | FM-011, DI-007 |
| E-CAP-003 | degraded | 0 | `Capture warning: message rate <rate>/sec exceeds display throttle — capture continues in background` | DEC-011 |

### Monitoring Errors (MON)

| Error Code | Severity | Exit Code | Message Format | Traces To |
|-----------|----------|-----------|---------------|-----------|
| E-MON-001 | degraded | 0 | `Alert breached: <metric> for server "<server>" exceeded threshold <threshold> (current: <value>)` | BC-6.14.002 |
| E-MON-002 | degraded | 0 | `Alert recovered: <metric> for server "<server>" returned to normal (<value>)` | BC-6.14.002 |
| E-MON-003 | degraded | 0 | `Monitoring: no traffic observed for server "<server>" — metrics stale since <timestamp>` | ASM-009 |
