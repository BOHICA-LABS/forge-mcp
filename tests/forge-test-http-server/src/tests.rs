//! Integration tests for the forge-test-http-server mock MCP server.
//!
//! Tests correspond to the STORY-003 acceptance criteria:
//! - AC-001: Session ID management
//! - AC-002: SSE stream for server-initiated messages
//! - AC-003: Session expiry simulation
//! - AC-004: Random port binding

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use reqwest::Client;
    use tokio_util::sync::CancellationToken;

    use crate::config::{HttpMockConfig, SessionConfig};
    use crate::server::MockHttpServer;

    const INIT_BODY: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#;
    const INIT_NOTIFICATION: &str = r#"{"jsonrpc":"2.0","method":"notifications/initialized"}"#;
    const LIST_TOOLS_BODY: &str =
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}"#;
    const LIST_RESOURCES_BODY: &str =
        r#"{"jsonrpc":"2.0","id":3,"method":"resources/list","params":{}}"#;

    // -----------------------------------------------------------------------
    // Helper: start server, return (client, base_url, ct)
    // -----------------------------------------------------------------------
    async fn spawn_default_server() -> (Client, String, CancellationToken) {
        spawn_server(HttpMockConfig::default()).await
    }

    async fn spawn_server(config: HttpMockConfig) -> (Client, String, CancellationToken) {
        let ct = CancellationToken::new();
        let (addr, _handle) = MockHttpServer::start(config, ct.child_token())
            .await
            .expect("server start");
        let base_url = format!("http://{addr}/mcp");
        let client = Client::new();
        (client, base_url, ct)
    }

    // -----------------------------------------------------------------------
    // AC-004: Random port binding
    // -----------------------------------------------------------------------

    /// The mock server binds to a random port and reports a non-zero port.
    #[tokio::test]
    async fn test_http_mock_random_port() {
        let ct = CancellationToken::new();
        let (addr, _handle) = MockHttpServer::start(HttpMockConfig::default(), ct.child_token())
            .await
            .expect("server start");

        assert_ne!(addr.port(), 0, "port should be OS-assigned (non-zero)");
        assert!(addr.port() >= 1024 || addr.port() > 0);

        ct.cancel();
    }

    // -----------------------------------------------------------------------
    // AC-001: Mcp-Session-Id header management
    // -----------------------------------------------------------------------

    /// On initialize, the server returns an `mcp-session-id` header.
    #[tokio::test]
    async fn test_http_mock_session_id_assigned_on_initialize() {
        let (client, url, ct) = spawn_default_server().await;

        let resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .body(INIT_BODY)
            .send()
            .await
            .expect("send");

        assert_eq!(resp.status(), 200);
        let session_id = resp
            .headers()
            .get("mcp-session-id")
            .expect("mcp-session-id header must be present on initialize response")
            .to_str()
            .expect("valid utf-8")
            .to_owned();
        assert!(!session_id.is_empty(), "session ID must be non-empty");

        ct.cancel();
    }

    /// A request with an invalid/unknown session ID receives HTTP 404.
    #[tokio::test]
    async fn test_http_mock_invalid_session_id_returns_404() {
        let (client, url, ct) = spawn_default_server().await;

        let resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .header("mcp-session-id", "totally-bogus-session-id")
            .body(LIST_TOOLS_BODY)
            .send()
            .await
            .expect("send");

        assert_eq!(
            resp.status(),
            404,
            "unknown session ID should return 404"
        );

        ct.cancel();
    }

    /// A request with a valid session ID can fetch tools successfully.
    #[tokio::test]
    async fn test_http_mock_session_id_management() {
        let (client, url, ct) = spawn_default_server().await;

        // Step 1: initialize and capture session ID
        let init_resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .body(INIT_BODY)
            .send()
            .await
            .expect("initialize");

        assert_eq!(init_resp.status(), 200);
        let session_id = init_resp
            .headers()
            .get("mcp-session-id")
            .expect("mcp-session-id")
            .to_str()
            .expect("utf8")
            .to_owned();

        // Read the init response body to complete the SSE stream
        let _body = init_resp.text().await.expect("init body");

        // Step 2: send initialized notification
        let notif_resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .header("mcp-session-id", &session_id)
            .body(INIT_NOTIFICATION)
            .send()
            .await
            .expect("notification");
        assert_eq!(notif_resp.status(), 202, "notification should be accepted");

        // Step 3: list tools with valid session ID
        let tools_resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .header("mcp-session-id", &session_id)
            .body(LIST_TOOLS_BODY)
            .send()
            .await
            .expect("tools/list");

        assert_eq!(tools_resp.status(), 200, "tools/list should succeed");
        let body = tools_resp.text().await.expect("body");
        // Response is SSE-wrapped; check the JSON inside the data: field
        assert!(
            body.contains("echo") && body.contains("add"),
            "tools list should contain echo and add tools, got: {body}"
        );

        ct.cancel();
    }

    // -----------------------------------------------------------------------
    // AC-002: SSE stream for server-initiated messages
    // -----------------------------------------------------------------------

    /// The initialize response is delivered as an SSE stream.
    #[tokio::test]
    async fn test_http_mock_sse_notifications() {
        let (client, url, ct) = spawn_default_server().await;

        let resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .body(INIT_BODY)
            .send()
            .await
            .expect("send");

        assert_eq!(resp.status(), 200);

        // In stateful mode, the response is SSE
        let content_type = resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(
            content_type.contains("text/event-stream"),
            "initialize response should be SSE, got content-type: {content_type}"
        );

        let body = resp.text().await.expect("body");
        assert!(
            body.contains("data:"),
            "SSE body should contain data: field, got: {body}"
        );
        assert!(
            body.contains("\"result\"") || body.contains("jsonrpc"),
            "SSE body should contain MCP initialize result, got: {body}"
        );

        ct.cancel();
    }

    /// GET /mcp with a valid session ID opens a standalone SSE stream.
    #[tokio::test]
    async fn test_http_mock_sse_get_stream() {
        let (client, url, ct) = spawn_default_server().await;

        // First initialize to get a session
        let init_resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .body(INIT_BODY)
            .send()
            .await
            .expect("init");
        assert_eq!(init_resp.status(), 200);
        let session_id = init_resp
            .headers()
            .get("mcp-session-id")
            .expect("mcp-session-id")
            .to_str()
            .expect("utf8")
            .to_owned();
        let _init_body = init_resp.text().await.expect("init body");

        // Open standalone SSE stream via GET
        let sse_resp = client
            .get(&url)
            .header("Accept", "text/event-stream")
            .header("mcp-session-id", &session_id)
            .send()
            .await
            .expect("GET SSE");

        assert_eq!(
            sse_resp.status(),
            200,
            "GET /mcp should return 200 with valid session"
        );
        let content_type = sse_resp
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(
            content_type.contains("text/event-stream"),
            "GET /mcp should return SSE stream, got: {content_type}"
        );

        ct.cancel();
    }

    // -----------------------------------------------------------------------
    // AC-003: Session expiry / loss simulation
    // -----------------------------------------------------------------------

    /// A session with a very short TTL expires and returns 404 after the TTL.
    #[tokio::test]
    async fn test_http_mock_session_expiry() {
        let config = HttpMockConfig {
            session: SessionConfig {
                ttl_ms: Some(50), // expire after 50ms
                forget_after_n_requests: None,
            },
            ..Default::default()
        };
        let (client, url, ct) = spawn_server(config).await;

        // Initialize and capture session ID
        let init_resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .body(INIT_BODY)
            .send()
            .await
            .expect("init");
        assert_eq!(init_resp.status(), 200);
        let session_id = init_resp
            .headers()
            .get("mcp-session-id")
            .expect("mcp-session-id")
            .to_str()
            .expect("utf8")
            .to_owned();
        let _body = init_resp.text().await.expect("body");

        // Wait for session to expire
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Now the session should be gone — next request returns 404
        let expired_resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .header("mcp-session-id", &session_id)
            .body(LIST_TOOLS_BODY)
            .send()
            .await
            .expect("post after expiry");

        assert_eq!(
            expired_resp.status(),
            404,
            "expired session should return 404"
        );

        ct.cancel();
    }

    /// A session "forgotten" after N requests simulates session loss mid-stream.
    #[tokio::test]
    async fn test_http_mock_session_forget_after_n_requests() {
        let config = HttpMockConfig {
            session: SessionConfig {
                ttl_ms: None,
                forget_after_n_requests: Some(2), // forget after 2nd request
            },
            ..Default::default()
        };
        let (client, url, ct) = spawn_server(config).await;

        // Initialize
        let init_resp = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .body(INIT_BODY)
            .send()
            .await
            .expect("init");
        assert_eq!(init_resp.status(), 200);
        let session_id = init_resp
            .headers()
            .get("mcp-session-id")
            .expect("mcp-session-id")
            .to_str()
            .expect("utf8")
            .to_owned();
        let _body = init_resp.text().await.expect("body");

        // Send initialized notification (doesn't count against request limit —
        // it's a notification, not a stream request)
        let _ = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .header("mcp-session-id", &session_id)
            .body(INIT_NOTIFICATION)
            .send()
            .await
            .expect("notification");

        // First RPC request — should succeed (count = 1)
        let resp1 = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .header("mcp-session-id", &session_id)
            .body(LIST_TOOLS_BODY)
            .send()
            .await
            .expect("req1");
        assert_eq!(resp1.status(), 200, "first request should succeed");
        let _body1 = resp1.text().await.expect("body1");

        // Second RPC request — triggers forget (count = 2) then tries to
        // respond; the session is evicted. We may get a 200 (already dispatched)
        // or we may get a 404 on the next request. The important assertion is
        // that *after* this, the session is gone.
        let resp2 = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .header("mcp-session-id", &session_id)
            .body(LIST_TOOLS_BODY)
            .send()
            .await
            .expect("req2");
        let _body2 = resp2.text().await.expect("body2");

        // Third request — session is now gone, must return 404
        let resp3 = client
            .post(&url)
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .header("mcp-session-id", &session_id)
            .body(LIST_RESOURCES_BODY)
            .send()
            .await
            .expect("req3");
        assert_eq!(
            resp3.status(),
            404,
            "session should be gone after forget_after_n_requests"
        );

        ct.cancel();
    }
}
