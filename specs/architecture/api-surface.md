---
document_type: architecture-section
level: L3
section: api-surface
version: "1.0"
status: draft
producer: architect
timestamp: 2026-03-29T14:05:00
phase: 1b
inputs: [domain-spec/L2-INDEX.md, specs/prd.md]
traces_to: ARCH-INDEX.md
---

# API Surface

Key traits and types that form the inter-module contracts.

## forge-core public API

```rust
// Protocol adapter trait — thin wrapper over rmcp Peer
pub trait McpClient: Send + Sync {
    async fn initialize(&self, caps: ClientCapabilities) -> Result<ServerCapabilities>;
    async fn list_tools(&self, cursor: Option<&str>) -> Result<PaginatedList<Tool>>;
    async fn call_tool(&self, name: &str, args: Value) -> Result<ToolResult>;
    async fn list_resources(&self, cursor: Option<&str>) -> Result<PaginatedList<Resource>>;
    async fn read_resource(&self, uri: &str) -> Result<ResourceContent>;
    async fn list_prompts(&self, cursor: Option<&str>) -> Result<PaginatedList<Prompt>>;
    async fn get_prompt(&self, name: &str, args: Value) -> Result<PromptResult>;
    async fn complete(&self, ref_type: CompletionRef, argument: &str) -> Result<Vec<String>>;
    async fn set_log_level(&self, level: LogLevel) -> Result<()>;
    async fn ping(&self) -> Result<Duration>;
    async fn subscribe_resource(&self, uri: &str) -> Result<()>;
    async fn unsubscribe_resource(&self, uri: &str) -> Result<()>;
}

// Client handler callbacks for server-initiated methods
pub trait ClientCallbacks: Send + Sync {
    async fn on_sampling_request(&self, req: SamplingRequest) -> Result<SamplingResponse>;
    async fn on_elicitation_request(&self, req: ElicitationRequest) -> Result<ElicitationResponse>;
    async fn on_roots_list(&self) -> Result<Vec<Root>>;
    async fn on_log_message(&self, msg: LogMessage);
    async fn on_tool_list_changed(&self);
    async fn on_resource_list_changed(&self);
    async fn on_resource_updated(&self, uri: &str);
    async fn on_progress(&self, token: ProgressToken, progress: f64, total: Option<f64>);
}

// Core message type observed by traffic capture
pub struct McpMessage {
    pub id: Option<MessageId>,
    pub method: Option<String>,
    pub direction: Direction,
    pub payload: Value,
    pub timestamp: Instant,
    pub raw_bytes: Bytes,
}

pub enum Direction { ClientToServer, ServerToClient }
pub enum MessageId { Number(i64), String(String) }

// Pagination helper (DI-019)
pub struct PaginatedList<T> { pub items: Vec<T>, pub next_cursor: Option<String> }
pub async fn collect_all_pages<T, F, Fut>(fetcher: F) -> Result<Vec<T>>
where F: Fn(Option<&str>) -> Fut, Fut: Future<Output = Result<PaginatedList<T>>>;

// Error classification (DI-020)
pub enum McpError {
    Protocol(ProtocolError),   // JSON-RPC error codes
    Tool(ToolError),           // result.isError = true
    Transport(TransportError), // connection-level
}
```

## forge-discovery public API

```rust
pub struct ServerEntry {
    pub name: String,
    pub transport: TransportConfig,
    pub source: ConfigSource,
    pub status: ServerStatus,
}

pub enum TransportConfig {
    Stdio { command: String, args: Vec<String>, env: HashMap<String, String> },
    Http { url: String, headers: HashMap<String, String> },
}

pub enum ConfigSource { ClaudeDesktop, Cursor, VsCode, Windsurf, Manual }

pub struct ServerRegistry { /* indexed by name */ }

pub fn discover_servers(sources: &[ConfigSource]) -> Result<ServerRegistry>;
pub fn parse_config(path: &Path, schema: ConfigSchema) -> Result<Vec<ServerEntry>>;
pub enum ConfigSchema { McpServers, Servers }
```

## forge-traffic public API

```rust
pub struct CaptureBuffer { /* ring buffer, bounded by NFR-012 */ }

impl CaptureBuffer {
    pub fn append(&mut self, msg: McpMessage);
    pub fn filter(&self, filter: &TrafficFilter) -> Vec<&McpMessage>;
    pub fn search(&self, query: &str) -> Vec<&McpMessage>;
    pub fn len(&self) -> usize;
    pub fn memory_usage(&self) -> usize;
}

pub struct TrafficFilter {
    pub methods: Option<Vec<String>>,
    pub direction: Option<Direction>,
    pub time_range: Option<(Instant, Instant)>,
    pub content_pattern: Option<Regex>,
}

pub struct TimingAnalysis {
    pub latency: Duration,
    pub gap_from_previous: Duration,
}
```

## forge-health public API

```rust
pub struct HealthCollector { /* per-server metrics */ }

pub struct MetricSnapshot {
    pub latency_histogram: Histogram,
    pub error_rate: f64,
    pub throughput_rps: f64,
    pub connection_status: ConnectionStatus,
    pub timestamp: Instant,
}

pub struct AlertState {
    pub status: AlertStatus,
    pub metric: MetricType,
    pub threshold: f64,
}

pub enum AlertStatus { Normal, Breached, Recovered }
```

## forge-security public API

```rust
pub struct SecurityEngine { /* rule set + state */ }

pub struct SecurityFinding {
    pub id: String,
    pub severity: Severity,
    pub confidence: f64,       // [0.0, 1.0] per DI-011
    pub category: Ast10Category,
    pub title: String,
    pub description: String,
    pub evidence: McpMessage,  // DI-010: findings require evidence
    pub suppressed: bool,
}

pub enum Severity { Critical, High, Medium, Low, Info }

pub enum Ast10Category {
    AST01, AST02, AST03, AST04, AST05,
    AST06, AST07, AST08, AST09, AST10,
}

pub fn analyze_message(msg: &McpMessage, rules: &RuleSet) -> Vec<SecurityFinding>;

/// Compares two tool sets and returns findings for any differences.
///
/// Uses a two-tier model:
/// - **Canonical fields** (name, description, inputSchema): changes produce ≥ high severity findings.
/// - **Auxiliary fields** (annotations): changes produce info-level findings.
/// - New/removed tools: info-level (added) or medium-level (removed) findings.
pub fn detect_schema_drift(prev: &ToolSet, current: &ToolSet) -> Vec<SecurityFinding>;
```

## forge-conformance public API

```rust
pub struct ConformanceResult {
    pub tests: Vec<TestResult>,
    pub summary: SuiteSummary,
}

pub struct TestResult {
    pub id: String,
    pub status: TestStatus,
    pub evidence: String,
    pub duration: Duration,
}

pub enum TestStatus { Pass, Fail, Skip, Error }

pub fn run_suite(client: &dyn McpClient, suite: SuiteType) -> Result<ConformanceResult>;
pub fn to_junit_xml(result: &ConformanceResult) -> String;
pub fn to_json(result: &ConformanceResult) -> Value;
```

## forge-config public API

```rust
pub struct DriftReport { pub diffs: Vec<ServerDiff> }

pub struct ServerDiff {
    pub name: String,
    pub source_a: ConfigSource,
    pub source_b: ConfigSource,
    pub changes: Vec<FieldChange>,
}

pub fn compare_configs(a: &ServerRegistry, b: &ServerRegistry) -> DriftReport;
pub fn diff_tool_schemas(a: &[Tool], b: &[Tool]) -> Vec<SchemaDiff>;
```
