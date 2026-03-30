//! Generic cursor-based pagination state machine.
//!
//! This module is **pure** — no I/O, no side effects.  It tracks cursor
//! state and detects infinite loops (same cursor returned twice) to satisfy
//! NFR-013 (pagination terminates).
//!
//! The [`PaginationState`] type is consumed and reproduced on each step so
//! transitions can be modelled as pure functions.  The effectful caller
//! (e.g. `protocol::list_tools`) drives the loop.
//!
//! ## Loop detection (VP-003)
//!
//! Two complementary guards prevent infinite pagination:
//! 1. **Cursor dedup set**: if the same cursor string is seen a second time,
//!    `advance()` returns `PaginationStep::LoopDetected`.
//! 2. **Page cap**: after 100 pages the iterator always stops, regardless of
//!    whether the cursor repeated.  `E-PRO-004` is emitted in both cases.

use std::collections::HashSet;

/// Maximum number of pages fetched before forcing termination.
///
/// Satisfies NFR-013: "pagination iterator SHALL terminate after at most 100
/// pages even when the server returns a non-terminating cursor sequence."
pub const MAX_PAGES: usize = 100;

/// Represents the current state of a cursor-based pagination sequence.
///
/// The type is `Clone` so callers can snapshot state if needed, but the
/// normal usage is a simple `while let` loop driven by [`PaginationState::advance`].
#[derive(Debug, Clone)]
pub struct PaginationState {
    /// Cursors we have already consumed.  Used for loop detection.
    seen_cursors: HashSet<String>,
    /// Number of pages fetched so far (including the current one in flight).
    pages: usize,
}

/// The decision returned by [`PaginationState::advance`].
#[derive(Debug, PartialEq, Eq)]
pub enum PaginationStep {
    /// Fetch the next page using this cursor (may be `None` for the first page).
    FetchPage { cursor: Option<String> },
    /// Pagination is complete — the server returned no next cursor.
    Done,
    /// A cursor loop was detected or the page cap was reached.
    /// The caller SHOULD emit `E-PRO-004` and stop iterating.
    LoopDetected { pages: usize },
}

impl Default for PaginationState {
    fn default() -> Self {
        Self::new()
    }
}

impl PaginationState {
    /// Create a fresh pagination state (ready to fetch page 1).
    pub fn new() -> Self {
        Self {
            seen_cursors: HashSet::new(),
            pages: 0,
        }
    }

    /// Returns the number of pages fetched so far.
    pub fn pages(&self) -> usize {
        self.pages
    }

    /// Pure transition: given the `next_cursor` from the most recent page
    /// response, decide what to do next.
    ///
    /// # Arguments
    /// * `next_cursor` — the `nextCursor` field from the server's list response.
    ///   Pass `None` if the field was absent (signals last page).
    ///
    /// # Return value
    /// * [`PaginationStep::Done`] — iteration is complete.
    /// * [`PaginationStep::FetchPage`] — caller should fetch with this cursor.
    /// * [`PaginationStep::LoopDetected`] — stop; caller should emit `E-PRO-004`.
    pub fn advance(&mut self, next_cursor: Option<String>) -> PaginationStep {
        self.pages += 1;

        match next_cursor {
            // No more pages — clean termination.
            None => PaginationStep::Done,

            Some(cursor) => {
                // Page cap guard (NFR-013).
                if self.pages >= MAX_PAGES {
                    return PaginationStep::LoopDetected { pages: self.pages };
                }

                // Cursor dedup guard.
                if self.seen_cursors.contains(&cursor) {
                    return PaginationStep::LoopDetected { pages: self.pages };
                }

                self.seen_cursors.insert(cursor.clone());
                PaginationStep::FetchPage {
                    cursor: Some(cursor),
                }
            }
        }
    }

    /// Returns the initial step before any pages have been fetched.
    ///
    /// Convenience method: the first page always uses `cursor = None`.
    pub fn first_page() -> PaginationStep {
        PaginationStep::FetchPage { cursor: None }
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use super::*;

    /// Pagination terminates when the server returns no next_cursor.
    #[test]
    fn test_pagination_terminates_on_no_cursor() {
        let mut state = PaginationState::new();
        // Simulate: server returns first page with a cursor, second page without.
        let step = state.advance(Some("page:1".to_string()));
        assert_eq!(step, PaginationStep::FetchPage { cursor: Some("page:1".to_string()) });
        let step = state.advance(None);
        assert_eq!(step, PaginationStep::Done);
        assert_eq!(state.pages(), 2);
    }

    /// Cursor dedup: same cursor twice → LoopDetected.
    #[test]
    fn test_BC_2_05_001_pagination_cursor_loop_dedup() {
        let mut state = PaginationState::new();
        // First page: cursor "loop" returned.
        let step = state.advance(Some("loop".to_string()));
        assert_eq!(step, PaginationStep::FetchPage { cursor: Some("loop".to_string()) });
        // Second page: same cursor returned again.
        let step = state.advance(Some("loop".to_string()));
        assert_eq!(step, PaginationStep::LoopDetected { pages: 2 });
    }

    /// Page cap: after MAX_PAGES advance calls, LoopDetected is returned.
    #[test]
    fn test_BC_2_05_001_pagination_page_cap_100() {
        let mut state = PaginationState::new();
        // Send 99 unique cursors — all should succeed.
        for i in 0..(MAX_PAGES - 1) {
            let step = state.advance(Some(format!("cursor:{i}")));
            match step {
                PaginationStep::FetchPage { .. } => {}
                other => panic!("unexpected step at page {i}: {other:?}"),
            }
        }
        // The 100th advance (pages == MAX_PAGES) triggers the cap.
        let step = state.advance(Some("cursor:99".to_string()));
        assert_eq!(step, PaginationStep::LoopDetected { pages: MAX_PAGES });
    }

    /// Single-page server: advance(None) on first call → Done immediately.
    #[test]
    fn test_pagination_single_page() {
        let mut state = PaginationState::new();
        let step = state.advance(None);
        assert_eq!(step, PaginationStep::Done);
        assert_eq!(state.pages(), 1);
    }

    /// first_page() convenience returns FetchPage { cursor: None }.
    #[test]
    fn test_pagination_first_page_convenience() {
        assert_eq!(PaginationState::first_page(), PaginationStep::FetchPage { cursor: None });
    }

    /// Different cursors on each page → no loop, terminates cleanly.
    #[test]
    fn test_pagination_many_unique_cursors_no_loop() {
        let mut state = PaginationState::new();
        for i in 0..50 {
            let step = state.advance(Some(format!("c:{i}")));
            assert!(matches!(step, PaginationStep::FetchPage { .. }), "page {i} should succeed");
        }
        let step = state.advance(None);
        assert_eq!(step, PaginationStep::Done);
    }
}
