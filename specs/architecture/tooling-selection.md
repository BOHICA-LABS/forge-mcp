---
document_type: architecture-section
level: L3
section: tooling-selection
version: "1.0"
status: draft
producer: architect
timestamp: 2026-03-29T14:05:00
phase: 1b
inputs: [domain-spec/L2-INDEX.md, specs/prd.md]
traces_to: ARCH-INDEX.md
---

# Tooling Selection

## Verification Toolchain

| Tool | Version | Purpose | Target Modules |
|------|---------|---------|---------------|
| **Kani** | latest stable | Model checking for bounded proofs | forge-core, forge-discovery, forge-traffic, forge-health, forge-security |
| **proptest** | 1.x | Property-based testing with shrinking | All modules with pure cores |
| **cargo-fuzz** | latest | Coverage-guided fuzzing | forge-core (JSON-RPC), forge-discovery (JSON), forge-traffic (regex), forge-security (payloads) |
| **cargo-mutants** | latest | Mutation testing for kill rate validation | All modules per criticality tier |
| **cargo-audit** | latest | Dependency vulnerability scanning | Workspace-wide |
| **cargo-deny** | latest | License and duplicate dependency checking | Workspace-wide |
| **clippy** | nightly | Lint enforcement (deny warnings) | All modules |

## Kani Configuration

Kani proofs require bounded input types. Strategy:

- Use `kani::any()` for primitive types (u8, u16, i64, bool, etc.)
- Bound string inputs to max 256 bytes for config parsing proofs
- Bound ring buffer size to 1024 entries for buffer proofs
- Bound tool list size to 100 entries for pagination proofs
- Use `kani::assume()` to enforce preconditions from BCs

## proptest Configuration

```rust
// proptest.toml
[default]
cases = 1000
max_shrink_iters = 10000
```

Strategy generators needed:

- `arb_mcp_message()` — valid JSON-RPC messages with all variants
- `arb_server_entry()` — valid server entries across all 4 editor schemas
- `arb_traffic_filter()` — valid filter combinations
- `arb_metric_series()` — time series of health metrics
- `arb_tool_set()` — sets of tools for schema drift testing

## Fuzz Targets

| Target | Input | Module | Goal |
|--------|-------|--------|------|
| `fuzz_jsonrpc_parse` | Arbitrary bytes | forge-core | No panics on malformed input |
| `fuzz_config_parse` | Arbitrary JSON | forge-discovery | No panics, graceful errors |
| `fuzz_filter_regex` | Arbitrary strings | forge-traffic | No regex DoS |
| `fuzz_security_analyze` | Arbitrary McpMessage | forge-security | No panics in rule engine |

## Mutation Testing

cargo-mutants targets per criticality tier:

- **CRITICAL modules:** ≥95% kill rate required
- **HIGH modules:** ≥90% kill rate required
- **MEDIUM modules:** ≥80% kill rate required
- **LOW modules:** ≥70% kill rate required
