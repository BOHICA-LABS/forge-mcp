//! Ring buffer with bounded memory for traffic capture.
//!
//! Provides a generic `RingBuffer<T>` that enforces both a message-count
//! capacity and a byte-size cap. When either limit is exceeded the oldest
//! item is evicted (FIFO). All operations are O(1).
//!
//! STORY-029 | BC-4.09.003 | VP-005 | AD-006 | NFR-012

use std::collections::VecDeque;

/// A fixed-capacity, bounded-memory ring buffer.
///
/// Items are stored in FIFO order. When the buffer is full (either by count
/// or by byte size) the oldest item is evicted before the new item is
/// inserted.
///
/// # Type Parameters
/// - `T` — the element type stored in the buffer. Must be `Sized` for memory
///   accounting via `std::mem::size_of::<T>()`.
pub struct RingBuffer<T> {
    /// Maximum number of items to hold (0 = immediately evict everything).
    capacity: usize,
    /// Maximum total byte footprint. `0` means no byte limit.
    max_bytes: usize,
    /// Internal storage — O(1) push_back / pop_front.
    items: VecDeque<T>,
    /// Running byte-usage tally.
    usage_bytes: usize,
}

impl<T: Sized> RingBuffer<T> {
    /// Create a new `RingBuffer` with the given message-count capacity and
    /// maximum byte budget.
    ///
    /// # Arguments
    /// - `capacity`  — maximum number of items to hold before eviction.
    ///   Pass `usize::MAX` for an effectively unlimited count cap.
    /// - `max_bytes` — maximum total byte size before eviction.
    ///   Pass `0` to disable the byte limit (count-only mode).
    ///   Pass `usize::MAX` for an effectively unlimited byte cap.
    pub fn new(capacity: usize, max_bytes: usize) -> Self {
        Self {
            capacity,
            max_bytes,
            items: VecDeque::new(),
            usage_bytes: 0,
        }
    }

    /// Byte footprint of a single item (stack size; heap is not tracked).
    #[inline]
    fn item_size() -> usize {
        std::mem::size_of::<T>()
    }

    /// Returns `true` if the byte limit is active and the current usage plus
    /// one new item would exceed it.
    #[inline]
    fn byte_limit_exceeded(&self) -> bool {
        if self.max_bytes == 0 {
            return false; // 0 == no byte limit
        }
        self.usage_bytes + Self::item_size() > self.max_bytes
    }

    /// Push `item` into the buffer.
    ///
    /// If the buffer is already at its count capacity **or** adding the item
    /// would exceed the byte limit, the oldest item is evicted and returned
    /// as `Some(evicted)`. Otherwise returns `None`.
    ///
    /// # Complexity
    /// O(1) — backed by `VecDeque::push_back` / `pop_front`.
    pub fn push(&mut self, item: T) -> Option<T> {
        // Zero-capacity: immediately evict the incoming item itself.
        if self.capacity == 0 {
            return Some(item);
        }

        // Evict the oldest item if either limit is exceeded.
        let must_evict = self.items.len() >= self.capacity || self.byte_limit_exceeded();
        let evicted = if must_evict {
            let old = self.items.pop_front();
            if let Some(ref _old_item) = old {
                self.usage_bytes = self.usage_bytes.saturating_sub(Self::item_size());
            }
            old
        } else {
            None
        };

        self.items.push_back(item);
        self.usage_bytes += Self::item_size();

        evicted
    }

    /// Number of items currently stored in the buffer.
    #[inline]
    pub fn len(&self) -> usize {
        self.items.len()
    }

    /// Returns `true` when the buffer contains no items.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Returns `true` when the buffer has reached its count capacity.
    #[inline]
    pub fn is_full(&self) -> bool {
        self.items.len() >= self.capacity
    }

    /// Iterate over stored items from oldest to newest without consuming them.
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.items.iter()
    }

    /// Estimated total memory used by stored items, in bytes.
    ///
    /// Uses `std::mem::size_of::<T>()` per item — this captures stack/inline
    /// size accurately. Heap allocations inside `T` (e.g. `String` payload)
    /// are not tracked.
    #[inline]
    pub fn memory_usage_bytes(&self) -> usize {
        self.usage_bytes
    }

    /// The maximum number of items this buffer can hold.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.capacity
    }
}
