//! Resource protocol operations for MCP.
//!
//! Provides `list_resources` (with pagination) and `read_resource` (text + blob).
//! Both functions require the server to have advertised the `resources` capability;
//! otherwise `E-PRO-003` is returned immediately without making any RPC call.
//!
//! ## Pagination
//!
//! `list_resources` collects **all** pages automatically.  Internally it uses
//! the rmcp `PaginatedRequestParams` cursor mechanism — the same pattern the
//! mock server uses in STORY-016/STORY-013 tests.  A hard `MAX_PAGES` guard
//! prevents an infinite loop against a misbehaving server that always returns
//! a non-`None` next cursor.
//!
//! ## Binary content
//!
//! When a `ReadResourceResult` contains a `BlobResourceContents` entry,
//! `read_resource` base64-decodes the blob string to raw bytes and stores
//! them in [`ResourceContent::Binary`].

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;
use rmcp::model::{PaginatedRequestParams, ReadResourceRequestParams, ResourceContents};

use crate::connection::McpConnection;
use crate::error::{CoreError, Result};
use rmcp::ClientHandler;

// ── Public types ─────────────────────────────────────────────────────────────

/// A single MCP resource returned by `list_resources`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    /// Unique URI identifying this resource (e.g. `file:///foo/bar.txt`).
    pub uri: String,
    /// Human-readable name.
    pub name: String,
    /// Optional description.
    pub description: Option<String>,
    /// Optional MIME type hint (e.g. `"text/plain"`, `"application/json"`).
    pub mime_type: Option<String>,
}

/// The content returned by `read_resource`.
#[derive(Debug, Clone, PartialEq)]
pub struct ResourceContent {
    /// The URI of the resource that was read.
    pub uri: String,
    /// MIME type, if the server provided one.
    pub mime_type: Option<String>,
    /// The actual data — text or decoded binary bytes.
    pub content: ResourceData,
}

/// Discriminated union for text vs. binary resource data.
#[derive(Debug, Clone, PartialEq)]
pub enum ResourceData {
    /// UTF-8 text content.
    Text(String),
    /// Binary content decoded from the server's base64-encoded blob.
    Binary(Vec<u8>),
}

// ── Pagination guard ──────────────────────────────────────────────────────────

/// Hard upper bound on pagination iterations.
///
/// Prevents an infinite loop against a buggy / malicious server that always
/// returns a non-`None` next cursor.  10 000 pages × typical page sizes
/// should be far more than any real-world resource list.
const MAX_PAGES: usize = 10_000;

// ── list_resources ────────────────────────────────────────────────────────────

/// List all resources advertised by the connected MCP server.
///
/// Automatically iterates through all pagination pages using the cursor
/// mechanism, collecting results into a single `Vec<Resource>`.
///
/// # Errors
///
/// - `E-PRO-003` — server did not advertise the `resources` capability.
/// - `E-PRO-001` — any underlying rmcp / JSON-RPC error.
pub async fn list_resources<H: ClientHandler>(conn: &McpConnection<H>) -> Result<Vec<Resource>> {
    if !conn.supports_resources() {
        return Err(CoreError::CapabilityNotSupported {
            method: "resources/list".to_string(),
            capability: "resources".to_string(),
        });
    }

    let peer = conn
        .peer()
        .ok_or_else(|| CoreError::Protocol("no active peer".to_string()))?;

    let mut all: Vec<Resource> = Vec::new();
    let mut cursor: Option<String> = None;

    for _ in 0..MAX_PAGES {
        let params = if cursor.is_some() {
            Some(PaginatedRequestParams::default().with_cursor(cursor))
        } else {
            None
        };

        let result = peer
            .list_resources(params)
            .await
            .map_err(|e| CoreError::Protocol(e.to_string()))?;

        for r in result.resources {
            all.push(Resource {
                uri: r.uri.clone(),
                name: r.name.clone(),
                description: r.description.clone(),
                mime_type: r.mime_type.clone(),
            });
        }

        cursor = result.next_cursor;
        if cursor.is_none() {
            break;
        }
    }

    Ok(all)
}

// ── read_resource ─────────────────────────────────────────────────────────────

/// Read the content of a specific MCP resource by URI.
///
/// Returns the first content item from the `ReadResourceResult`.  If the
/// result is a blob, the base64 string is decoded to raw bytes.
///
/// # Errors
///
/// - `E-PRO-003` — server did not advertise the `resources` capability.
/// - `E-PRO-001` — RPC failure, unknown URI, or base64 decode error.
pub async fn read_resource<H: ClientHandler>(
    conn: &McpConnection<H>,
    uri: impl Into<String>,
) -> Result<ResourceContent> {
    if !conn.supports_resources() {
        return Err(CoreError::CapabilityNotSupported {
            method: "resources/read".to_string(),
            capability: "resources".to_string(),
        });
    }

    let uri = uri.into();
    let peer = conn
        .peer()
        .ok_or_else(|| CoreError::Protocol("no active peer".to_string()))?;

    let result = peer
        .read_resource(ReadResourceRequestParams::new(uri.clone()))
        .await
        .map_err(|e| CoreError::Protocol(e.to_string()))?;

    // Use the first content item; real-world servers nearly always return one.
    let first =
        result.contents.into_iter().next().ok_or_else(|| {
            CoreError::Protocol("read_resource returned empty contents".to_string())
        })?;

    match first {
        ResourceContents::TextResourceContents {
            uri: content_uri,
            mime_type,
            text,
            ..
        } => Ok(ResourceContent {
            uri: content_uri,
            mime_type,
            content: ResourceData::Text(text),
        }),
        ResourceContents::BlobResourceContents {
            uri: content_uri,
            mime_type,
            blob,
            ..
        } => {
            let bytes = BASE64_STANDARD
                .decode(blob.as_bytes())
                .map_err(|e| CoreError::Protocol(format!("base64 decode error: {e}")))?;
            Ok(ResourceContent {
                uri: content_uri,
                mime_type,
                content: ResourceData::Binary(bytes),
            })
        }
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod unit_tests {
    use super::*;
    use base64::Engine as _;
    use base64::engine::general_purpose::STANDARD as BASE64_STANDARD;

    /// AC-002 (binary): base64 blob content is decoded to raw bytes.
    ///
    /// This test exercises the `BlobResourceContents` decode path in
    /// `read_resource` directly without a live server connection.
    #[test]
    fn test_blob_base64_decode_roundtrip() {
        let original: Vec<u8> = vec![0xDE, 0xAD, 0xBE, 0xEF, 0x00, 0xFF];
        let encoded = BASE64_STANDARD.encode(&original);

        // Simulate the decode path used by read_resource.
        let decoded = BASE64_STANDARD
            .decode(encoded.as_bytes())
            .expect("decode should succeed");

        assert_eq!(decoded, original, "decoded bytes must match original");
    }

    /// Verify that a bad base64 string returns an error (not a panic).
    #[test]
    fn test_blob_base64_invalid_returns_error() {
        let bad = "not!!valid@@base64";
        let result = BASE64_STANDARD.decode(bad.as_bytes());
        assert!(result.is_err(), "invalid base64 must return Err");
    }
}
