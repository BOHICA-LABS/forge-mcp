---
document_type: architecture-section
level: L3
section: dependency-graph
version: "1.1"
status: draft
producer: architect
timestamp: 2026-03-29T14:39:00
phase: 1d
inputs: [domain-spec/L2-INDEX.md, specs/prd.md]
traces_to: ARCH-INDEX.md
revision_note: "ADV-P1-005 — Normalized layering model. Redefined forge-core as domain kernel (shared types + protocol adapter). Removed forge-discovery→forge-core coupling by extracting shared types into forge-core's pure domain layer. Redrawn DAG to match dependency table and downward-flow rule."
---

# Dependency Graph

**Direction rule:** Dependencies flow DOWNWARD. Higher-level modules depend on lower-level. Core depends on nothing internal (only rmcp and serialization crates).

## Layering Model

The crate graph is organized into four layers. Every dependency points strictly downward — no lateral or upward references.

| Layer | Crates | Role |
|-------|--------|------|
| **L0 — Domain Kernel** | forge-core | Shared domain types (ServerEntry, MCP message types, error taxonomy) + protocol adapter over rmcp. All crates above depend on forge-core for type definitions. |
| **L1 — Foundation** | forge-discovery, forge-traffic, forge-health | Each depends only on forge-core. Produces domain data (config registry, message buffers, metrics). |
| **L2 — Analysis** | forge-security, forge-config, forge-conformance | Depends on L0 and selectively on L1. Consumes domain data for analysis, diffing, or protocol testing. |
| **L3 — Presentation & Orchestration** | forge-tui, forge-daemon | Depends on lower layers. Renders data or manages connection lifecycle. |
| **L4 — Entry Point** | forge-mcp (binary) | Depends on everything. Thin CLI dispatch. |

### Why forge-core is the domain kernel (not just a protocol adapter)

ADV-P1-005 identified that forge-discovery depended on forge-core solely for `ServerEntry` and shared MCP types — coupling a discovery/config concern to a "protocol adapter." The resolution: forge-core is explicitly redefined as the **domain kernel crate** containing:

1. **Shared domain types** — `ServerEntry`, `ServerTransport`, capability enums, session ID types. These are the lingua franca of the workspace.
2. **MCP message types** — `McpMessage`, `JsonRpcRequest`, `JsonRpcResponse`, direction enum.
3. **Protocol adapter** — rmcp `Peer` wrapper, connection state machine, capability negotiation.

This makes forge-core's dual role explicit: it owns both the shared type vocabulary and the protocol implementation. All other crates depend on forge-core for types, and only forge-daemon/forge-conformance use its protocol adapter functionality.

## ASCII DAG

```
                    forge-mcp (binary)           ← L4
                   /    |    |    \
                  /     |    |     \
          forge-tui  forge-daemon   \            ← L3
           /  |  \      |           \
          /   |   \     |            \
 forge-traffic | forge-health    forge-conformance  ← L2/L1
          \   |   /                   |
           \  |  /                   /
        forge-security  forge-config            ← L2
              \            /
               \          /
            forge-discovery                     ← L1
                    \
                 forge-core                     ← L0
                     |
                   [rmcp]
```

**Reading the DAG:** Arrows point downward (dependency direction). Every crate depends only on crates below it in the diagram. forge-config depends on forge-discovery (L1) and forge-core (L0) — both strictly below L2.

## Dependency Table

| Crate | Layer | Depends On | Reason |
|-------|-------|-----------|--------|
| forge-core | L0 | rmcp, serde, serde_json | Domain kernel: shared types (ServerEntry, McpMessage, error taxonomy) + protocol adapter wrapping rmcp Peer API |
| forge-discovery | L1 | forge-core, serde_json, dirs | Config parsing; uses forge-core domain types (ServerEntry, ServerTransport) for unified registry |
| forge-traffic | L1 | forge-core | Captures forge-core McpMessage types into ring buffer |
| forge-health | L1 | forge-core | Metrics derived from forge-core McpMessage types |
| forge-security | L2 | forge-core, forge-traffic | Analyzes messages from traffic capture; references forge-core types for finding model |
| forge-config | L2 | forge-discovery, forge-core | Needs parsed config registry (from forge-discovery) + forge-core types for schema diff |
| forge-conformance | L2 | forge-core | Sends protocol methods via forge-core adapter, validates responses |
| forge-tui | L3 | forge-core, forge-traffic, forge-health, forge-security, forge-discovery | Renders all subsystem data in TUI panels |
| forge-daemon | L3 | forge-core, forge-discovery | Manages connections (via forge-core protocol adapter) and session pooling |
| forge-mcp | L4 | ALL crates | Binary entry point; clap CLI dispatch to subsystems |

## Dependency Direction Rules

1. **forge-core (L0) depends on nothing internal** — only rmcp and standard serialization crates. It is the domain kernel.
2. **L1 crates depend only on L0** — forge-discovery, forge-traffic, forge-health each depend only on forge-core.
3. **L2 crates depend on L0 and selectively L1** — forge-security depends on forge-core + forge-traffic; forge-config depends on forge-core + forge-discovery; forge-conformance depends only on forge-core.
4. **L3 crates depend on L0–L2** — forge-tui renders data from L1/L2 modules; forge-daemon uses L0 protocol adapter + L1 discovery.
5. **L4 (binary) depends on everything** — entry point only.
6. **No lateral dependencies within a layer** — L1 crates never depend on each other; L2 crates never depend on each other.
7. **No upward dependencies** — a lower-layer crate never depends on a higher-layer crate.
8. **No circular dependencies** — DAG is strictly acyclic.

## ADR: forge-core as Domain Kernel (ADV-P1-005)

**Decision:** Redefine forge-core as the domain kernel crate rather than a pure protocol adapter.

**Context:** The original dependency graph described forge-core as "Protocol adapter; wraps rmcp Peer API" but then had forge-discovery depend on it for shared types like `ServerEntry`. This created an inconsistency: a config-parsing crate depending on a "protocol adapter" for non-protocol types. The ASCII DAG also placed forge-config below forge-discovery, implying the wrong flow direction.

**Alternatives considered:**
1. **Extract a `forge-types` crate** — Adds a crate for ~200 lines of type definitions. Rejected: overhead not justified for a 10-crate workspace where forge-core is already the natural bottom.
2. **Move ServerEntry to forge-discovery** — Then forge-daemon and forge-tui would need to depend on forge-discovery for types, creating unnecessary coupling.
3. **Redefine forge-core as domain kernel** — Chosen. Makes the dual role explicit without adding crates.

**Consequences:** forge-core's module decomposition description must reflect both roles (domain types + protocol adapter). The purity boundary within forge-core remains: types are pure, protocol adapter is effectful shell.
