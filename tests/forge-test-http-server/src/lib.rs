//! Library entrypoint for forge-test-http-server.
//!
//! Exposes `MockHttpServer` and config types for use in integration tests
//! of other crates (e.g., forge-core's HTTP transport tests).

pub mod config;
pub mod server;

#[cfg(test)]
mod tests;
