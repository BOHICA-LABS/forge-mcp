---
document_type: domain-spec-section
level: L2
section: failure-modes
version: "1.0"
status: draft
producer: business-analyst
timestamp: 2026-03-29T10:55:00
phase: 1a
inputs: [product-brief.md, market-intel.md]
input-hash: ""
traces_to: L2-INDEX.md
---

# Failure Modes

> **Sharded L2 section (DF-021).** Navigate via `L2-INDEX.md`.
> Failure modes describe how the system fails and how it should recover.
> Grouped by subsystem.

## Connection Subsystem

**FM-001 — DNS resolution failure.** HTTP transport target hostname cannot be resolved. Symptoms: ConnectionRequested followed by immediate error, no ConnectionEstablished. Detection: rmcp transport returns DNS error. Recovery: Report clear error with hostname and suggest checking network/DNS config. Retry with exponential backoff (max 3 attempts). Severity: blocks connection to that server only. Traces to: CAP-002.

**FM-002 — Server process crash.** Stdio-connected server process terminates unexpectedly (SIGKILL, SIGSEGV, OOM). Symptoms: EOF on stdin/stdout pipes, pending requests never complete. Detection: rmcp transport detects broken pipe or process exit. Recovery: Fire ConnectionLost event, transition server to error state, notify user. Optional auto-reconnect with backoff. Record crash in health metrics. Severity: session-level — other sessions unaffected. Traces to: CAP-002, DEC-001.

**FM-003 — Transport timeout.** Server connected but unresponsive — no data for configured timeout period. Symptoms: Requests hang, TUI shows pending indicator. Detection: Request-level timeout fires. Recovery: Cancel pending request, report timeout to user, keep connection alive for potential recovery. If multiple timeouts in sequence, suggest server health issue. Severity: request-level, recoverable. Traces to: CAP-002, CAP-005, DEC-005.

**FM-004 — TLS/certificate error.** HTTPS transport to MCP server fails TLS handshake (expired cert, self-signed, hostname mismatch). Symptoms: Connection fails immediately after TCP connect. Detection: rmcp or HTTP client reports TLS error. Recovery: Report specific TLS error type (expired, self-signed, mismatch). Do not silently downgrade to HTTP. Optionally allow user to accept risk for development servers (with explicit flag). Severity: blocks connection. Traces to: CAP-002.

**FM-005 — Daemon socket conflict.** Daemon cannot bind its listening socket (address in use from previous instance or another process). Symptoms: Forge MCP startup fails with "address in use" error. Detection: Socket bind error. Recovery: Detect stale lock file or process, attempt cleanup. If another Forge MCP daemon is healthy, connect to it. If unresponsive, offer to kill and restart. Severity: blocks all sessions, recoverable with user action. Traces to: CAP-003, DEC-006.

## Config Subsystem

**FM-006 — Config file permissions.** Config file exists but is not readable (wrong permissions, encrypted volume unmounted). Symptoms: Config source shows status=error instead of expected servers. Detection: File open returns permission error. Recovery: Report specific permission error with file path. Suggest remediation (chmod, mount). Skip this source and continue with others. Severity: degraded discovery, non-blocking. Traces to: CAP-001.

**FM-007 — Config JSON parse error.** Config file contains malformed JSON (truncated write, editor crash during save, encoding issue). Symptoms: Config source shows status=invalid with parse error details. Detection: JSON parser returns error with line/column. Recovery: Report parse error with location. Show last-known-good config if available. Skip this source. Severity: degraded discovery, non-blocking. Traces to: CAP-001, DEC-008.

**FM-008 — Config schema mismatch.** Valid JSON but unexpected schema (editor updated config format). Symptoms: Config parses as JSON but server entries cannot be extracted. Detection: Schema validation fails or extraction yields zero servers from non-empty file. Recovery: Report schema version mismatch. Suggest updating Forge MCP or checking editor version. Severity: degraded discovery, non-blocking. Traces to: CAP-001, ASM-002, R-006.

**FM-009 — Config file not found.** Expected config path does not exist (editor not installed). Symptoms: Config source shows status=missing. Detection: File access returns ENOENT. Recovery: Silent skip — this is expected when not all editors are installed. Include in `list` output which sources were searched and which were found. Severity: informational. Traces to: CAP-001, DEC-007.

## Traffic Subsystem

**FM-010 — Capture buffer overflow.** Traffic capture grows beyond configured memory limit. Symptoms: Memory usage approaches limit, system may become sluggish. Detection: Buffer size exceeds configured threshold. Recovery: Rotate oldest messages to disk or discard based on configured policy (FIFO eviction). Warn user that capture is no longer complete. Never silently drop messages without notification (DI-005 requires capture integrity). Severity: degraded capture, recoverable. Traces to: CAP-009, R-010, DEC-011.

**FM-011 — Replay target unreachable.** User initiates replay but the designated target server is not connected. Symptoms: Replay command returns error immediately. Detection: Target server lookup fails or connection check fails. Recovery: Report "target server not connected" with suggestion to connect first. Do not fall back to any other server (DI-007). Severity: replay blocked, expected error. Traces to: CAP-010, DI-007.

**FM-012 — Out-of-memory from unbounded capture.** System-level OOM from unbounded traffic capture accumulation over a long session. Symptoms: Process killed by OS OOM killer, or allocation failure. Detection: Memory monitoring (if implemented) or process termination. Recovery: Prevention is primary — enforce capture buffer limits by default (R-010). If OOM occurs, daemon restart with bounded defaults. Session data is lost. Severity: critical, preventable. Traces to: CAP-009, R-010.

## TUI Subsystem

**FM-013 — Terminal capability detection failure.** Terminal does not support expected features (truecolor, mouse, Unicode). Symptoms: Garbled rendering, missing characters, broken layout. Detection: Terminal capability query returns unexpected values. Recovery: Graceful degradation chain: truecolor → 256-color → 16-color. Unicode box-drawing → ASCII fallback. Mouse → keyboard-only. Report detected terminal capabilities at startup with `--verbose`. Severity: degraded UX, functional. Traces to: CAP-006, CAP-008.

**FM-014 — Terminal resize during render.** User resizes terminal window while TUI is rendering a frame. Symptoms: Momentary layout glitch or partial render. Detection: SIGWINCH signal (Unix) or console resize event (Windows). Recovery: Abort current frame, recalculate layout for new dimensions, render fresh frame. Must handle rapid successive resizes without accumulating layout debt. Severity: cosmetic, transient. Traces to: CAP-006.

## Security Subsystem

**FM-015 — Heuristic rule load failure.** Security heuristic rules fail to load (corrupted file, incompatible version). Symptoms: Security auditing is non-functional — no findings generated. Detection: Rule loading returns error. Recovery: Report which rules failed to load and why. Continue with successfully loaded rules. If zero rules load, disable security auditing with clear warning rather than producing false sense of security. Severity: degraded security coverage, non-blocking. Traces to: CAP-016.
