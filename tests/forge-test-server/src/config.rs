//! Configuration structures for the mock MCP server.

use serde::{Deserialize, Serialize};

/// A single mock tool definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockTool {
    pub name: String,
    pub description: String,
    /// Extra input parameters (name → JSON Schema type string).
    #[serde(default)]
    pub extra_params: Vec<String>,
}

impl MockTool {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            extra_params: vec![],
        }
    }
}

/// A single mock resource definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockResource {
    pub uri: String,
    pub name: String,
    pub description: String,
    pub content: String,
}

impl MockResource {
    pub fn new(index: usize) -> Self {
        Self {
            uri: format!("resource://mock/{index}"),
            name: format!("Mock Resource {index}"),
            description: format!("Mock resource number {index}"),
            content: format!("Content of resource {index}"),
        }
    }
}

/// A single mock prompt definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockPrompt {
    pub name: String,
    pub description: String,
}

impl MockPrompt {
    pub fn new(index: usize) -> Self {
        Self {
            name: format!("mock_prompt_{index}"),
            description: format!("Mock prompt number {index}"),
        }
    }
}

/// What capabilities the server advertises.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MockCapabilities {
    pub tools: bool,
    pub resources: bool,
    pub prompts: bool,
    pub logging: bool,
    pub completions: bool,
}

impl MockCapabilities {
    /// Advertise all capabilities.
    pub fn all() -> Self {
        Self {
            tools: true,
            resources: true,
            prompts: true,
            logging: true,
            completions: true,
        }
    }
}

/// Cursor-based pagination configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationConfig {
    /// Maximum items to return per page. 0 means no pagination (all at once).
    pub page_size: usize,
    /// When true, always return the same cursor (simulates an infinite loop
    /// for testing client-side loop-detection).
    pub loop_cursor: bool,
}

impl Default for PaginationConfig {
    fn default() -> Self {
        Self {
            page_size: 0,
            loop_cursor: false,
        }
    }
}

/// Error injection configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ErrorInjectionConfig {
    /// Milliseconds to sleep before responding (0 = no delay).
    pub delay_ms: u64,
    /// If true, exit the process immediately when a tool is called.
    pub crash_on_tool_call: bool,
    /// If true, return `isError: true` from tool calls.
    pub error_tool_result: bool,
    /// If true, emit malformed (non-JSON) output and then exit.
    pub emit_malformed: bool,
}

/// Schema drift configuration: after `n_calls` tool-list calls the tool schema changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftConfig {
    /// After this many `tools/list` calls, mutate the schema.
    pub drift_after_n_calls: usize,
}

/// Top-level server configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MockConfig {
    pub tools: Vec<MockTool>,
    pub resources: Vec<MockResource>,
    pub prompts: Vec<MockPrompt>,
    pub capabilities: MockCapabilities,
    pub pagination: PaginationConfig,
    pub error_injection: ErrorInjectionConfig,
    pub drift_config: Option<DriftConfig>,
}

impl MockConfig {
    /// Build a quick default config from plain counts.
    pub fn from_counts(tool_count: usize, resource_count: usize, prompt_count: usize) -> Self {
        let tools = (0..tool_count)
            .map(|i| MockTool::new(format!("mock_tool_{i}"), format!("Mock tool {i}")))
            .collect();
        let resources = (0..resource_count).map(MockResource::new).collect();
        let prompts = (0..prompt_count).map(MockPrompt::new).collect();

        Self {
            tools,
            resources,
            prompts,
            capabilities: MockCapabilities::all(),
            pagination: PaginationConfig::default(),
            error_injection: ErrorInjectionConfig::default(),
            drift_config: None,
        }
    }
}
