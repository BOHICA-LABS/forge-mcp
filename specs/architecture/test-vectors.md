---
document_type: architecture-section
level: L3
section: test-vectors
version: "1.0"
status: draft
producer: architect
timestamp: 2026-03-29T14:05:00
phase: 1b
inputs: [domain-spec/L2-INDEX.md, specs/prd.md]
traces_to: ARCH-INDEX.md
---

# Test Vectors

Key test vectors for each subsystem. These are reference inputs/outputs for implementation validation.

## Config Parsing Vectors

### Claude Desktop config (mcpServers schema)

```json
{
  "mcpServers": {
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": { "GITHUB_TOKEN": "ghp_xxx" }
    }
  }
}
```

**Expected:** 1 server, name="github", transport=Stdio, source=ClaudeDesktop.

### VS Code config (servers schema)

```json
{
  "servers": {
    "playwright": {
      "type": "stdio",
      "command": "npx",
      "args": ["-y", "@microsoft/mcp-server-playwright"]
    }
  }
}
```

**Expected:** 1 server, name="playwright", transport=Stdio, source=VsCode.

## MCP Protocol Vectors

### Initialize request

```json
{
  "jsonrpc": "2.0",
  "method": "initialize",
  "id": 1,
  "params": {
    "protocolVersion": "2025-11-25",
    "capabilities": {
      "sampling": {},
      "roots": { "listChanged": true }
    },
    "clientInfo": {
      "name": "forge-mcp",
      "version": "0.1.0"
    }
  }
}
```

### Tools/list response with pagination

```json
{
  "jsonrpc": "2.0",
  "result": {
    "tools": [
      {
        "name": "read_file",
        "description": "Read file contents",
        "inputSchema": {
          "type": "object",
          "properties": { "path": { "type": "string" } },
          "required": ["path"]
        }
      }
    ],
    "nextCursor": "abc123"
  },
  "id": 2
}
```

### Tool error (isError=true, NOT a JSON-RPC error)

```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [
      { "type": "text", "text": "File not found: /missing.txt" }
    ],
    "isError": true
  },
  "id": 3
}
```

**Expected classification:** McpError::Tool — must NOT be classified as Protocol error.

### Protocol error (JSON-RPC error)

```json
{
  "jsonrpc": "2.0",
  "error": { "code": -32601, "message": "Method not found" },
  "id": 4
}
```

**Expected classification:** McpError::Protocol — must NOT be classified as Tool error.

## Security Detection Vectors

### SSRF to metadata endpoint

```json
{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "id": 5,
  "params": {
    "name": "fetch_url",
    "arguments": {
      "url": "http://169.254.169.254/latest/meta-data/"
    }
  }
}
```

**Expected:** SecurityFinding(severity=Critical, category=AST01, confidence=0.95)

### Dangerous tool pattern (file system write)

```json
{
  "name": "write_file",
  "description": "Write arbitrary content to any file path",
  "inputSchema": {
    "type": "object",
    "properties": {
      "path": { "type": "string" },
      "content": { "type": "string" }
    }
  }
}
```

**Expected:** SecurityFinding(severity=High, category=AST03, confidence=0.8)

## Health Metric Vectors

**Input:** 10 request/response pairs with latencies [5ms, 8ms, 12ms, 3ms, 7ms, 15ms, 6ms, 9ms, 4ms, 11ms]

**Expected:**
- p50: 7.5ms
- p99: 15ms
- error_rate: 0.0
- throughput: 10/window

## Alert State Machine Vectors

| Initial State | Metric Value | Threshold | Expected Result |
|--------------|-------------|-----------|----------------|
| Normal | 500ms | 100ms | → Breached |
| Breached | 50ms | 100ms | → Recovered |
| Recovered | 200ms | 100ms | → Breached |
| Normal | 50ms | 100ms | → Normal (remains, NOT Recovered) |

The last case validates that the Recovered state is only reachable from Breached, never from Normal — enforcing the state machine invariant VP-007.
