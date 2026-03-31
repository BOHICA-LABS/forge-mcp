//! Integration tests for `RingBuffer<T>` — STORY-029 | BC-4.09.003 | VP-005
//!
//! RED GATE: All tests call `todo!()` stubs and must FAIL until Phase 3
//! implementation is complete. Do not merge until every test passes.

// Test names follow the BC-based naming convention (test_BC_S_SS_NNN_...) for
// full traceability to behavioral contracts. The uppercase segments are
// intentional — suppress the lint.
#![allow(non_snake_case)]

use forge_traffic::RingBuffer;

// ---------------------------------------------------------------------------
// AC-001 — ring buffer stores messages (BC-4.09.003 postcondition)
// ---------------------------------------------------------------------------

/// AC-001: Messages pushed into the buffer are retained and iterable.
///
/// Creates a buffer with capacity 10 and pushes 5 integer items.
/// Asserts:
///   - `len()` == 5
///   - `iter()` yields all 5 items in insertion order (oldest → newest)
#[test]
fn test_BC_4_09_003_ring_buffer_stores_messages() {
    let mut buf: RingBuffer<i32> = RingBuffer::new(10, usize::MAX);

    for i in 1..=5 {
        let evicted = buf.push(i);
        assert!(
            evicted.is_none(),
            "No eviction expected when buffer has free space; got {:?}",
            evicted
        );
    }

    assert_eq!(buf.len(), 5, "len() should be 5 after 5 pushes");

    let items: Vec<&i32> = buf.iter().collect();
    assert_eq!(items.len(), 5, "iter() should yield 5 items");

    let values: Vec<i32> = items.into_iter().copied().collect();
    assert_eq!(
        values,
        vec![1, 2, 3, 4, 5],
        "iter() must return items in insertion (oldest → newest) order"
    );
}

// ---------------------------------------------------------------------------
// AC-002 — FIFO eviction (BC-4.09.003 postcondition, VP-005)
// ---------------------------------------------------------------------------

/// AC-002 / VP-005: When the buffer is full, the oldest item is evicted FIFO.
///
/// Creates a buffer with capacity 3. Pushes A, B, C (fills buffer), then D.
/// Asserts:
///   - `push(D)` returns `Some(A)` (A is the evicted item)
///   - `len()` == 3 after four pushes
///   - `iter()` yields [B, C, D] in order
#[test]
fn test_BC_4_09_003_fifo_eviction() {
    let mut buf: RingBuffer<&str> = RingBuffer::new(3, usize::MAX);

    assert!(buf.push("A").is_none(), "push A should not evict");
    assert!(buf.push("B").is_none(), "push B should not evict");
    assert!(buf.push("C").is_none(), "push C should not evict");

    // Buffer is now full: [A, B, C]
    let evicted = buf.push("D");
    assert_eq!(
        evicted,
        Some("A"),
        "push D must evict the oldest item A (FIFO)"
    );

    assert_eq!(buf.len(), 3, "len() must remain 3 after eviction");

    let items: Vec<&&str> = buf.iter().collect();
    let values: Vec<&str> = items.into_iter().copied().collect();
    assert_eq!(
        values,
        vec!["B", "C", "D"],
        "iter() must yield B, C, D in FIFO order after A was evicted"
    );
}

// ---------------------------------------------------------------------------
// AC-003 — memory bound (BC-4.09.003 invariant, NFR-012)
// ---------------------------------------------------------------------------

/// AC-003 / NFR-012: `memory_usage_bytes()` never exceeds `max_bytes`.
///
/// Creates a buffer with max_bytes=1000 and pushes items until the memory
/// usage would exceed the limit. After every push the reported usage must
/// remain ≤ max_bytes.
#[test]
fn test_BC_4_09_003_memory_bound() {
    const MAX_BYTES: usize = 1000;
    let mut buf: RingBuffer<String> = RingBuffer::new(usize::MAX, MAX_BYTES);

    // Each String "item_NNN" is roughly 8–10 bytes of payload; push enough
    // to fill well past the raw byte limit.
    for i in 0..200u32 {
        buf.push(format!("item_{:04}", i));
        let usage = buf.memory_usage_bytes();
        assert!(
            usage <= MAX_BYTES,
            "memory_usage_bytes() = {} exceeded max_bytes = {} after push {}",
            usage,
            MAX_BYTES,
            i
        );
    }
}

// ---------------------------------------------------------------------------
// AC-004 — eviction warning threshold detection (BC-4.09.003 — E-CAP-001)
// ---------------------------------------------------------------------------

/// AC-004: Buffer detects when it crosses 90 % capacity.
///
/// The test validates the threshold detection logic: when the buffer reaches
/// or exceeds 90 % of its count capacity, a warning channel / flag must be
/// observable. For the stub test we verify the threshold is crossed and that
/// `len() >= 0.9 * capacity()` — the real implementation will emit E-CAP-001.
///
/// NOTE: The test intentionally exercises the push path that triggers
/// threshold detection. Once implemented, this test should be extended to
/// capture the actual tracing/warning event.
#[test]
fn test_BC_4_09_003_eviction_warning_emitted() {
    const CAPACITY: usize = 10;
    let mut buf: RingBuffer<u32> = RingBuffer::new(CAPACITY, usize::MAX);

    // Push exactly 9 items — 90 % of capacity.
    for i in 0..9u32 {
        buf.push(i);
    }

    let threshold = (CAPACITY as f64 * 0.9).floor() as usize;

    assert!(
        buf.len() >= threshold,
        "buffer should be at or above 90% capacity ({} >= {}), got len = {}",
        buf.len(),
        threshold,
        buf.len()
    );

    assert_eq!(
        buf.capacity(),
        CAPACITY,
        "capacity() must return the configured capacity"
    );

    // Push one more to ensure the over-capacity / eviction path also runs
    // without panicking.
    for i in 9..20u32 {
        let _ = buf.push(i);
        assert!(
            buf.len() <= CAPACITY,
            "len() must never exceed capacity after push {}",
            i
        );
    }
}

// ---------------------------------------------------------------------------
// AC-005 — O(1) append at scale (BC-4.09.003 — AD-006)
// ---------------------------------------------------------------------------

/// AC-005: Append is O(1) — verified by exercising 1 000 pushes without panic.
///
/// A true O(1) timing proof would require criterion benchmarks; this test
/// validates the contract at the unit level by confirming that 1 000 pushes on
/// a capacity-100 buffer complete without panic, stack overflow, or allocation
/// blow-up. The Kani proof (VP-005) provides the formal guarantee.
#[test]
fn test_BC_4_09_003_append_is_constant_time() {
    const CAPACITY: usize = 100;
    let mut buf: RingBuffer<u64> = RingBuffer::new(CAPACITY, usize::MAX);

    for i in 0..1_000u64 {
        let _ = buf.push(i);
    }

    // After 1 000 pushes into a capacity-100 buffer the newest 100 items
    // must be retained.
    assert_eq!(buf.len(), CAPACITY, "len() must equal capacity after fill");
    assert!(
        !buf.is_empty(),
        "buffer must not be empty after 1 000 pushes"
    );
    assert!(buf.is_full(), "buffer must be full at capacity");
}

// ---------------------------------------------------------------------------
// EC-001 — zero-capacity buffer
// ---------------------------------------------------------------------------

/// EC-001: A buffer with capacity 0 evicts every item immediately.
///
/// Every push must return `Some(item)` and `len()` must remain 0.
#[test]
fn test_zero_capacity_buffer() {
    let mut buf: RingBuffer<i32> = RingBuffer::new(0, usize::MAX);

    assert_eq!(buf.len(), 0, "zero-capacity buffer starts empty");
    assert!(buf.is_empty(), "is_empty() must be true");

    for i in 0..5 {
        let evicted = buf.push(i);
        assert_eq!(
            evicted,
            Some(i),
            "zero-capacity buffer must immediately evict item {}",
            i
        );
        assert_eq!(
            buf.len(),
            0,
            "len() must remain 0 after eviction of item {}",
            i
        );
    }
}

// ---------------------------------------------------------------------------
// EC — empty buffer operations
// ---------------------------------------------------------------------------

/// Empty buffer: `len()==0`, `is_empty()==true`, `is_full()==false`,
/// `iter()` yields nothing, `memory_usage_bytes()==0`.
#[test]
fn test_empty_buffer_operations() {
    let buf: RingBuffer<String> = RingBuffer::new(10, 4096);

    assert_eq!(buf.len(), 0, "fresh buffer len() must be 0");
    assert!(buf.is_empty(), "fresh buffer must be empty");
    assert!(!buf.is_full(), "fresh buffer must not be full");
    assert_eq!(
        buf.memory_usage_bytes(),
        0,
        "fresh buffer memory_usage_bytes() must be 0"
    );

    let count = buf.iter().count();
    assert_eq!(count, 0, "iter() on empty buffer must yield no items");

    assert_eq!(
        buf.capacity(),
        10,
        "capacity() must return the configured value"
    );
}
