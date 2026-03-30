//! Elicitation dialog stub for forge-tui.
//!
//! This module provides the TUI-side stub for MCP `elicitation/create` requests.
//! Full ratatui form rendering is deferred to Wave 4 (STORY-037+).
//!
//! ## Elicitation modes
//!
//! The MCP `elicitation/create` method supports two modes:
//!
//! - **Form mode** (`CreateElicitationRequestParams::FormElicitationParams`):
//!   The server provides a JSON Schema describing the fields to collect. The TUI
//!   renders a modal form, and on submit returns the collected data as `ElicitResult`.
//!
//! - **URL mode** (`CreateElicitationRequestParams::UrlElicitationParams`):
//!   The server provides a URL for the user to open (e.g., OAuth flow). The TUI
//!   displays the URL and prompts the user to confirm completion. Returns
//!   `action: Accept` with `{"confirmed": true}` when the user confirms.
//!
//! ## Cancel behavior
//!
//! The user can press Escape to cancel any elicitation dialog. This produces
//! `action: Cancel` with no content. The server should treat this as the user
//! aborting the operation.
//!
//! ## Status
//!
//! - ✅ Handler routing in `forge-core::handler` (STORY-020)
//! - ✅ Non-interactive rejection (E-PRO-008) (STORY-020)
//! - 🔲 Form rendering via ratatui (STORY-037)
//! - 🔲 URL display with confirmation prompt (STORY-037)
//! - 🔲 Keyboard navigation and validation (STORY-037)

// Placeholder — full implementation in STORY-037.
//
// When STORY-037 is implemented, this module will export:
//
// ```rust
// pub struct ElicitationDialog { ... }
//
// impl ElicitationDialog {
//     pub fn new(params: CreateElicitationRequestParams) -> Self { ... }
//     pub fn render(&self, frame: &mut Frame, area: Rect) { ... }
//     pub fn handle_event(&mut self, event: &Event) -> Option<CreateElicitationResult> { ... }
// }
// ```
