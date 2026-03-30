//! MockServer: implements the rmcp `ServerHandler` trait.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use rmcp::model::{
    AnnotateAble, CallToolRequestParams, CallToolResult, CompleteRequestParams, CompleteResult,
    CompletionInfo, Content, GetPromptRequestParams, GetPromptResult, Implementation,
    InitializeResult, ListPromptsResult, ListResourcesResult, ListToolsResult,
    PaginatedRequestParams, Prompt, PromptMessage, PromptMessageRole, RawResource,
    ReadResourceRequestParams, ReadResourceResult, Resource, ResourceContents, ResourcesCapability,
    ServerCapabilities, ServerInfo, SetLevelRequestParams, SubscribeRequestParams, Tool,
    ToolsCapability, UnsubscribeRequestParams,
};
use rmcp::service::{NotificationContext, RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler};
use serde_json::json;

use crate::config::{MockConfig, MockTool};

// ── Pagination helpers ──────────────────────────────────────────────────────

fn encode_cursor(offset: usize) -> String {
    format!("page:{offset}")
}

fn decode_cursor(cursor: &str) -> Option<usize> {
    cursor.strip_prefix("page:").and_then(|s| s.parse().ok())
}

// ── Schema builder ──────────────────────────────────────────────────────────

fn build_input_schema(
    tool: &MockTool,
    drifted: bool,
) -> Arc<serde_json::Map<String, serde_json::Value>> {
    let mut properties = serde_json::Map::new();
    properties.insert(
        "input".to_string(),
        json!({"type": "string", "description": "Tool input"}),
    );
    if drifted {
        properties.insert(
            "drifted_param".to_string(),
            json!({"type": "string", "description": "Added by drift simulation"}),
        );
    }
    for extra in &tool.extra_params {
        properties.insert(
            extra.clone(),
            json!({"type": "string", "description": format!("Extra param: {extra}")}),
        );
    }
    let schema = serde_json::Map::from_iter([
        ("type".to_string(), json!("object")),
        (
            "properties".to_string(),
            serde_json::Value::Object(properties),
        ),
    ]);
    Arc::new(schema)
}

// ── MockServer ──────────────────────────────────────────────────────────────

/// The mock MCP server handler.
pub struct MockServer {
    config: MockConfig,
    tool_list_calls: Arc<AtomicUsize>,
}

impl MockServer {
    pub fn new(config: MockConfig) -> Self {
        Self {
            config,
            tool_list_calls: Arc::new(AtomicUsize::new(0)),
        }
    }

    fn is_drifted(&self) -> bool {
        if let Some(dc) = &self.config.drift_config {
            let calls = self.tool_list_calls.load(Ordering::Relaxed);
            calls >= dc.drift_after_n_calls
        } else {
            false
        }
    }

    fn build_tools(&self) -> Vec<Tool> {
        let drifted = self.is_drifted();
        self.config
            .tools
            .iter()
            .map(|t| {
                Tool::new(
                    t.name.clone(),
                    t.description.clone(),
                    build_input_schema(t, drifted),
                )
            })
            .collect()
    }

    fn build_resources(&self) -> Vec<Resource> {
        self.config
            .resources
            .iter()
            .map(|r| {
                RawResource {
                    uri: r.uri.clone(),
                    name: r.name.clone(),
                    description: Some(r.description.clone()),
                    mime_type: Some("text/plain".to_string()),
                    title: None,
                    size: None,
                    icons: None,
                    meta: None,
                }
                .no_annotation()
            })
            .collect()
    }

    fn build_prompts(&self) -> Vec<Prompt> {
        self.config
            .prompts
            .iter()
            .map(|p| Prompt::new(p.name.clone(), Some(p.description.clone()), None))
            .collect()
    }

    fn paginate<T: Clone>(&self, items: &[T], cursor: Option<&str>) -> (Vec<T>, Option<String>) {
        let page_size = self.config.pagination.page_size;

        if page_size == 0 {
            return (items.to_vec(), None);
        }

        if self.config.pagination.loop_cursor {
            let start = cursor
                .and_then(decode_cursor)
                .unwrap_or(0)
                .min(items.len());
            let end = (start + page_size).min(items.len());
            let page = items[start..end].to_vec();
            // Always return the same cursor — simulates an infinite-loop server.
            return (page, Some(encode_cursor(start)));
        }

        let start = cursor
            .and_then(decode_cursor)
            .unwrap_or(0)
            .min(items.len());
        let end = (start + page_size).min(items.len());
        let page = items[start..end].to_vec();
        let next_cursor = if end < items.len() {
            Some(encode_cursor(end))
        } else {
            None
        };
        (page, next_cursor)
    }

    async fn maybe_delay(&self) {
        let ms = self.config.error_injection.delay_ms;
        if ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(ms)).await;
        }
    }

    fn maybe_crash(&self) {
        if self.config.error_injection.crash_on_tool_call {
            std::process::exit(1);
        }
    }

    fn build_server_capabilities(&self) -> ServerCapabilities {
        let caps = &self.config.capabilities;
        let mut sc = ServerCapabilities::default();
        if caps.tools {
            sc.tools = Some(ToolsCapability::default());
        }
        if caps.resources {
            sc.resources = Some(ResourcesCapability::default());
        }
        if caps.prompts {
            sc.prompts = Some(rmcp::model::PromptsCapability::default());
        }
        if caps.logging {
            sc.logging = Some(serde_json::Map::new());
        }
        if caps.completions {
            sc.completions = Some(serde_json::Map::new());
        }
        sc
    }
}

// ── ServerHandler impl ──────────────────────────────────────────────────────

impl ServerHandler for MockServer {
    fn get_info(&self) -> ServerInfo {
        InitializeResult::new(self.build_server_capabilities())
            .with_server_info(Implementation::new("forge-test-server", "0.1.0"))
    }

    async fn list_tools(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        self.maybe_delay().await;
        self.tool_list_calls.fetch_add(1, Ordering::Relaxed);
        let all_tools = self.build_tools();
        let cursor = request.as_ref().and_then(|r| r.cursor.as_deref());
        let (tools, next_cursor) = self.paginate(&all_tools, cursor);
        Ok(ListToolsResult {
            tools,
            next_cursor,
            meta: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        self.maybe_delay().await;
        self.maybe_crash();

        let tool_name = &request.name;

        let tool_exists = self.config.tools.iter().any(|t| t.name == *tool_name);
        if !tool_exists {
            return Err(McpError::invalid_params(
                format!("Unknown tool: {tool_name}"),
                None,
            ));
        }

        if self.config.error_injection.error_tool_result {
            return Ok(CallToolResult::error(vec![Content::text(format!(
                "Injected error for tool: {tool_name}"
            ))]));
        }

        Ok(CallToolResult::success(vec![Content::text(format!(
            "Mock result from {tool_name}"
        ))]))
    }

    async fn list_resources(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        self.maybe_delay().await;
        let all = self.build_resources();
        let cursor = request.as_ref().and_then(|r| r.cursor.as_deref());
        let (resources, next_cursor) = self.paginate(&all, cursor);
        Ok(ListResourcesResult {
            resources,
            next_cursor,
            meta: None,
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        self.maybe_delay().await;
        let uri = &request.uri;
        let resource = self.config.resources.iter().find(|r| r.uri == *uri);
        match resource {
            Some(r) => Ok(ReadResourceResult::new(vec![
                ResourceContents::TextResourceContents {
                    uri: r.uri.clone(),
                    mime_type: Some("text/plain".to_string()),
                    text: r.content.clone(),
                    meta: None,
                },
            ])),
            None => Err(McpError::invalid_params(
                format!("Unknown resource URI: {uri}"),
                None,
            )),
        }
    }

    async fn subscribe(
        &self,
        _request: SubscribeRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<(), McpError> {
        self.maybe_delay().await;
        Ok(())
    }

    async fn unsubscribe(
        &self,
        _request: UnsubscribeRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<(), McpError> {
        self.maybe_delay().await;
        Ok(())
    }

    async fn list_prompts(
        &self,
        request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        self.maybe_delay().await;
        let all = self.build_prompts();
        let cursor = request.as_ref().and_then(|r| r.cursor.as_deref());
        let (prompts, next_cursor) = self.paginate(&all, cursor);
        Ok(ListPromptsResult {
            prompts,
            next_cursor,
            meta: None,
        })
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResult, McpError> {
        self.maybe_delay().await;
        let name = &request.name;
        let prompt = self.config.prompts.iter().find(|p| p.name == *name);
        match prompt {
            Some(p) => {
                let msg = PromptMessage::new_text(
                    PromptMessageRole::Assistant,
                    format!("Mock response for prompt: {}", p.name),
                );
                Ok(GetPromptResult::new(vec![msg]))
            }
            None => Err(McpError::invalid_params(
                format!("Unknown prompt: {name}"),
                None,
            )),
        }
    }

    async fn complete(
        &self,
        _request: CompleteRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CompleteResult, McpError> {
        self.maybe_delay().await;
        let info = CompletionInfo::new(vec!["mock-completion".to_string()])
            .expect("always valid");
        Ok(CompleteResult::new(info))
    }

    async fn set_level(
        &self,
        _request: SetLevelRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<(), McpError> {
        self.maybe_delay().await;
        Ok(())
    }

    async fn ping(
        &self,
        _context: RequestContext<RoleServer>,
    ) -> Result<(), McpError> {
        Ok(())
    }

    async fn on_initialized(
        &self,
        _context: NotificationContext<RoleServer>,
    ) {
        tracing::info!("forge-test-server: client initialized");
    }
}
