---
document_type: gene-transfusion-assessment
level: L3
version: "1.0"
status: draft
producer: architect
timestamp: 2026-03-29T14:05:00
phase: 1b
inputs: [specs/architecture/module-decomposition.md]
traces_to: specs/architecture/ARCH-INDEX.md
---

# Gene Transfusion Assessment: Forge MCP

## Result: NO GENE TRANSFUSION CANDIDATES

After reviewing all 10 modules, no module has a suitable reference implementation for cross-language porting:

| Module | Algorithm Complexity | Reference Exists? | Assessment | Decision |
|--------|---------------------|-------------------|-----------|----------|
| forge-core | Medium (protocol adapter) | Yes — rmcp IS the reference | We're already using rmcp directly; no translation needed | SKIP |
| forge-discovery | Low (JSON parsing + path resolution) | Many JSON parsers exist | Too simple for translation overhead; standard serde_json usage | SKIP |
| forge-daemon | Medium (session pooling) | Various connection pool implementations | Tokio ecosystem has mature solutions; no cross-language benefit | SKIP |
| forge-tui | Medium (terminal UI) | ratatui IS the reference | We're using ratatui directly; TUI logic is framework-specific | SKIP |
| forge-traffic | Low-Medium (ring buffer + filtering) | Ring buffer implementations in every language | Too simple; standard data structure, trivial to implement in Rust | SKIP |
| forge-health | Low (statistical calculations) | Histogram implementations everywhere | Standard math; HDR Histogram crate available if needed | SKIP |
| forge-security | Medium (pattern matching rules) | No equivalent MCP security scanner | No reference exists — this is novel functionality | SKIP |
| forge-conformance | Medium (spec assertions) | MCP inspector (TypeScript) | Different architecture (interactive vs batch); translation not beneficial | SKIP |
| forge-config | Low (diffing) | Many diff libraries | Too simple; standard comparison logic | SKIP |
| forge-mcp (binary) | Low (CLI glue) | N/A | Clap framework handles this | SKIP |

## Justification

1. **rmcp handles the hard part** — The most complex cross-language candidate would be the MCP protocol implementation, but rmcp already provides this in Rust. No translation needed.
2. **No equivalent security scanner** — forge-security is novel functionality; there's no mature reference implementation in any language to translate from.
3. **Framework-specific UI** — forge-tui is ratatui-specific; translating from another TUI framework (Python's textual, Go's bubbletea) would lose more than it gains.
4. **Simple algorithms** — The remaining modules implement standard patterns (ring buffers, JSON parsing, diffing) that are simpler to write fresh in Rust with TDD than to translate.

## Recommendation

All modules will be implemented from scratch via standard TDD. No Semport-related steps needed in Phase 2 and beyond.
