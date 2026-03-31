//! Ring buffer with bounded memory for traffic capture.
//!
//! Provides a generic `RingBuffer<T>` that enforces both a message-count
//! capacity and a byte-size cap. When either limit is exceeded the oldest
//! item is evicted (FIFO). All operations are O(1).
//!
//! STORY-029 | BC-4.09.003 | VP-005 | AD-006 | NFR-012

/// A fixed-capacity, bounded-memory ring buffer.
///
/// Items are stored in FIFO order. When the buffer is full (either by count
/// or by byte size) the oldest item is evicted before the new item is
/// inserted.
///
/// # Type Parameters
/// - `T` — the element type stored in the buffer.
pub struct RingBuffer<T> {
    capacity: usize,
    max_bytes: usize,
    // Implementation fields will be added during Phase 3 implementation.
    _marker: std::marker::PhantomData<T>,
}

impl<T> RingBuffer<T> {
    /// Create a new `RingBuffer` with the given message-count capacity and
    /// maximum byte budget.
    ///
    /// # Arguments
    /// - `capacity`  — maximum number of items to hold before eviction.
    /// - `max_bytes` — maximum total byte size before eviction.
    pub fn new(capacity: usize, max_bytes: usize) -> Self {
        todo!()
    }

    /// Push `item` into the buffer.
    ///
    /// If the buffer is already at capacity (count **or** byte limit), the
    /// oldest item is evicted and returned as `Some(evicted)`. Otherwise
    /// returns `None`.
    pub fn push(&mut self, item: T) -> Option<T> {
        todo!()
    }

    /// Number of items currently stored in the buffer.
    pub fn len(&self) -> usize {
        todo!()
    }

    /// Returns `true` when the buffer contains no items.
    pub fn is_empty(&self) -> bool {
        todo!()
    }

    /// Returns `true` when the buffer is at its count capacity.
    pub fn is_full(&self) -> bool {
        todo!()
    }

    /// Iterate over stored items from oldest to newest without consuming them.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        todo!();
        // Satisfy the return-type requirement — replaced by real impl later.
        #[allow(unreachable_code)]
        std::iter::empty::<&T>()
    }

    /// Estimated total memory used by stored items, in bytes.
    pub fn memory_usage_bytes(&self) -> usize {
        todo!()
    }

    /// The maximum number of items this buffer can hold.
    pub fn capacity(&self) -> usize {
        todo!()
    }
}
