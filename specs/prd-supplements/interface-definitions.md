---
document_type: prd-supplement-interface-definitions
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

# Interface Definitions: Forge MCP

> PRD supplement — extracted from PRD Section 3.
> Referenced by: implementer, test-writer, devops-engineer.

## CLI Interface

```
forge-mcp — Unified MCP server inspection, debugging, monitoring, and security auditing tool

USAGE:
    forge-mcp [OPTIONS] <COMMAND>

COMMANDS:
    list        List discovered MCP servers and their capabilities
    call        Invoke a tool on an MCP server
    info        Show detailed information about a server or capability
    grep        Search across server capabilities by pattern
    test        Run protocol conformance tests against a server
    audit       Run runtime security audit against a server
    tui         Launch interactive TUI dashboard
    daemon      Manage the background daemon process
    config      Show or compare editor MCP configurations
    diff        Compare two MCP servers side-by-side
    help        Print help for a command

GLOBAL OPTIONS:
    -s, --server <NAME>          Target server name (from config registry)
    -c, --config <PATH>          Additional config file path [repeatable]
    -o, --output <FORMAT>        Output format: json|text|table [default: text]
    -q, --quiet                  Suppress stderr diagnostics
    -v, --verbose                Increase verbosity (-v, -vv, -vvv)
    --no-daemon                  Connect directly, bypass daemon session pool
    --timeout <SECONDS>          Request timeout in seconds [default: 30]
    --color <WHEN>               Color output: auto|always|never [default: auto]
    -h, --help                   Print help
    -V, --version                Print version

SUBCOMMAND: list
    forge-mcp list [OPTIONS]

    List discovered servers, tools, resources, or prompts.

    OPTIONS:
        --source <EDITOR>        Filter by config source: claude|cursor|vscode|windsurf|all [default: all]
        --type <TYPE>            What to list: servers|tools|resources|prompts [default: servers]
        --server <NAME>          List capabilities for a specific server
        --json                   Shorthand for --output json

SUBCOMMAND: call
    forge-mcp call <SERVER> <TOOL> [ARGS...]

    Invoke a tool on the specified server.

    ARGUMENTS:
        <SERVER>                 Server name
        <TOOL>                   Tool name to invoke
        [ARGS...]                Tool arguments as key=value pairs or JSON string

    OPTIONS:
        --json-args <JSON>       Tool arguments as JSON object
        --raw                    Print raw JSON-RPC response (no formatting)
        --wait <SECONDS>         Wait timeout for long-running tasks [default: 60]

SUBCOMMAND: info
    forge-mcp info <SERVER> [CAPABILITY]

    Show detailed information about a server or a specific capability.

    ARGUMENTS:
        <SERVER>                 Server name
        [CAPABILITY]             Specific tool/resource/prompt name

    OPTIONS:
        --schema                 Show JSON Schema for tool inputs
        --metadata               Include all annotations and metadata

SUBCOMMAND: grep
    forge-mcp grep <PATTERN> [OPTIONS]

    Search across server capabilities by regex pattern.

    ARGUMENTS:
        <PATTERN>                Regex search pattern

    OPTIONS:
        --scope <SCOPE>          Search scope: names|descriptions|schemas|all [default: all]
        --server <NAME>          Limit search to specific server
        --type <TYPE>            Limit to: tools|resources|prompts|all [default: all]

SUBCOMMAND: test
    forge-mcp test <SERVER> [OPTIONS]

    Run protocol conformance tests against a server.

    OPTIONS:
        --suite <SUITE>          Test suite: full|negotiation|methods|transport|errors [default: full]
        --spec-version <VER>     Target spec version: 2025-11-25|2024-11-05 [default: 2025-11-25]
        --format <FMT>           Result format: json|junit|text [default: text]
        --output-file <PATH>     Write results to file (default: stdout)
        --fail-fast              Stop on first failure

SUBCOMMAND: audit
    forge-mcp audit <SERVER> [OPTIONS]

    Run runtime security audit against a server.

    OPTIONS:
        --rules <PATH>           Custom security rules file
        --suppress <PATH>        Suppression rules file
        --min-confidence <F>     Minimum confidence threshold [0.0-1.0, default: 0.5]
        --format <FMT>           Report format: json|text|sarif [default: text]
        --output-file <PATH>     Write report to file (default: stdout)
        --ast10                  Include OWASP AST10 coverage summary

SUBCOMMAND: tui
    forge-mcp tui [OPTIONS]

    Launch interactive TUI dashboard.

    OPTIONS:
        --server <NAME>          Auto-connect to server on launch
        --layout <LAYOUT>        Initial layout: default|traffic|health|security [default: default]
        --fps <N>                Target frame rate [default: 60]
        --mouse <BOOL>           Enable mouse input [default: true]
        --unicode <BOOL>         Use Unicode box drawing [default: auto]

SUBCOMMAND: daemon
    forge-mcp daemon <ACTION>

    Manage the background daemon process.

    ACTIONS:
        start                    Start daemon (usually automatic)
        stop                     Stop daemon gracefully
        status                   Show daemon status and active sessions
        restart                  Restart daemon

    OPTIONS:
        --idle-timeout <SECS>    Idle timeout before auto-shutdown [default: 300]
        --port <PORT>            Daemon listen port [default: auto]

SUBCOMMAND: config
    forge-mcp config [OPTIONS]

    Show or compare editor MCP configurations.

    OPTIONS:
        --source <EDITOR>        Show config for specific editor
        --diff                   Show differences across all editor configs
        --format <FMT>           Output format: json|text|table [default: text]

SUBCOMMAND: diff
    forge-mcp diff <SERVER_A> <SERVER_B> [OPTIONS]

    Compare two MCP servers side-by-side.

    ARGUMENTS:
        <SERVER_A>               First server name
        <SERVER_B>               Second server name

    OPTIONS:
        --scope <SCOPE>          Comparison scope: tools|capabilities|behavior|all [default: all]
        --format <FMT>           Output format: json|text|table [default: text]
```

## Exit Code Semantics

| Code | Meaning | When |
|------|---------|------|
| 0 | Success | Command completed without errors or test failures |
| 1 | Test/audit failure | Conformance test failed, or security audit found critical findings |
| 2 | Connection error | Could not connect to target server (DNS, timeout, auth, transport) |
| 3 | Config error | Config file not found, parse error, or schema mismatch |
| 4 | Security finding | Audit found findings above minimum confidence (non-critical) |
| 100 | Internal error | Unexpected internal error (bug) |

## JSON Output Schema

### Server List Output

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "servers": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "name": { "type": "string" },
          "transport": { "enum": ["stdio", "http"] },
          "source": { "enum": ["claude_desktop", "cursor", "vscode", "windsurf", "manual"] },
          "status": { "enum": ["connected", "disconnected", "error", "unknown"] },
          "protocol_version": { "type": "string", "nullable": true },
          "capabilities": {
            "type": "object",
            "properties": {
              "tools": { "type": "boolean" },
              "resources": { "type": "boolean" },
              "prompts": { "type": "boolean" },
              "logging": { "type": "boolean" },
              "completions": { "type": "boolean" },
              "sampling": { "type": "boolean" },
              "elicitation": { "type": "boolean" }
            }
          }
        },
        "required": ["name", "transport", "source", "status"]
      }
    },
    "sources_checked": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "editor": { "type": "string" },
          "path": { "type": "string" },
          "status": { "enum": ["found", "missing", "error"] }
        }
      }
    }
  }
}
```

### Tool Call Output

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "server": { "type": "string" },
    "tool": { "type": "string" },
    "result": {
      "type": "object",
      "properties": {
        "content": { "type": "array", "items": { "type": "object" } },
        "isError": { "type": "boolean" }
      }
    },
    "timing": {
      "type": "object",
      "properties": {
        "request_ms": { "type": "number" },
        "response_ms": { "type": "number" },
        "total_ms": { "type": "number" }
      }
    }
  }
}
```

### Security Audit Output

```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "type": "object",
  "properties": {
    "server": { "type": "string" },
    "audit_timestamp": { "type": "string", "format": "date-time" },
    "findings": {
      "type": "array",
      "items": {
        "type": "object",
        "properties": {
          "id": { "type": "string" },
          "severity": { "enum": ["critical", "high", "medium", "low", "info"] },
          "confidence": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
          "category": { "type": "string" },
          "ast10_mapping": { "type": "string", "nullable": true },
          "title": { "type": "string" },
          "description": { "type": "string" },
          "evidence": {
            "type": "object",
            "properties": {
              "message_id": { "type": "string" },
              "method": { "type": "string" },
              "content_excerpt": { "type": "string" }
            }
          },
          "suppressed": { "type": "boolean" }
        },
        "required": ["id", "severity", "confidence", "category", "title"]
      }
    },
    "summary": {
      "type": "object",
      "properties": {
        "total_findings": { "type": "integer" },
        "by_severity": { "type": "object" },
        "ast10_coverage": {
          "type": "object",
          "properties": {
            "covered": { "type": "array", "items": { "type": "string" } },
            "not_applicable": { "type": "array", "items": { "type": "string" } },
            "coverage_percent": { "type": "number" }
          }
        }
      }
    }
  }
}
```

## Config File Schema

```toml
# forge-mcp.toml — Forge MCP configuration
# Location: ~/.config/forge-mcp/config.toml or ./forge-mcp.toml (project-local)

# Global settings
[global]
default_output = "text"          # json | text | table
color = "auto"                   # auto | always | never
verbose = 0                      # 0-3
timeout = 30                     # seconds

# Daemon settings
[daemon]
enabled = true                   # Enable background daemon
idle_timeout = 300               # Seconds before auto-shutdown
# port = 0                       # 0 = auto-assign

# Config sources — override auto-detection
[sources]
claude_desktop = true            # Enable Claude Desktop config discovery
cursor = true                    # Enable Cursor config discovery
vscode = true                    # Enable VS Code config discovery
windsurf = true                  # Enable Windsurf config discovery
# additional = ["/path/to/custom/config.json"]

# Per-server overrides
[servers.my-server]
timeout = 60                     # Override default timeout
# transport = "stdio"            # Force transport type
# env = { API_KEY = "..." }      # Additional env vars

# TUI settings
[tui]
fps = 60                         # Target frame rate
mouse = true                     # Enable mouse input
unicode = "auto"                 # auto | true | false
# layout = "default"             # default | traffic | health | security
# theme = "dark"                 # dark | light | auto

# Health monitoring
[monitoring]
enabled = true
# latency_threshold_ms = 1000
# error_rate_threshold = 0.05
# throughput_threshold = 0

# Security auditing
[security]
# rules_path = "~/.config/forge-mcp/rules/"
# suppress_path = "~/.config/forge-mcp/suppress.toml"
min_confidence = 0.5             # Minimum confidence for findings
# ast10_enabled = true

# Capture settings
[capture]
max_memory_mb = 100              # Maximum capture buffer size
# max_messages = 100000          # Maximum messages to retain
# rotate_to_disk = false         # Spill to disk when buffer full
```

## Flag Interactions

| Flag A | Flag B | Interaction | Resolution |
|--------|--------|-------------|------------|
| `--quiet` | `--verbose` | Conflicts | `--quiet` wins: suppresses stderr entirely |
| `--output json` | `--json` | Redundant | Either sets JSON output; no conflict |
| `--no-daemon` | `daemon start` | Conflicts | Error: "Cannot use --no-daemon with daemon subcommand" |
| `--server` | `list --type servers` | Narrows | Lists capabilities for that server only |
| `--color never` | TUI mode | Override | TUI ignores `--color never` (requires color for rendering) |
| `test --fail-fast` | `test --format junit` | Compatible | JUnit output includes only tests up to first failure |
| `audit --min-confidence` | `audit --suppress` | Composable | Suppression applied after confidence filter |
| `--timeout` | `call --wait` | Composable | `--timeout` is transport-level; `--wait` is task-level (stacks) |

## TUI Widget Contracts

| Widget | Location | Data Source | Update Frequency |
|--------|----------|-------------|-----------------|
| Server Browser | Left pane | Config registry + connection state | On connect/disconnect events |
| Capability Explorer | Center-top | Selected server's negotiated capabilities | On server selection change |
| Traffic Inspector | Center-bottom | Capture buffer (filtered view) | Per-message (real-time) |
| Health Metrics | Right pane | Metric collectors per server | 1Hz (sparklines/histograms) |
| Status Bar | Bottom | Global state (daemon, active server, capture) | On state change |
| Shortcut Bar | Top/Bottom | Static key bindings | Static (render once) |
| Security Findings | Overlay/Tab | Security auditor findings list | On new finding |
| JSON Detail | Popup | Selected message from traffic inspector | On message selection |
