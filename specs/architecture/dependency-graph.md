---
document_type: architecture-section
level: L3
section: dependency-graph
version: "1.0"
status: draft
producer: architect
timestamp: 2026-03-29T14:05:00
phase: 1b
inputs: [domain-spec/L2-INDEX.md, specs/prd.md]
traces_to: ARCH-INDEX.md
---

# Dependency Graph

**Direction rule:** Dependencies flow DOWNWARD. Higher-level modules depend on lower-level. Core depends on nothing internal (only rmcp).

## ASCII DAG

```
                    forge-mcp (binary)
                   /    |    |    \
                  /     |    |     \
          forge-tui  forge-daemon  forge-conformance
           /  |  \      |            |
          /   |   \     |            |
 forge-traffic |  forge-health      |
          \   |   /     |           |
           \  |  /      |          /
        forge-security  |         /
              \         |        /
               \        |       /
            forge-discovery    /
                 \            /
                  \          /
                forge-config/
                      \
                   forge-core
                       |
                     [rmcp]
```

## Dependency Table

| Crate | Depends On | Reason |
|-------|-----------|--------|
| forge-core | rmcp, serde, serde_json | Protocol adapter; wraps rmcp Peer API |
| forge-discovery | forge-core, serde_json, dirs | Config parsing; needs core types for ServerEntry |
| forge-config | forge-discovery, forge-core | Needs parsed configs + core types for schema diff |
| forge-traffic | forge-core | Captures core Message types |
| forge-health | forge-core | Metrics derived from core Message types |
| forge-security | forge-core, forge-traffic | Analyzes messages from traffic capture |
| forge-tui | forge-core, forge-traffic, forge-health, forge-security, forge-discovery | Renders all subsystem data |
| forge-daemon | forge-core, forge-discovery | Manages connections and sessions |
| forge-conformance | forge-core | Sends protocol methods, validates responses |
| forge-mcp | ALL crates | Binary entry point; dispatches to subsystems |

## Dependency Direction Rules

1. **forge-core depends on nothing internal** — only rmcp and standard serialization crates
2. **forge-discovery depends only on forge-core** — config parsing needs MCP types
3. **forge-traffic, forge-health depend only on forge-core** — data layer modules
4. **forge-security depends on forge-core + forge-traffic** — analyzes captured messages
5. **forge-tui depends on data layer modules** — renders everything
6. **forge-daemon depends on forge-core + forge-discovery** — connection lifecycle
7. **forge-config depends on forge-discovery + forge-core** — needs parsed configs
8. **forge-conformance depends on forge-core** — protocol testing
9. **forge-mcp (binary) depends on everything** — entry point
10. **No circular dependencies** — DAG is strictly acyclic
