---
document_type: verification-property
level: L4
version: "1.0"
status: draft
producer: architect
timestamp: 2026-03-29T14:05:00
phase: 1b
inputs: [specs/prd.md, specs/architecture/ARCH-INDEX.md]
input-hash: ""
traces_to: specs/prd.md
source_bc: [BC-1.01.002]
module: forge-discovery
proof_method: fuzz
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: v0.1.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-004: Config Parsing Never Panics

## Property Statement

For any byte sequence `input` representing a potential config file, `parse_config(input, schema)` for both `McpServers` and `Servers` schemas MUST return `Ok(Vec<ServerEntry>)` or `Err(ConfigError)`. It MUST NOT panic, abort, or enter an infinite loop.

This is a **total function** property applied to the dual-schema configuration parser.

## Source Contract

- **BC-1.01.002** — Dual-schema config parsing: the discovery module must accept both `mcpServers` and `servers` JSON schemas. Malformed config files from user environments must never crash the application.

## Proof Method

| Aspect | Detail |
|--------|--------|
| Method | Fuzz testing (cargo-fuzz / libFuzzer) |
| Tool | `cargo-fuzz` with `libfuzzer-sys` |
| Input space | Arbitrary byte sequences (representing config file content) |
| Coverage target | All code paths in `parse_config` for both schema variants |
| Iteration time | Milliseconds |
| Corpus | Seed with valid config samples for both schemas + edge cases |

## Harness Skeleton

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = forge_discovery::parse_config(data, ConfigSchema::McpServers);
    let _ = forge_discovery::parse_config(data, ConfigSchema::Servers);
});
```

### Seed Corpus

Place in `fuzz/corpus/parse_config/`:
- `valid_mcp_servers.json` — well-formed `mcpServers` schema config
- `valid_servers.json` — well-formed `servers` schema config
- `empty.json` — `{}`
- `empty_array.json` — `[]`
- `nested_deep.json` — deeply nested JSON object
- `mixed_types.json` — valid JSON with unexpected types in server fields
- `utf8_keys.json` — server names with Unicode characters
- `trailing_comma.json` — JSON with trailing commas (invalid but common)

## Feasibility Assessment

| Factor | Assessment |
|--------|-----------|
| Input space | Arbitrary bytes — same pattern as VP-001 |
| Complexity | Low — config parsing is self-contained, no network I/O |
| Tool support | Excellent — same cargo-fuzz infrastructure as VP-001 |
| Time per iteration | Milliseconds |
| Expected duration | Minutes to hours for deep coverage |
| Verdict | **FEASIBLE** |

## Lifecycle

| Field | Value |
|-------|-------|
| Introduced | v0.1.0 |
| Modified | — |
| Deprecated | — |
| Retired | — |
