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
source_bc: [BC-2.04.001]
module: forge-core
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

# VP-001: JSON-RPC Message Parsing Never Panics

## Property Statement

For any byte sequence `input`, calling `parse_jsonrpc(input)` MUST either return `Ok(McpMessage)` or `Err(ParseError)`. It MUST NOT panic, abort, or enter an infinite loop.

This is a **total function** property: every possible input produces a defined output within bounded time.

## Source Contract

- **BC-2.04.001** — Bidirectional capability negotiation requires robust parsing of all incoming JSON-RPC messages. Malformed input from any MCP server must never crash the proxy.

## Proof Method

| Aspect | Detail |
|--------|--------|
| Method | Fuzz testing (cargo-fuzz / libFuzzer) |
| Tool | `cargo-fuzz` with `libfuzzer-sys` |
| Input space | Arbitrary byte sequences up to 16MB (max message size) |
| Coverage target | All code paths in `parse_jsonrpc` |
| Expected time | Seconds per iteration |
| Corpus | Seed with valid JSON-RPC samples + edge cases (empty, huge, nested) |

## Harness Skeleton

```rust
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = forge_core::parse_jsonrpc(data);
    // No panic = success
});
```

### Seed Corpus

Place in `fuzz/corpus/parse_jsonrpc/`:
- `valid_request.json` — well-formed JSON-RPC 2.0 request
- `valid_response.json` — well-formed JSON-RPC 2.0 response
- `valid_notification.json` — well-formed notification (no id)
- `empty.bin` — zero bytes
- `max_depth.json` — deeply nested JSON (1000 levels)
- `huge_string.json` — 16MB string value
- `binary_garbage.bin` — random bytes
- `truncated_utf8.bin` — valid JSON prefix with truncated UTF-8 sequence

## Feasibility Assessment

| Factor | Assessment |
|--------|-----------|
| Input space | Arbitrary bytes up to 16MB — well-suited for fuzzing |
| Complexity | Low — parsing is self-contained with no external dependencies |
| Tool support | Excellent — cargo-fuzz is mature and well-integrated with Rust |
| Expected time | Milliseconds per iteration; minutes to hours for deep coverage |
| Verdict | **FEASIBLE** |

## Lifecycle

| Field | Value |
|-------|-------|
| Introduced | v0.1.0 |
| Modified | — |
| Deprecated | — |
| Retired | — |
