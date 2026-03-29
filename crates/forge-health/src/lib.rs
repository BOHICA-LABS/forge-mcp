//! # forge-health
//!
//! MCP server health monitoring and alerting for Forge MCP.
//!
//! This is an L2 crate — it depends on `forge-traffic` (L1).
//!
//! ## Responsibilities
//! - Compute health scores from traffic observations (error rates, latency percentiles)
//! - Detect server degradation: timeouts, error spikes, connection drops
//! - Provide a `HealthStatus` enum: Healthy, Degraded, Unhealthy, Unknown
//! - Support configurable alert thresholds and notification hooks
//! - Expose a time-series ring buffer for recent health snapshots
