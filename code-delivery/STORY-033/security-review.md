---
story: STORY-033
title: "Security Review — Passive Latency & Throughput Metric Collection"
reviewer: security-reviewer
date: 2026-03-31
verdict: APPROVE
total_findings: 4
critical: 0
high: 0
medium: 2
low: 2
files_reviewed: 5
---

# Security Review: STORY-033 — Passive Latency & Throughput Metric Collection

## Scope

Reviewed the diff between `develop` and `feature/STORY-033` (4 commits, 5 changed files).

### Files Reviewed

| # | File | Status | Lines Changed |
|---|------|--------|---------------|
| 1 | `crates/forge-health/Cargo.toml` | Modified | +5 / -1 |
| 2 | `crates/forge-health/src/latency.rs` | Added | +83 |
| 3 | `crates/forge-health/src/throughput.rs` | Added | +62 |
| 4 | `crates/forge-health/src/snapshot.rs` | Added | +74 |
| 5 | `crates/forge-health/src/lib.rs` | Modified | +9 / -1 |

Test file reviewed (not counted as production code):
- `crates/forge-health/tests/metric_tests.rs` — Added (+196)

---

## Attack Surface Assessment

This PR adds **passive, read-only metric collection** within an internal library crate (`forge-health`). Key characteristics:

- **No external input processing** — collectors receive `f64` values and `Instant` timestamps from internal callers only
- **No network I/O** — no sockets, HTTP handlers, or file I/O
- **No serialization of untrusted data** — `serde` derives exist at crate level but `MetricSnapshot` does not derive `Deserialize`
- **No authentication/authorization surface** — internal library with no API boundary
- **No command execution** — pure computation crate
- **No unsafe code** — all safe Rust

**Overall risk: LOW.** This is a metrics aggregation library with no direct exposure to untrusted input.

---

## Findings

### SEC-001: `expect()` panics on histogram `record()` could propagate to callers
- **Severity:** MEDIUM
- **CWE:** CWE-248 (Uncaught Exception)
- **OWASP:** N/A (internal library)
- **File:** `crates/forge-health/src/latency.rs`
- **Lines:** 26, 41
- **Attack Vector:** If a caller passes an extremely large `latency_ms` value (e.g., `f64::MAX` or values exceeding the histogram's 3,600,000 ms upper bound after rounding), `self.histogram.record(value).expect(...)` at line 41 will panic. While the `.max(1)` clamp handles the lower bound, there is no upper-bound clamp. A latency value of `3_600_001.0` or higher would cause `record()` to return `Err(RecordError)`, triggering the `expect()` and crashing the process.
- **Evidence:**
  ```rust
  // latency.rs:37-41
  pub fn record_latency(&mut self, latency_ms: f64) {
      let value = (latency_ms.round() as u64).max(1);
      self.histogram
          .record(value)
          .expect("latency value within histogram bounds");
  }
  ```
  The histogram is created with `new_with_bounds(1, 3_600_000, 3)` — any value > 3,600,000 causes an error.
- **Impact:** Process crash (denial of service). In a daemon context, a single outlier latency measurement exceeding 1 hour would crash the entire forge-daemon process.
- **Proposed Mitigation:** Clamp the upper bound before recording:
  ```rust
  pub fn record_latency(&mut self, latency_ms: f64) {
      let value = (latency_ms.round() as u64).clamp(1, 3_600_000);
      self.histogram
          .record(value)
          .expect("latency value within clamped histogram bounds");
  }
  ```
  Alternatively, use `let _ = self.histogram.record(value);` to silently drop out-of-range values and log a warning.

---

### SEC-002: `expect()` panic during histogram construction is non-recoverable
- **Severity:** LOW
- **CWE:** CWE-248 (Uncaught Exception)
- **OWASP:** N/A (internal library)
- **File:** `crates/forge-health/src/latency.rs`
- **Line:** 26
- **Attack Vector:** `Histogram::new_with_bounds(1, 3_600_000, 3).expect(...)` panics if the histogram library cannot allocate or the bounds are invalid. These bounds are hardcoded constants, so this is not exploitable in practice.
- **Evidence:**
  ```rust
  // latency.rs:25-26
  let histogram = Histogram::<u64>::new_with_bounds(1, 3_600_000, 3)
      .expect("valid histogram bounds: 1..3_600_000 with 3 sig figs");
  ```
- **Impact:** Process crash during construction. Since bounds are compile-time constants, this will either always succeed or always fail — it is deterministic and will be caught during testing.
- **Proposed Mitigation:** Acceptable as-is for a library constructor with constant bounds. The `expect()` message documents the invariant. No change required, but consider converting to `Result` propagation if this code will be called in error-recovery contexts.

---

### SEC-003: Unbounded `VecDeque` growth in `ThroughputCollector` under sustained load
- **Severity:** MEDIUM
- **CWE:** CWE-770 (Allocation of Resources Without Limits or Throttling)
- **OWASP:** N/A (internal library, but relates to A05:2021 Security Misconfiguration for resource exhaustion)
- **File:** `crates/forge-health/src/throughput.rs`
- **Lines:** 34-38
- **Attack Vector:** `record_message()` pushes timestamps into an unbounded `VecDeque`. Eviction only occurs for timestamps older than `window`. Under sustained high-throughput conditions (e.g., 100K msg/sec with a 60-second window), the VecDeque could hold 6 million entries (~48 MB for `Instant` on most platforms). While not a classical "attack," this represents an unbounded memory growth proportional to message rate × window size, with no cap.
- **Evidence:**
  ```rust
  // throughput.rs:34-38
  pub fn record_message(&mut self) {
      let now = Instant::now();
      self.timestamps.push_back(now);
      self.evict_stale(now);
  }
  ```
  No `max_capacity` or sampling mechanism limits growth within the window.
- **Impact:** Memory exhaustion under extreme throughput. For typical MCP traffic (hundreds to low thousands of messages/sec), this is unlikely to be an issue. For adversarial or misconfigured environments, this could consume significant memory.
- **Proposed Mitigation:** Consider adding a `max_entries` parameter that caps the VecDeque size and switches to sampling when exceeded. Alternatively, document the expected memory usage: `sizeof(Instant) × max_msg_rate × window_secs`. For now, this is acceptable given the expected MCP traffic volumes, but should be revisited for STORY-034/036 when collectors are wired to real traffic.

---

### SEC-004: `f64` NaN/Infinity handling in `record_latency`
- **Severity:** LOW
- **CWE:** CWE-20 (Improper Input Validation)
- **OWASP:** N/A (internal library)
- **File:** `crates/forge-health/src/latency.rs`
- **Lines:** 37-41
- **Attack Vector:** If `latency_ms` is `f64::NAN` or `f64::INFINITY`, the expression `latency_ms.round() as u64` produces `0` for NaN (on most platforms; technically undefined in Rust prior to saturating casts) and `u64::MAX` for infinity, which after `.max(1)` becomes `u64::MAX`. The `u64::MAX` value exceeds the histogram's upper bound of 3,600,000, triggering the `expect()` panic from SEC-001.
- **Evidence:**
  ```rust
  // latency.rs:37-38
  let value = (latency_ms.round() as u64).max(1);
  ```
  `f64::NAN.round() as u64` = 0 → clamped to 1 (benign, records 1ms)
  `f64::INFINITY.round() as u64` = `u64::MAX` → exceeds histogram bounds → panic
- **Impact:** Same as SEC-001 — process crash if infinity reaches this code path.
- **Proposed Mitigation:** Add a guard for non-finite values:
  ```rust
  pub fn record_latency(&mut self, latency_ms: f64) {
      if !latency_ms.is_finite() || latency_ms < 0.0 {
          tracing::warn!(latency_ms, "dropping non-finite or negative latency");
          return;
      }
      let value = (latency_ms.round() as u64).clamp(1, 3_600_000);
      // ...
  }
  ```

---

## Dependency Analysis

| Dependency | Version | Type | CVE/Advisory Status |
|-----------|---------|------|---------------------|
| `hdrhistogram` | 7 | runtime | ✅ CLEAN — no advisories on crates.io or RustSec |
| `proptest` | 1 | dev-only | ✅ CLEAN — dev dependency, not shipped in production binary |

**Dependency change:** `forge-traffic` was replaced with `forge-core` as the L1 dependency. This is an internal workspace crate swap — no supply chain impact.

---

## OWASP Top 10 Assessment

| Category | Applicable? | Finding |
|----------|-------------|---------|
| A01:2021 Broken Access Control | ❌ No | No auth/authz surface |
| A02:2021 Cryptographic Failures | ❌ No | No crypto operations |
| A03:2021 Injection | ❌ No | No user input processing, no shell/SQL/code execution |
| A04:2021 Insecure Design | ❌ No | Design is sound for passive metric collection |
| A05:2021 Security Misconfiguration | ⚠️ Minor | SEC-003: unbounded VecDeque growth under extreme load |
| A06:2021 Vulnerable Components | ❌ No | Dependencies are clean |
| A07:2021 Auth Failures | ❌ No | No authentication surface |
| A08:2021 Data Integrity Failures | ❌ No | No deserialization of untrusted data |
| A09:2021 Logging/Monitoring Failures | ❌ No | Not in scope for this story |
| A10:2021 SSRF | ❌ No | No outbound requests |

---

## Summary

| Severity | Count | Finding IDs |
|----------|-------|-------------|
| CRITICAL | 0 | — |
| HIGH | 0 | — |
| MEDIUM | 2 | SEC-001, SEC-003 |
| LOW | 2 | SEC-002, SEC-004 |

### Verdict: **APPROVE** ✅

All findings are MEDIUM or LOW severity. No CRITICAL or HIGH findings.

**Rationale for approval:**
1. This is an internal library crate with **no external attack surface** — it processes only values from trusted internal callers
2. SEC-001 and SEC-004 (panic on out-of-range values) are mitigable with a simple `.clamp()` but are not exploitable from outside the process
3. SEC-003 (unbounded growth) is bounded by realistic MCP traffic volumes and the window size parameter
4. The code uses **no unsafe Rust**, no raw pointers, no FFI, no file I/O, no network I/O
5. The property-based tests (proptest) add strong coverage for edge cases

**Recommended follow-ups (non-blocking):**
- Apply the `.clamp(1, 3_600_000)` fix from SEC-001 before this collector handles real traffic
- Add NaN/Infinity guards from SEC-004 as defensive programming
- Consider documenting memory bounds for ThroughputCollector (SEC-003)
