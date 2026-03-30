//! Unit tests for STORY-005: Dual-Schema Config Parsing
//!
//! Naming convention: `test_BC_1_01_002_<description>`
//! All tests exercise `parse_config()` with inline JSON — no filesystem I/O.

use std::path::PathBuf;

use forge_core::types::{EditorKind, Transport, TransportConfig};
use forge_discovery::{parse_config, ConfigError, EditorKind as DiscoveryEditorKind, ParseInput};

// ── Helpers ───────────────────────────────────────────────────────────────────

fn claude_input(json: &str) -> ParseInput {
    ParseInput::new(json, "/home/user/.config/Claude/claude_desktop_config.json", DiscoveryEditorKind::ClaudeDesktop)
}

fn cursor_input(json: &str) -> ParseInput {
    ParseInput::new(json, "/home/user/.cursor/mcp.json", DiscoveryEditorKind::Cursor)
}

fn vscode_input(json: &str) -> ParseInput {
    ParseInput::new(json, "/home/user/.config/Code/User/mcp.json", DiscoveryEditorKind::VSCode)
}

fn windsurf_input(json: &str) -> ParseInput {
    ParseInput::new(json, "/home/user/.codeium/windsurf/mcp_config.json", DiscoveryEditorKind::Windsurf)
}

// ── AC-001: mcpServers schema — Stdio transport ───────────────────────────────

/// AC-001 / POST-001: Claude Desktop stdio entry parsed correctly.
#[test]
fn test_bc_1_01_002_mcp_servers_schema_stdio() {
    let json = r#"{
        "mcpServers": {
            "my-server": {
                "command": "npx",
                "args": ["-y", "@modelcontextprotocol/server-github"],
                "env": {
                    "GITHUB_TOKEN": "ghp_secret"
                }
            }
        }
    }"#;

    let entries = parse_config(&claude_input(json)).expect("should parse");
    assert_eq!(entries.len(), 1);

    let entry = &entries[0];
    assert_eq!(entry.name, "my-server");
    assert_eq!(entry.transport, Transport::Stdio);
    assert_eq!(entry.source_editor, EditorKind::ClaudeDesktop);
    assert!(entry.enabled);
    assert!(entry.always_allow.is_empty());

    match &entry.config {
        TransportConfig::Stdio(cfg) => {
            assert_eq!(cfg.command, "npx");
            assert_eq!(cfg.args, vec!["-y", "@modelcontextprotocol/server-github"]);
            assert_eq!(cfg.env.get("GITHUB_TOKEN").map(|s| s.as_str()), Some("ghp_secret"));
        }
        other => panic!("expected Stdio, got {other:?}"),
    }
}

/// AC-001 / POST-001: mcpServers schema with `url` field → Http transport.
#[test]
fn test_bc_1_01_002_mcp_servers_schema_http() {
    let json = r#"{
        "mcpServers": {
            "remote-server": {
                "url": "https://api.example.com/mcp"
            }
        }
    }"#;

    let entries = parse_config(&claude_input(json)).expect("should parse");
    assert_eq!(entries.len(), 1);

    let entry = &entries[0];
    assert_eq!(entry.name, "remote-server");
    assert_eq!(entry.transport, Transport::Http);

    match &entry.config {
        TransportConfig::Http(cfg) => {
            assert_eq!(cfg.url, "https://api.example.com/mcp");
            assert!(cfg.headers.is_empty());
        }
        other => panic!("expected Http, got {other:?}"),
    }
}

// ── AC-002: VS Code servers schema ───────────────────────────────────────────

/// AC-002 / POST-002: VS Code `servers` schema with explicit `"type": "stdio"`.
#[test]
fn test_bc_1_01_002_servers_schema_vscode() {
    let json = r#"{
        "servers": {
            "playwright": {
                "type": "stdio",
                "command": "npx",
                "args": ["-y", "@microsoft/mcp-server-playwright"]
            },
            "remote": {
                "type": "sse",
                "url": "https://api.example.com/mcp"
            }
        }
    }"#;

    let entries = parse_config(&vscode_input(json)).expect("should parse");
    assert_eq!(entries.len(), 2);

    let playwright = entries.iter().find(|e| e.name == "playwright").unwrap();
    assert_eq!(playwright.transport, Transport::Stdio);
    assert_eq!(playwright.source_editor, EditorKind::VSCode);
    match &playwright.config {
        TransportConfig::Stdio(cfg) => {
            assert_eq!(cfg.command, "npx");
            assert_eq!(cfg.args, vec!["-y", "@microsoft/mcp-server-playwright"]);
        }
        other => panic!("expected Stdio, got {other:?}"),
    }

    let remote = entries.iter().find(|e| e.name == "remote").unwrap();
    assert_eq!(remote.transport, Transport::Http);
    match &remote.config {
        TransportConfig::Http(cfg) => {
            assert_eq!(cfg.url, "https://api.example.com/mcp");
        }
        other => panic!("expected Http, got {other:?}"),
    }
}

/// AC-002 extended: VS Code `"type": "http"` (alternative spelling) also maps to Http.
#[test]
fn test_bc_1_01_002_servers_schema_vscode_http_type() {
    let json = r#"{
        "servers": {
            "api-server": {
                "type": "http",
                "url": "https://mcp.example.com"
            }
        }
    }"#;

    let entries = parse_config(&vscode_input(json)).expect("should parse");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].transport, Transport::Http);
}

// ── AC-003: ServerEntry has all required fields ───────────────────────────────

/// AC-003 / POST-003: All required fields present on every ServerEntry.
#[test]
fn test_bc_1_01_002_server_entry_all_fields() {
    let json = r#"{
        "mcpServers": {
            "test-server": {
                "command": "node",
                "args": ["server.js"],
                "env": {"DEBUG": "1"}
            }
        }
    }"#;

    let entries = parse_config(&claude_input(json)).unwrap();
    let entry = &entries[0];

    // All fields must be accessible
    let _name: &str = &entry.name;
    let _transport: &Transport = &entry.transport;
    let _config: &TransportConfig = &entry.config;
    let _source_editor: &EditorKind = &entry.source_editor;
    let _source_path: &PathBuf = &entry.source_path;
    let _enabled: bool = entry.enabled;
    let _always_allow: &Vec<String> = &entry.always_allow;

    assert_eq!(entry.name, "test-server");
    assert_eq!(entry.source_editor, EditorKind::ClaudeDesktop);
    assert_eq!(
        entry.source_path,
        PathBuf::from("/home/user/.config/Claude/claude_desktop_config.json")
    );
    assert!(entry.enabled, "default enabled=true");
    assert!(entry.always_allow.is_empty(), "default always_allow=[]");
}

// ── AC-004: disabled → enabled=false ─────────────────────────────────────────

/// AC-004 / POST-004: Cursor `disabled: true` maps to `enabled = false`.
#[test]
fn test_bc_1_01_002_disabled_maps_to_enabled_false() {
    let json = r#"{
        "mcpServers": {
            "disabled-server": {
                "command": "node",
                "args": [],
                "disabled": true
            },
            "active-server": {
                "command": "python",
                "args": [],
                "disabled": false
            }
        }
    }"#;

    let entries = parse_config(&cursor_input(json)).unwrap();
    assert_eq!(entries.len(), 2);

    let disabled = entries.iter().find(|e| e.name == "disabled-server").unwrap();
    assert!(!disabled.enabled, "disabled=true must map to enabled=false");

    let active = entries.iter().find(|e| e.name == "active-server").unwrap();
    assert!(active.enabled, "disabled=false must map to enabled=true");
}

/// AC-004 extended: server still appears in registry when disabled.
#[test]
fn test_bc_1_01_002_disabled_server_still_in_registry() {
    let json = r#"{
        "mcpServers": {
            "off": { "command": "node", "args": [], "disabled": true }
        }
    }"#;

    let entries = parse_config(&cursor_input(json)).unwrap();
    assert_eq!(entries.len(), 1, "disabled server must still appear in registry");
    assert!(!entries[0].enabled);
}

// ── AC-005: alwaysAllow mapping ───────────────────────────────────────────────

/// AC-005 / POST-005: Cursor `alwaysAllow` field maps to `always_allow`.
#[test]
fn test_bc_1_01_002_always_allow_mapping() {
    let json = r#"{
        "mcpServers": {
            "github": {
                "command": "npx",
                "args": ["-y", "@modelcontextprotocol/server-github"],
                "env": {},
                "alwaysAllow": ["list_repos", "get_file"]
            }
        }
    }"#;

    let entries = parse_config(&cursor_input(json)).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].always_allow, vec!["list_repos", "get_file"]);
}

// ── AC-006: env not expanded ──────────────────────────────────────────────────

/// AC-006 / POST-006: `env` values stored as-is; no variable expansion.
#[test]
fn test_bc_1_01_002_env_not_expanded() {
    let json = r#"{
        "mcpServers": {
            "server": {
                "command": "npx",
                "args": [],
                "env": {
                    "API_KEY": "${MY_SECRET}",
                    "HOME": "$HOME",
                    "LITERAL": "just-a-string"
                }
            }
        }
    }"#;

    let entries = parse_config(&claude_input(json)).unwrap();
    let env = match &entries[0].config {
        TransportConfig::Stdio(cfg) => &cfg.env,
        other => panic!("expected Stdio, got {other:?}"),
    };

    // Values must be stored verbatim — no expansion
    assert_eq!(env.get("API_KEY").map(|s| s.as_str()), Some("${MY_SECRET}"));
    assert_eq!(env.get("HOME").map(|s| s.as_str()), Some("$HOME"));
    assert_eq!(env.get("LITERAL").map(|s| s.as_str()), Some("just-a-string"));
}

// ── AC-007: invalid JSON → E-CFG-003 ─────────────────────────────────────────

/// AC-007 / EC-002: Invalid JSON returns E-CFG-003. No partial results.
#[test]
fn test_bc_1_01_002_invalid_json_errors() {
    let bad_jsons = [
        r#"{ this is not json }"#,
        r#"{"mcpServers": {unclosed"#,
        r#""#,
        r#"null"#,  // null is valid JSON but has no keys
    ];

    // First two must be JsonParseError; last two handled separately
    for bad in &bad_jsons[..2] {
        let result = parse_config(&claude_input(bad));
        assert!(result.is_err(), "expected error for: {bad}");
        match result.unwrap_err() {
            ConfigError::JsonParseError { path, .. } => {
                let msg = format!("{}", ConfigError::JsonParseError {
                    path: path.clone(),
                    source: serde_json::from_str::<serde_json::Value>("bad").unwrap_err(),
                });
                assert!(
                    msg.contains("E-CFG-003"),
                    "error message must contain E-CFG-003"
                );
            }
            other => panic!("expected JsonParseError, got {other:?}"),
        }
    }

    // Empty string is a JSON parse error
    let result = parse_config(&claude_input(""));
    assert!(
        matches!(result, Err(ConfigError::JsonParseError { .. })),
        "empty string must be JsonParseError"
    );
}

/// Error message for E-CFG-003 contains the error code.
#[test]
fn test_bc_1_01_002_invalid_json_error_message_contains_code() {
    let result = parse_config(&claude_input("{bad}"));
    let err = result.unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("E-CFG-003"), "error must contain E-CFG-003, got: {msg}");
}

// ── AC-008: no recognized schema → E-CFG-004, continue ───────────────────────

/// AC-008 / EC-003 / SR-004: Valid JSON with neither `mcpServers` nor `servers`
/// returns E-CFG-004. Processing of other config files continues (non-fatal).
#[test]
fn test_bc_1_01_002_no_recognized_schema_continues() {
    let json = r#"{ "something_else": { "irrelevant": true } }"#;
    let result = parse_config(&claude_input(json));
    match result {
        Err(ConfigError::NoRecognizedSchema { path }) => {
            let err = ConfigError::NoRecognizedSchema { path };
            let msg = err.to_string();
            assert!(msg.contains("E-CFG-004"), "must contain E-CFG-004, got: {msg}");
        }
        Ok(_) => panic!("expected NoRecognizedSchema error"),
        Err(other) => panic!("expected NoRecognizedSchema, got {other:?}"),
    }
}

/// Empty JSON object (no keys) → E-CFG-004.
#[test]
fn test_bc_1_01_002_empty_object_no_schema() {
    let result = parse_config(&claude_input("{}"));
    assert!(
        matches!(result, Err(ConfigError::NoRecognizedSchema { .. })),
        "empty object must yield NoRecognizedSchema"
    );
}

// ── AC-009: unknown fields ignored ───────────────────────────────────────────

/// AC-009 / EC-006: Unknown fields in server entries are silently ignored.
#[test]
fn test_bc_1_01_002_unknown_fields_ignored() {
    let json = r#"{
        "mcpServers": {
            "server": {
                "command": "node",
                "args": ["server.js"],
                "unknownFutureField": "some-value",
                "anotherNewField": 42
            }
        }
    }"#;

    // Must not error
    let entries = parse_config(&claude_input(json)).expect("unknown fields must be ignored");
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].name, "server");
}

/// AC-009 extended: unknown fields in VS Code servers schema also ignored.
#[test]
fn test_bc_1_01_002_unknown_fields_vscode_ignored() {
    let json = r#"{
        "servers": {
            "server": {
                "type": "stdio",
                "command": "node",
                "args": [],
                "futureVscodeField": true
            }
        }
    }"#;

    let entries = parse_config(&vscode_input(json)).expect("unknown fields must be ignored");
    assert_eq!(entries.len(), 1);
}

// ── AC-010: non-string env value → E-CFG-007, per-entry error ────────────────

/// AC-010 / EC-005: `env` value that is not a string returns E-CFG-007.
#[test]
fn test_bc_1_01_002_env_non_string_per_entry_error() {
    let json = r#"{
        "mcpServers": {
            "bad-server": {
                "command": "node",
                "args": [],
                "env": {
                    "VALID_KEY": "valid-string",
                    "BAD_KEY": 42
                }
            }
        }
    }"#;

    let result = parse_config(&claude_input(json));
    match result {
        Err(ConfigError::EnvValueNotString { key, server, .. }) => {
            assert_eq!(key, "BAD_KEY");
            assert_eq!(server, "bad-server");
            let err = ConfigError::EnvValueNotString {
                path: PathBuf::from("/tmp/test.json"),
                server: "bad-server".to_owned(),
                key: "BAD_KEY".to_owned(),
            };
            let msg = err.to_string();
            assert!(msg.contains("E-CFG-007"), "must contain E-CFG-007, got: {msg}");
        }
        Ok(_) => panic!("expected EnvValueNotString error"),
        Err(other) => panic!("expected EnvValueNotString, got {other:?}"),
    }
}

/// AC-010: boolean env value also triggers E-CFG-007.
#[test]
fn test_bc_1_01_002_env_bool_value_error() {
    let json = r#"{
        "mcpServers": {
            "server": {
                "command": "node",
                "args": [],
                "env": { "FLAG": true }
            }
        }
    }"#;

    assert!(
        matches!(
            parse_config(&claude_input(json)),
            Err(ConfigError::EnvValueNotString { .. })
        )
    );
}

/// AC-010: null env value also triggers E-CFG-007.
#[test]
fn test_bc_1_01_002_env_null_value_error() {
    let json = r#"{
        "mcpServers": {
            "server": {
                "command": "node",
                "args": [],
                "env": { "KEY": null }
            }
        }
    }"#;

    assert!(
        matches!(
            parse_config(&claude_input(json)),
            Err(ConfigError::EnvValueNotString { .. })
        )
    );
}

// ── Additional coverage ───────────────────────────────────────────────────────

/// Cursor global config — full realistic example.
#[test]
fn test_bc_1_01_002_cursor_global_full_example() {
    let json = r#"{
        "mcpServers": {
            "github": {
                "command": "npx",
                "args": ["-y", "@modelcontextprotocol/server-github"],
                "env": {
                    "GITHUB_PERSONAL_ACCESS_TOKEN": "ghp_abc123"
                },
                "disabled": false,
                "alwaysAllow": ["list_repos", "read_file"]
            },
            "disabled-tool": {
                "command": "node",
                "args": ["/path/to/server.js"],
                "disabled": true
            }
        }
    }"#;

    let entries = parse_config(&cursor_input(json)).unwrap();
    assert_eq!(entries.len(), 2);

    let github = entries.iter().find(|e| e.name == "github").unwrap();
    assert_eq!(github.source_editor, EditorKind::Cursor);
    assert!(github.enabled);
    assert_eq!(github.always_allow, vec!["list_repos", "read_file"]);

    match &github.config {
        TransportConfig::Stdio(cfg) => {
            assert_eq!(cfg.env.get("GITHUB_PERSONAL_ACCESS_TOKEN").map(|s| s.as_str()), Some("ghp_abc123"));
        }
        other => panic!("expected Stdio, got {other:?}"),
    }

    let disabled = entries.iter().find(|e| e.name == "disabled-tool").unwrap();
    assert!(!disabled.enabled);
}

/// Windsurf config (same mcpServers schema as Claude Desktop).
#[test]
fn test_bc_1_01_002_windsurf_config_parsed() {
    let json = r#"{
        "mcpServers": {
            "maps": {
                "command": "npx",
                "args": ["-y", "@modelcontextprotocol/server-google-maps"],
                "env": {
                    "GOOGLE_MAPS_API_KEY": "key123"
                }
            }
        }
    }"#;

    let entries = parse_config(&windsurf_input(json)).unwrap();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].source_editor, EditorKind::Windsurf);
    assert_eq!(entries[0].source_path, PathBuf::from("/home/user/.codeium/windsurf/mcp_config.json"));
}

/// Multiple servers in a single config file.
#[test]
fn test_bc_1_01_002_multiple_servers_parsed() {
    let json = r#"{
        "mcpServers": {
            "server-a": { "command": "node", "args": ["a.js"] },
            "server-b": { "command": "python", "args": ["b.py"] },
            "server-c": { "url": "https://c.example.com/mcp" }
        }
    }"#;

    let entries = parse_config(&claude_input(json)).unwrap();
    assert_eq!(entries.len(), 3);

    let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
    assert!(names.contains(&"server-a"));
    assert!(names.contains(&"server-b"));
    assert!(names.contains(&"server-c"));
}

/// Empty mcpServers object → empty list (not an error).
#[test]
fn test_bc_1_01_002_empty_mcp_servers_ok() {
    let json = r#"{ "mcpServers": {} }"#;
    let entries = parse_config(&claude_input(json)).unwrap();
    assert!(entries.is_empty(), "empty mcpServers must yield empty list");
}

/// Empty servers object → empty list (not an error).
#[test]
fn test_bc_1_01_002_empty_servers_ok() {
    let json = r#"{ "servers": {} }"#;
    let entries = parse_config(&vscode_input(json)).unwrap();
    assert!(entries.is_empty(), "empty servers must yield empty list");
}

/// EC-001: mcpServers with both `command` AND `url` — command wins (Stdio).
#[test]
fn test_bc_1_01_002_both_command_and_url_command_wins() {
    let json = r#"{
        "mcpServers": {
            "ambiguous": {
                "command": "node",
                "args": ["server.js"],
                "url": "https://example.com/mcp"
            }
        }
    }"#;

    let entries = parse_config(&claude_input(json)).unwrap();
    assert_eq!(entries[0].transport, Transport::Stdio, "command takes precedence over url in mcpServers schema");
}

/// Source path is preserved accurately in every entry.
#[test]
fn test_bc_1_01_002_source_attribution_preserved() {
    let json = r#"{
        "mcpServers": {
            "s1": { "command": "node", "args": [] },
            "s2": { "command": "node", "args": [] }
        }
    }"#;

    let input = ParseInput::new(json, "/specific/path/mcp.json", DiscoveryEditorKind::Cursor);
    let entries = parse_config(&input).unwrap();

    for entry in &entries {
        assert_eq!(entry.source_path, PathBuf::from("/specific/path/mcp.json"));
        assert_eq!(entry.source_editor, EditorKind::Cursor);
    }
}

/// VS Code sse server with headers.
#[test]
fn test_bc_1_01_002_vscode_sse_with_headers() {
    let json = r#"{
        "servers": {
            "remote": {
                "type": "sse",
                "url": "https://mcp.example.com/v1",
                "headers": {
                    "Authorization": "Bearer token123",
                    "X-Custom": "value"
                }
            }
        }
    }"#;

    let entries = parse_config(&vscode_input(json)).unwrap();
    assert_eq!(entries.len(), 1);
    match &entries[0].config {
        TransportConfig::Http(cfg) => {
            assert_eq!(cfg.url, "https://mcp.example.com/v1");
            assert_eq!(cfg.headers.get("Authorization").map(|s| s.as_str()), Some("Bearer token123"));
            assert_eq!(cfg.headers.get("X-Custom").map(|s| s.as_str()), Some("value"));
        }
        other => panic!("expected Http, got {other:?}"),
    }
}

/// Args default to empty vec when missing.
#[test]
fn test_bc_1_01_002_args_default_empty() {
    let json = r#"{
        "mcpServers": {
            "server": { "command": "node" }
        }
    }"#;

    let entries = parse_config(&claude_input(json)).unwrap();
    match &entries[0].config {
        TransportConfig::Stdio(cfg) => {
            assert!(cfg.args.is_empty(), "args default to empty");
        }
        other => panic!("expected Stdio, got {other:?}"),
    }
}

/// Env defaults to empty map when missing.
#[test]
fn test_bc_1_01_002_env_default_empty() {
    let json = r#"{
        "mcpServers": {
            "server": { "command": "node", "args": [] }
        }
    }"#;

    let entries = parse_config(&claude_input(json)).unwrap();
    match &entries[0].config {
        TransportConfig::Stdio(cfg) => {
            assert!(cfg.env.is_empty(), "env defaults to empty map");
        }
        other => panic!("expected Stdio, got {other:?}"),
    }
}
