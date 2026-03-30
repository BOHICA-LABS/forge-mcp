# Demo Evidence — STORY-027: Transparent JSON-RPC Message Capture

- **Story ID:** STORY-027
- **Date/Time Recorded:** 2026-03-30T17:37 CDT (2026-03-30T22:37 UTC)
- **Branch/Worktree:** `.worktrees/STORY-027`
- **Recorded By:** demo-recorder subagent

---

## Summary

| AC | Test Name | Result |
|----|-----------|--------|
| AC-001 | `test_BC_4_09_001_all_messages_captured` | ✅ PASS |
| AC-002 | `test_BC_4_09_001_payload_content_integrity` | ✅ PASS |
| AC-003 | `test_BC_4_09_001_multiple_consumers` | ✅ PASS |
| AC-004 | `test_capture_overhead_benchmark` | ✅ PASS |
| EC-001 | `test_capture_no_subscribers` | ✅ PASS |
| EC-002 | `test_capture_large_payload` | ✅ PASS |
| EC-003 | `test_capture_channel_lagged` | ✅ PASS |

**Total: 7/7 tests passed, 0 failed.**

---

## AC-001 — All Messages Captured (traces to BC-4.09.001 postcondition)

**Test:** `test_BC_4_09_001_all_messages_captured`  
**Command:** `cargo test --test capture_tests test_BC_4_09_001_all_messages_captured -- --nocapture`  
**Result:** ✅ PASS  
**Evidence file:** `AC-001-all-messages-captured.txt`

```
   Finished `test` profile [unoptimized + debuginfo] target(s) in 0.36s
    Running tests/capture_tests.rs (target/debug/deps/capture_tests-1edb0f36ea687908)

running 1 test
test test_BC_4_09_001_all_messages_captured ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s
```

---

## AC-002 — Content Integrity (proptest, VP-006)

**Test:** `test_BC_4_09_001_payload_content_integrity`  
**Command:** `cargo test --test capture_tests test_BC_4_09_001_payload_content_integrity -- --nocapture`  
**Result:** ✅ PASS  
**Evidence file:** `AC-002-payload-content-integrity.txt`

```
   Finished `test` profile [unoptimized + debuginfo] target(s) in 0.41s
    Running tests/capture_tests.rs (target/debug/deps/capture_tests-1edb0f36ea687908)

running 1 test
proptest: FileFailurePersistence::SourceParallel set, but no source file known
test test_BC_4_09_001_payload_content_integrity ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.12s
```

> Note: `proptest: FileFailurePersistence::SourceParallel set, but no source file known` is expected in test binary context (no .rs source path available at runtime). The proptest ran successfully.

---

## AC-003 — Multiple Consumers (Tokio broadcast channel)

**Test:** `test_BC_4_09_001_multiple_consumers`  
**Command:** `cargo test --test capture_tests test_BC_4_09_001_multiple_consumers -- --nocapture`  
**Result:** ✅ PASS  
**Evidence file:** `AC-003-multiple-consumers.txt`

```
   Finished `test` profile [unoptimized + debuginfo] target(s) in 0.36s
    Running tests/capture_tests.rs (target/debug/deps/capture_tests-1edb0f36ea687908)

running 1 test
test test_BC_4_09_001_multiple_consumers ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s
```

---

## AC-004 — Performance Overhead < 1% (NFR-004)

**Test:** `test_capture_overhead_benchmark`  
**Command:** `cargo test --test capture_tests test_capture_overhead_benchmark -- --nocapture`  
**Result:** ✅ PASS  
**Evidence file:** `AC-004-capture-overhead-benchmark.txt`

```
   Finished `test` profile [unoptimized + debuginfo] target(s) in 0.42s
    Running tests/capture_tests.rs (target/debug/deps/capture_tests-1edb0f36ea687908)

running 1 test
test test_capture_overhead_benchmark ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s
```

---

## Edge Cases (EC-001, EC-002, EC-003) — Full Suite

**Command:** `cargo test --test capture_tests -- --nocapture`  
**Result:** ✅ ALL PASS (7/7)  
**Evidence file:** `full-suite.txt`

| Edge Case | Test | Result |
|-----------|------|--------|
| EC-001 (no subscribers) | `test_capture_no_subscribers` | ✅ PASS |
| EC-002 (large payload >1MB) | `test_capture_large_payload` | ✅ PASS |
| EC-003 (channel full/lagged) | `test_capture_channel_lagged` | ✅ PASS |

```
   Finished `test` profile [unoptimized + debuginfo] target(s) in 0.37s
    Running tests/capture_tests.rs (target/debug/deps/capture_tests-1edb0f36ea687908)

running 7 tests
proptest: FileFailurePersistence::SourceParallel set, but no source file known
test test_capture_no_subscribers ... ok
test test_BC_4_09_001_multiple_consumers ... ok
test test_BC_4_09_001_all_messages_captured ... ok
test test_capture_channel_lagged ... ok
test test_capture_large_payload ... ok
test test_capture_overhead_benchmark ... ok
test test_BC_4_09_001_payload_content_integrity ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s
```

---

## Coverage Mapping

| Acceptance Criterion | Behavioral Contract | Test | Evidence File | Status |
|---------------------|---------------------|------|--------------|--------|
| AC-001 | BC-4.09.001 (transparent capture) | `test_BC_4_09_001_all_messages_captured` | `AC-001-all-messages-captured.txt` | ✅ PASS |
| AC-002 | VP-006 (content integrity) | `test_BC_4_09_001_payload_content_integrity` | `AC-002-payload-content-integrity.txt` | ✅ PASS |
| AC-003 | BC-4.09.001 (event bus) | `test_BC_4_09_001_multiple_consumers` | `AC-003-multiple-consumers.txt` | ✅ PASS |
| AC-004 | NFR-004 (<1% overhead) | `test_capture_overhead_benchmark` | `AC-004-capture-overhead-benchmark.txt` | ✅ PASS |
| EC-001 | No subscribers edge case | `test_capture_no_subscribers` | `full-suite.txt` | ✅ PASS |
| EC-002 | Large payload edge case | `test_capture_large_payload` | `full-suite.txt` | ✅ PASS |
| EC-003 | Channel full/lagged edge case | `test_capture_channel_lagged` | `full-suite.txt` | ✅ PASS |

**Verdict: STORY-027 demo evidence COMPLETE. All 4 ACs + 3 edge cases verified green.**
