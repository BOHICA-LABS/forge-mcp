//! LLM proxy client for `sampling/createMessage` forwarding.
//!
//! Translates MCP `CreateMessageRequestParams` into OpenAI-compatible
//! `POST /v1/chat/completions` requests and maps the response back to
//! `CreateMessageResult`.
//!
//! ## Configuration
//!
//! Read from environment variables at construction time:
//! - `FORGE_LLM_URL`     — base URL (e.g. `https://api.openai.com`)
//! - `FORGE_LLM_API_KEY` — bearer token sent in `Authorization` header
//! - `FORGE_LLM_MODEL`   — optional model override (e.g. `gpt-4o`)
//!
//! If `FORGE_LLM_URL` is unset, [`LlmProxy::from_env`] returns `None` and
//! callers must fall back to `E-PRO-007`.

use rmcp::{
    ErrorData as McpError,
    model::{
        CreateMessageRequestParams, CreateMessageResult, ErrorCode, Role, SamplingMessage,
        SamplingMessageContent,
    },
};
use serde::{Deserialize, Serialize};

// ── Wire types (OpenAI chat completions API) ──────────────────────────────────

/// A single message in the OpenAI chat completions request.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct OpenAiMessage {
    pub role: String,
    pub content: String,
}

/// Payload for `POST /v1/chat/completions`.
#[derive(Debug, Clone, Serialize)]
struct OpenAiRequest {
    pub model: String,
    pub messages: Vec<OpenAiMessage>,
    pub max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
}

/// One choice returned by the OpenAI API.
#[derive(Debug, Deserialize)]
struct OpenAiChoice {
    pub message: OpenAiMessage,
    pub finish_reason: Option<String>,
}

/// Top-level OpenAI chat completions response.
#[derive(Debug, Deserialize)]
struct OpenAiResponse {
    pub model: String,
    pub choices: Vec<OpenAiChoice>,
}

// ── LlmProxyConfig ────────────────────────────────────────────────────────────

/// Pure configuration struct for the LLM proxy.
///
/// Constructed from environment variables by [`LlmProxy::from_env`].
/// Fields are intentionally public for testing.
#[derive(Debug, Clone)]
pub struct LlmProxyConfig {
    /// Base URL of the LLM API (e.g. `https://api.openai.com`).
    pub url: String,
    /// Bearer token sent in the `Authorization` header.
    pub api_key: String,
    /// Optional model name override. When `None` the proxy uses the model
    /// hint from `modelPreferences.hints[0].name` or falls back to `"gpt-4o"`.
    pub model_override: Option<String>,
}

impl LlmProxyConfig {
    /// Construct a config from explicit values (used in tests).
    pub fn new(url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            api_key: api_key.into(),
            model_override: None,
        }
    }

    /// Construct a config with an explicit model override.
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model_override = Some(model.into());
        self
    }

    /// Read config from environment variables.
    ///
    /// Returns `None` when `FORGE_LLM_URL` is not set (sampling disabled).
    pub fn from_env() -> Option<Self> {
        let url = std::env::var("FORGE_LLM_URL").ok()?;
        let api_key = std::env::var("FORGE_LLM_API_KEY").unwrap_or_default();
        let model_override = std::env::var("FORGE_LLM_MODEL").ok();
        Some(Self {
            url,
            api_key,
            model_override,
        })
    }
}

// ── LlmProxy ─────────────────────────────────────────────────────────────────

/// HTTP proxy that forwards MCP sampling requests to an OpenAI-compatible LLM.
///
/// Effectful — performs outbound HTTP.
#[derive(Debug, Clone)]
pub struct LlmProxy {
    config: LlmProxyConfig,
    client: reqwest::Client,
}

impl LlmProxy {
    /// Create a proxy from a config.
    pub fn new(config: LlmProxyConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Construct from environment variables. Returns `None` when `FORGE_LLM_URL`
    /// is unset (sampling is not configured).
    pub fn from_env() -> Option<Self> {
        LlmProxyConfig::from_env().map(Self::new)
    }

    /// Forward an MCP sampling request to the configured LLM endpoint.
    ///
    /// Maps `CreateMessageRequestParams` → OpenAI chat completions request,
    /// performs the HTTP call, then maps the response back to `CreateMessageResult`.
    ///
    /// # Errors
    ///
    /// Returns `Err(McpError)` when:
    /// - HTTP transport or TLS errors occur
    /// - The LLM API returns a non-2xx status
    /// - The response body cannot be parsed
    pub async fn send_sampling_request(
        &self,
        request: &CreateMessageRequestParams,
    ) -> Result<CreateMessageResult, McpError> {
        // Determine model to use.
        let model = self
            .config
            .model_override
            .clone()
            .or_else(|| {
                request
                    .model_preferences
                    .as_ref()
                    .and_then(|mp| mp.hints.as_ref())
                    .and_then(|hints| hints.first())
                    .and_then(|h| h.name.clone())
            })
            .unwrap_or_else(|| "gpt-4o".to_string());

        // Convert MCP messages to OpenAI format.
        let messages = self.convert_messages(&request.messages);

        // Build the OpenAI request, forwarding relevant fields.
        let openai_req = OpenAiRequest {
            model,
            messages,
            max_tokens: request.max_tokens,
            stop: request.stop_sequences.clone(),
            temperature: request.temperature,
        };

        // POST to /v1/chat/completions.
        let url = format!("{}/v1/chat/completions", self.config.url.trim_end_matches('/'));

        let response = self
            .client
            .post(&url)
            .bearer_auth(&self.config.api_key)
            .json(&openai_req)
            .send()
            .await
            .map_err(|e| {
                McpError::new(
                    ErrorCode::INTERNAL_ERROR,
                    format!("E-PRO-007: LLM proxy HTTP error: {e}"),
                    None,
                )
            })?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().await.unwrap_or_default();
            return Err(McpError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("E-PRO-007: LLM API returned HTTP {status}: {body}"),
                None,
            ));
        }

        let openai_resp: OpenAiResponse = response.json().await.map_err(|e| {
            McpError::new(
                ErrorCode::INTERNAL_ERROR,
                format!("E-PRO-007: failed to parse LLM response: {e}"),
                None,
            )
        })?;

        // Map the first choice back to CreateMessageResult.
        let choice = openai_resp.choices.into_iter().next().ok_or_else(|| {
            McpError::new(
                ErrorCode::INTERNAL_ERROR,
                "E-PRO-007: LLM returned no choices",
                None,
            )
        })?;

        let message = SamplingMessage::new(
            Role::Assistant,
            SamplingMessageContent::text(choice.message.content),
        );

        let mut result = CreateMessageResult::new(message, openai_resp.model);
        if let Some(finish_reason) = choice.finish_reason {
            result = result.with_stop_reason(finish_reason);
        }

        Ok(result)
    }

    /// Convert MCP `SamplingMessage` list to OpenAI message list.
    ///
    /// Only text content is forwarded; image/audio content is represented
    /// as a placeholder. This is sufficient for the sampling proxy use-case
    /// as most LLM interactions are text-based.
    fn convert_messages(&self, messages: &[SamplingMessage]) -> Vec<OpenAiMessage> {
        messages
            .iter()
            .map(|msg| {
                let role = match msg.role {
                    Role::User => "user",
                    Role::Assistant => "assistant",
                }
                .to_string();

                // Extract text from the first text content block.
                let content = msg
                    .content
                    .first()
                    .and_then(|c| c.as_text())
                    .map(|t| t.text.clone())
                    .unwrap_or_else(|| "[non-text content]".to_string());

                OpenAiMessage { role, content }
            })
            .collect()
    }
}
