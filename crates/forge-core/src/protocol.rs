//! High-level protocol operations: tool list and tool invocation.
//!
//! This module is **effectful** — it performs RPC calls via `McpConnection`.
//!
//! ## Responsibilities
//! - Paginated `tools/list` via [`list_tools`] (reuses [`crate::pagination`])
//! - Argument schema validation before `tools/call` (saves a round-trip)
//! - Tool-error vs protocol-error distinction (VP-002, DI-020)
//! - `list_changed` invalidation signal via [`ToolListWatcher`]
//!
//! ## Error classification (CRITICAL — VP-002 / DI-020)
//!
//! | Scenario | Return value |
//! |---------|-------------|
//! | Server returns `isError: true` | `Ok(ToolResult { is_error: true, .. })` |
//! | Transport / JSON-RPC failure | `Err(CoreError::Protocol(...))` |
//! | Server lacks tools capability | `Err(CoreError::CapabilityNotSupported)` |
//! | Arguments fail schema validation | `Err(CoreError::SchemaValidationFailed)` |
//! | Pagination cursor loop | `Err(CoreError::PaginationCursorLoop)` + warning |

use std::fmt;

use rmcp::{
    ClientHandler,
    model::{CallToolResult, PaginatedRequestParams, Tool},
};
use serde_json::Value;
use tokio::sync::watch;
use tracing::warn;

use crate::{
    connection::McpConnection,
    error::{CoreError, Result},
    pagination::{PaginationState, PaginationStep},
};

// ── Tool result type ──────────────────────────────────────────────────────────

/// The result of a `tools/call` invocation.
///
/// This is a Forge-friendly wrapper around rmcp's `CallToolResult` that makes
/// the `is_error` field non-optional and normalises the `content` field.
///
/// ## Error classification (VP-002 / DI-020)
/// When `is_error` is `true`, the call itself succeeded at the protocol level —
/// the tool ran and reported an application-level error.  Callers MUST NOT
/// treat this as a transport failure.  Only network/JSON-RPC errors propagate
/// as `Err(...)`.
#[derive(Debug, Clone, PartialEq)]
pub struct ToolResult {
    /// Content items returned by the tool.
    pub content: Vec<rmcp::model::Content>,
    /// `true` iff the tool reported an application-level error (`isError: true`).
    pub is_error: bool,
}

impl From<CallToolResult> for ToolResult {
    fn from(r: CallToolResult) -> Self {
        Self {
            content: r.content,
            is_error: r.is_error.unwrap_or(false),
        }
    }
}

impl fmt::Display for ToolResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // AC-005: ✓ for success, ✗ for error.
        let status = if self.is_error { "✗ error" } else { "✓ ok" };
        // Render the first content item's text, if any, for context.
        let preview: String = self
            .content
            .iter()
            .filter_map(|c| {
                // Extract text from Text variant; skip other content types.
                match &c.raw {
                    rmcp::model::RawContent::Text(t) => Some(t.text.clone()),
                    _ => None,
                }
            })
            .take(1)
            .collect();
        if preview.is_empty() {
            write!(f, "{status}")
        } else {
            write!(f, "{status}: {preview}")
        }
    }
}

// ── list_tools ────────────────────────────────────────────────────────────────

/// Retrieve the complete list of tools from a connected MCP server.
///
/// Iterates through all pages using cursor-based pagination.  Cursor loop
/// detection is delegated to [`PaginationState`] (pure, VP-003):
/// - If the same cursor is seen twice → stop + `E-PRO-004`.
/// - After 100 pages → stop + `E-PRO-004`.
///
/// # Errors
/// - `E-PRO-003`: server does not advertise the `tools` capability.
/// - `E-PRO-004`: pagination cursor loop detected.
/// - `E-PRO-001`: transport or JSON-RPC failure.
pub async fn list_tools<H: ClientHandler>(conn: &McpConnection<H>) -> Result<Vec<Tool>> {
    // Capability guard — pure check, no I/O.
    if !conn.supports_tools() {
        return Err(CoreError::CapabilityNotSupported {
            method: "tools/list".to_string(),
            capability: "tools".to_string(),
        });
    }

    let peer = conn
        .peer()
        .ok_or_else(|| CoreError::Protocol("connection peer unavailable".to_string()))?;

    let mut all_tools: Vec<Tool> = Vec::new();
    let mut state = PaginationState::new();
    let mut next_step = PaginationState::first_page();

    loop {
        // Determine the cursor for this request.
        let cursor: Option<String> = match next_step {
            PaginationStep::FetchPage { cursor } => cursor,
            PaginationStep::Done => break,
            PaginationStep::LoopDetected { pages } => {
                warn!(
                    pages,
                    "E-PRO-004: pagination cursor loop detected in tools/list"
                );
                return Err(CoreError::PaginationCursorLoop { pages });
            }
        };

        // Perform the RPC call.
        let params = cursor.map(|c| PaginatedRequestParams::default().with_cursor(Some(c)));
        let page = peer
            .list_tools(params)
            .await
            .map_err(|e| CoreError::Protocol(e.to_string()))?;

        all_tools.extend(page.tools);

        // Advance the pagination state machine.
        next_step = state.advance(page.next_cursor);
    }

    Ok(all_tools)
}

// ── call_tool ─────────────────────────────────────────────────────────────────

/// Invoke a named tool on a connected MCP server.
///
/// ## Schema validation (AC-005)
/// If a [`Tool`] definition is provided via `tool_def`, the `arguments` are
/// validated against the tool's `inputSchema` before the request is sent.
/// Invalid arguments return `Err(E-PRO-005)` without a round-trip.
///
/// ## Error classification (VP-002 / DI-020)
/// - `isError: true` in the response → `Ok(ToolResult { is_error: true, .. })`
/// - Network/JSON-RPC failure → `Err(CoreError::Protocol(...))`
///
/// # Arguments
/// * `conn` — live MCP connection.
/// * `name` — tool name to invoke.
/// * `arguments` — JSON object of arguments (pass `Value::Object(...)` or `Value::Null`).
/// * `tool_def` — optional tool definition used for pre-call schema validation.
///
/// # Errors
/// - `E-PRO-003`: server does not advertise the `tools` capability.
/// - `E-PRO-005`: arguments fail schema validation (only when `tool_def` provided).
/// - `E-PRO-001`: transport or JSON-RPC failure.
pub async fn call_tool<H: ClientHandler>(
    conn: &McpConnection<H>,
    name: &str,
    arguments: Value,
    tool_def: Option<&Tool>,
) -> Result<ToolResult> {
    // Capability guard.
    if !conn.supports_tools() {
        return Err(CoreError::CapabilityNotSupported {
            method: "tools/call".to_string(),
            capability: "tools".to_string(),
        });
    }

    // Schema validation (pure step — no I/O).
    if let Some(tool) = tool_def {
        validate_arguments(name, &tool.input_schema, &arguments)?;
    }

    // Build params for rmcp.
    let args_map = match arguments {
        Value::Object(map) => Some(map),
        Value::Null => None,
        // Non-object values are wrapped as `{ "value": ... }` to produce a
        // valid JSON-Schema object argument.
        other => {
            let mut map = serde_json::Map::new();
            map.insert("value".to_string(), other);
            Some(map)
        }
    };

    let params = match args_map {
        Some(map) => rmcp::model::CallToolRequestParams::new(name.to_string()).with_arguments(map),
        None => rmcp::model::CallToolRequestParams::new(name.to_string()),
    };

    let peer = conn
        .peer()
        .ok_or_else(|| CoreError::Protocol("connection peer unavailable".to_string()))?;

    // Execute the RPC.  Protocol failures become Err; isError=true stays Ok.
    let raw = peer
        .call_tool(params)
        .await
        .map_err(|e| CoreError::Protocol(e.to_string()))?;

    Ok(ToolResult::from(raw))
}

// ── Schema validation (pure helper) ──────────────────────────────────────────

/// Validate `arguments` against a JSON Schema object.
///
/// Returns `Err(E-PRO-005)` if validation fails, `Ok(())` otherwise.
/// An empty schema (`{}`) accepts any arguments (EC-002).
fn validate_arguments(
    tool_name: &str,
    schema: &serde_json::Map<String, Value>,
    arguments: &Value,
) -> Result<()> {
    // Empty schema — accept any arguments (EC-002).
    if schema.is_empty() {
        return Ok(());
    }

    let schema_value = Value::Object(schema.clone());

    // Build the validator.
    let compiled = jsonschema::validator_for(&schema_value)
        .map_err(|e| CoreError::SchemaValidationFailed {
            tool: tool_name.to_string(),
            reason: format!("invalid inputSchema: {e}"),
        })?;

    // Collect validation errors into a single message.
    let errors: Vec<String> = compiled
        .iter_errors(arguments)
        .map(|e| e.to_string())
        .collect();

    if errors.is_empty() {
        Ok(())
    } else {
        Err(CoreError::SchemaValidationFailed {
            tool: tool_name.to_string(),
            reason: errors.join("; "),
        })
    }
}

// ── ToolListWatcher — list_changed notification support (AC-006) ─────────────

/// A watcher that signals when the server sends `notifications/tools/list_changed`.
///
/// Consumers hold the `ToolListWatcher` and call `is_stale()` or `changed()`
/// to detect invalidation.  The `ToolListInvalidator` is held by the
/// `ClientHandler` and calls `invalidate()` from the notification callback.
#[derive(Debug, Clone)]
pub struct ToolListWatcher {
    rx: watch::Receiver<u64>,
}

/// The invalidation side of the [`ToolListWatcher`] / `ToolListInvalidator` pair.
///
/// Call [`ToolListInvalidator::invalidate`] from a `list_changed` notification
/// handler.  All [`ToolListWatcher`] clones that share the channel will be
/// notified.
#[derive(Debug, Clone)]
pub struct ToolListInvalidator {
    tx: watch::Sender<u64>,
}

impl ToolListInvalidator {
    /// Create a new linked watcher/invalidator pair.
    ///
    /// The returned `ToolListWatcher` starts at generation `0`.
    pub fn new() -> (Self, ToolListWatcher) {
        let (tx, rx) = watch::channel(0u64);
        (Self { tx }, ToolListWatcher { rx })
    }

    /// Signal all watchers that the tool list has changed.
    ///
    /// Monotonically increments the generation counter.
    pub fn invalidate(&self) {
        self.tx.send_modify(|g| *g += 1);
    }

    /// Returns the current generation counter.
    pub fn generation(&self) -> u64 {
        *self.tx.borrow()
    }
}

impl Default for ToolListInvalidator {
    fn default() -> Self {
        Self::new().0
    }
}

impl ToolListWatcher {
    /// Returns `true` if the tool list has been invalidated since the last
    /// time `mark_fresh` was called (or since construction).
    pub fn is_stale(&mut self) -> bool {
        self.rx.has_changed().unwrap_or(false)
    }

    /// Mark the current generation as seen (clears the `has_changed` flag).
    pub fn mark_fresh(&mut self) {
        self.rx.mark_unchanged();
    }

    /// Returns the current generation counter value.
    pub fn generation(&self) -> u64 {
        *self.rx.borrow()
    }

    /// Async wait for the next invalidation.
    pub async fn wait_for_change(&mut self) {
        let _ = self.rx.changed().await;
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use super::*;

    // ── Schema validation ─────────────────────────────────────────────────────

    /// AC-005: Valid arguments pass schema validation.
    #[test]
    fn test_BC_2_05_001_schema_validation_valid_args() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "input": { "type": "string" }
            },
            "required": ["input"]
        });
        let schema_obj = schema.as_object().unwrap().clone();
        let args = serde_json::json!({ "input": "hello" });
        assert!(validate_arguments("my_tool", &schema_obj, &args).is_ok());
    }

    /// AC-005: Missing required field → Err(E-PRO-005).
    #[test]
    fn test_BC_2_05_001_argument_schema_validation_missing_required() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "input": { "type": "string" }
            },
            "required": ["input"]
        });
        let schema_obj = schema.as_object().unwrap().clone();
        let args = serde_json::json!({}); // missing "input"
        let err = validate_arguments("my_tool", &schema_obj, &args).unwrap_err();
        match err {
            CoreError::SchemaValidationFailed { tool, reason } => {
                assert_eq!(tool, "my_tool");
                assert!(!reason.is_empty());
            }
            other => panic!("expected SchemaValidationFailed, got {other:?}"),
        }
    }

    /// AC-005: Wrong type → Err(E-PRO-005).
    #[test]
    fn test_BC_2_05_001_schema_validation_wrong_type() {
        let schema = serde_json::json!({
            "type": "object",
            "properties": {
                "count": { "type": "integer" }
            },
            "required": ["count"]
        });
        let schema_obj = schema.as_object().unwrap().clone();
        let args = serde_json::json!({ "count": "not-an-int" });
        let err = validate_arguments("counter_tool", &schema_obj, &args).unwrap_err();
        assert!(matches!(err, CoreError::SchemaValidationFailed { .. }));
    }

    /// EC-002: Empty schema accepts any arguments.
    #[test]
    fn test_BC_2_05_001_empty_schema_accepts_any() {
        let schema_obj = serde_json::Map::new(); // empty schema
        let args = serde_json::json!({ "anything": 42 });
        assert!(validate_arguments("any_tool", &schema_obj, &args).is_ok());
    }

    // ── ToolResult conversion ─────────────────────────────────────────────────

    /// AC-004: isError=true in CallToolResult → ToolResult { is_error: true }.
    #[test]
    fn test_BC_2_05_001_tool_error_is_error_true() {
        let raw = CallToolResult::error(vec![rmcp::model::Content::text("oops")]);
        let result = ToolResult::from(raw);
        assert!(result.is_error, "isError=true must produce ToolResult.is_error=true");
        assert_eq!(result.content.len(), 1);
    }

    /// AC-004: isError=false in CallToolResult → ToolResult { is_error: false }.
    #[test]
    fn test_BC_2_05_001_tool_success_is_error_false() {
        let raw = CallToolResult::success(vec![rmcp::model::Content::text("ok")]);
        let result = ToolResult::from(raw);
        assert!(!result.is_error, "isError=false must produce ToolResult.is_error=false");
    }

    /// AC-004: isError absent → ToolResult { is_error: false } (defaults to non-error).
    #[test]
    fn test_tool_result_is_error_absent_defaults_false() {
        // CallToolResult::success sets is_error=Some(false).  To simulate "absent"
        // we can't use struct literal (non_exhaustive), so we use serde round-trip.
        let json = serde_json::json!({ "content": [{ "type": "text", "text": "data" }] });
        let raw: CallToolResult = serde_json::from_value(json).expect("deserialize");
        // is_error should be None after deserialization (absent in JSON).
        let result = ToolResult::from(raw);
        assert!(!result.is_error, "absent isError should default to false");
    }

    // ── ToolListWatcher / ToolListInvalidator ────────────────────────────────

    /// AC-006: Invalidator triggers watcher.
    #[test]
    fn test_BC_2_05_001_list_changed_notification() {
        let (inv, mut watcher) = ToolListInvalidator::new();

        // Initially fresh.
        assert!(!watcher.is_stale(), "should not be stale before invalidation");
        assert_eq!(watcher.generation(), 0);

        // Invalidate.
        inv.invalidate();
        assert!(watcher.is_stale(), "should be stale after invalidation");
        assert_eq!(watcher.generation(), 1);
        assert_eq!(inv.generation(), 1);

        // Mark fresh.
        watcher.mark_fresh();
        assert!(!watcher.is_stale(), "should not be stale after mark_fresh");

        // Invalidate again.
        inv.invalidate();
        assert!(watcher.is_stale(), "should be stale after second invalidation");
        assert_eq!(watcher.generation(), 2);
    }

    /// Multiple invalidations increment generation monotonically.
    #[test]
    fn test_list_changed_multi_invalidate() {
        let (inv, watcher) = ToolListInvalidator::new();
        for i in 1..=5u64 {
            inv.invalidate();
            assert_eq!(watcher.generation(), i);
        }
    }
}
