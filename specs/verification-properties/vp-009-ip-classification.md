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
source_bc: [BC-7.16.002]
module: forge-security
proof_method: kani
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

# VP-009: IP Classification Correctness

## Property Statement

For any IPv4 address (represented as `u32`), `classify_ip(addr)` MUST correctly identify:

1. **RFC1918 private ranges:** `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16`
2. **Link-local:** `169.254.0.0/16`
3. **Cloud metadata:** `169.254.169.254` specifically
4. **Localhost:** `127.0.0.0/8`

Classification MUST have **zero false negatives** for these ranges. Every address in these ranges MUST be classified correctly.

This is a **classification correctness** property: the IP classifier partitions all IPv4 addresses into the correct network category with exhaustive coverage.

## Source Contract

- **BC-7.16.002** — Network-level security analysis: URL/IP classification to detect SSRF, internal network access, and cloud metadata endpoint targeting.
- **R-004** — Risk: SSRF via MCP tool URLs. Mitigation depends on correct IP classification.

## Proof Method

| Aspect | Detail |
|--------|--------|
| Method | Model checking (Kani) |
| Tool | Kani Rust Verifier |
| Input space | Full u32 range (all possible IPv4 addresses) |
| Bound | No loop — single function call per proof invocation |
| Expected time | Minutes (Kani uses bit-level reasoning over u32) |

## Harness Skeleton

```rust
#[kani::proof]
fn verify_ip_classification() {
    let addr: u32 = kani::any();
    let class = classify_ip(addr);

    let a = (addr >> 24) as u8;
    let b = (addr >> 16) as u8;

    // RFC1918: 10.0.0.0/8
    if a == 10 {
        assert!(class.is_private());
    }
    // RFC1918: 172.16.0.0/12
    if a == 172 && (b >= 16 && b <= 31) {
        assert!(class.is_private());
    }
    // RFC1918: 192.168.0.0/16
    if a == 192 && b == 168 {
        assert!(class.is_private());
    }
    // Link-local: 169.254.0.0/16
    if a == 169 && b == 254 {
        assert!(class.is_link_local());
    }
    // Metadata: 169.254.169.254
    if addr == 0xA9FEA9FE {
        assert!(class.is_metadata());
    }
    // Localhost: 127.0.0.0/8
    if a == 127 {
        assert!(class.is_localhost());
    }
}
```

### Classification Hierarchy

Note that `169.254.169.254` is both link-local AND metadata. The classifier must return `is_metadata() == true` for this specific address, which is a stricter classification. The harness validates both:

```rust
#[kani::proof]
fn verify_metadata_is_also_link_local() {
    let metadata_addr: u32 = 0xA9FEA9FE; // 169.254.169.254
    let class = classify_ip(metadata_addr);
    assert!(class.is_metadata());
    assert!(class.is_link_local()); // metadata ⊂ link-local
}
```

### Boundary Values

The harness implicitly covers boundary addresses through Kani's exhaustive symbolic exploration:
- `10.0.0.0` (first in 10/8)
- `10.255.255.255` (last in 10/8)
- `172.15.255.255` (just outside 172.16/12)
- `172.16.0.0` (first in 172.16/12)
- `172.31.255.255` (last in 172.16/12)
- `172.32.0.0` (just outside 172.16/12)

## Feasibility Assessment

| Factor | Assessment |
|--------|-----------|
| Input space | u32 — Kani explores symbolically via bit-level SAT/SMT |
| Complexity | Low — classification is pure bitwise arithmetic |
| Tool support | Excellent — Kani excels at bit-manipulation proofs |
| Expected time | Minutes |
| Verdict | **FEASIBLE** |

## Lifecycle

| Field | Value |
|-------|-------|
| Introduced | v0.1.0 |
| Modified | — |
| Deprecated | — |
| Retired | — |
