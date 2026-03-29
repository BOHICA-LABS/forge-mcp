# Toolchain Preflight Report — forge-mcp

**Generated:** 2026-03-29  
**Agent:** DX Engineer (pre-pipeline preflight)  
**Project:** forge-mcp — Rust CLI for MCP server discovery, inspection, and security auditing  
**Pipeline Stage:** Pre-Phase-3 (before implementation begins)

---

## Summary

| Category | Installed | Missing | Blocking? |
|----------|-----------|---------|-----------|
| Rust Toolchain | 5/5 ✅ | 0 | No |
| Rust Targets (for cross-compile) | 1/5 ⚠️ | 4 targets | **YES** |
| Cross-compilation driver | 0/1 ❌ | `cross` | **YES** |
| Testing Tools | 2/2 ✅ | 0 | No |
| Formal Verification | 1/4 ⚠️ | kani, mutants | Advisory |
| Security Tools | 2/2 ✅ | 0 | No |
| LiteLLM Proxy | Reachable ✅ | — | No |
| MCP Servers | Partial ⚠️ | mcporter in PATH | Advisory |

---

## 1. Rust Toolchain

> **Requirement:** Rust 2024 edition (stable since 1.85). Nightly needed for cargo-kani.

| Tool | Status | Version | Notes |
|------|--------|---------|-------|
| `rustc` (Homebrew) | ✅ INSTALLED | 1.94.0 | Primary compiler on PATH |
| `rustc` (rustup stable) | ✅ INSTALLED | 1.93.1 | Via rustup; slightly behind Homebrew |
| `rustc` (rustup nightly) | ✅ INSTALLED | 1.96.0-nightly (2026-03-15) | Available via `rustup run nightly` |
| `cargo` (Homebrew) | ✅ INSTALLED | 1.94.0 | |
| `rustfmt` | ✅ INSTALLED | 1.8.0 | |
| `clippy` | ✅ INSTALLED | 0.1.94 | |
| `rustup` | ✅ INSTALLED | 1.27.1 | |

**⚠️ Toolchain Note — Version Discrepancy:**  
`rustc` on `$PATH` is **Homebrew-managed** (1.94.0) while `rustup`'s default stable is 1.93.1.  
This can cause subtle version mismatches. Recommendation: set `rustup default stable` and ensure `~/.cargo/bin` precedes `/opt/homebrew/bin` in `$PATH`, or pin via `rust-toolchain.toml`.

**✅ Rust 2024 Edition:** Both 1.93.1 and 1.94.0 exceed the 1.85 minimum for 2024 edition on stable. No blocker.

---

## 2. Cross-Compilation Targets

> **Requirement:** Linux x64/ARM64, macOS Intel/ARM64, Windows x64 — static binaries < 25MB stripped with LTO.

### 2a. rustup Targets (currently installed on stable toolchain)

| Target | Status |
|--------|--------|
| `aarch64-apple-darwin` | ✅ INSTALLED |
| `x86_64-apple-darwin` | ❌ MISSING |
| `x86_64-unknown-linux-gnu` | ❌ MISSING |
| `aarch64-unknown-linux-gnu` | ❌ MISSING |
| `x86_64-pc-windows-gnu` | ❌ MISSING |

All targets are **available in rustup** (`rustup target list` confirms them present in the registry) — they just need to be added to the active stable toolchain.

**Recommendation:**
```bash
rustup target add x86_64-apple-darwin
rustup target add x86_64-unknown-linux-gnu
rustup target add aarch64-unknown-linux-gnu
rustup target add x86_64-pc-windows-gnu
```

### 2b. Cross-Compilation Driver

| Tool | Status | Notes |
|------|--------|-------|
| `cross` | ❌ MISSING | Preferred driver for Linux cross-compile (uses Docker containers with correct linkers) |
| `cargo-zigbuild` | ❌ MISSING | Alternative using Zig as cross-linker — no Docker needed |
| `zig` | ❌ MISSING | Required by cargo-zigbuild |
| `musl-gcc` | ❌ MISSING | Required for static musl Linux builds without cross/zig |
| `x86_64-linux-gnu-gcc` | ❌ MISSING | Required for native Linux cross-linking |
| `aarch64-linux-gnu-gcc` | ❌ MISSING | Required for Linux ARM64 cross-linking |

**⛔ BLOCKING:** Cross-compilation for Linux targets requires one of: `cross` (Docker-based), `cargo-zigbuild` + `zig`, or native cross-linkers. None are installed.

**Recommendation (choose one):**
- **Option A — `cross` (most reliable):** `cargo install cross --locked` + Docker daemon must be running.
- **Option B — `cargo-zigbuild` (no Docker):** `cargo install cargo-zigbuild --locked` + `brew install zig`. Simpler for CI.
- Note: Docker 28.5.2 is installed ✅, so Option A is immediately viable.

### 2c. Static Binary Requirements

For `< 25MB` stripped + LTO targets:
- Linux static: requires `*-unknown-linux-musl` targets (not gnu) for truly static binaries. GNU targets dynamically link glibc.
- Recommend adding musl targets: `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl`
- Windows: `x86_64-pc-windows-gnu` links via MinGW (no MSVC on macOS host); acceptable for CI.

---

## 3. Testing Tools

| Tool | Status | Version | Notes |
|------|--------|---------|-------|
| `cargo-nextest` | ✅ INSTALLED | 0.9.129 | Preferred test runner — ready |
| `cargo-llvm-cov` | ✅ INSTALLED | 0.8.4 | Coverage — ready |

No blockers. Both tools are current.

---

## 4. Formal Verification Tools

| Tool | Status | Version | Notes |
|------|--------|---------|-------|
| `cargo-fuzz` | ✅ INSTALLED | 0.13.1 | Requires nightly toolchain at runtime — nightly is installed |
| `cargo-kani` | ❌ MISSING | — | Not installed globally; needs `cargo install --locked kani-verifier` |
| `cargo-mutants` | ❌ MISSING | — | Not in cargo install list |
| `proptest` (library) | ✅ IN CACHE | 1.10.0 | Found in `~/.cargo/registry`; add as `[dev-dependencies]` in Cargo.toml — not a global binary |

**Advisory (not pipeline-blocking for Phase 3, blocking for Phase 5):**

- **cargo-kani:** Kani requires nightly Rust; nightly toolchain (1.96.0-nightly) is installed and compatible. Install: `cargo install --locked kani-verifier && cargo kani setup`
- **cargo-mutants:** Install: `cargo install --locked cargo-mutants`

---

## 5. Security Tools

| Tool | Status | Version | Notes |
|------|--------|---------|-------|
| `cargo-audit` | ✅ INSTALLED | 0.22.1 | Audits dependencies against RustSec advisory DB |
| `cargo-deny` | ✅ INSTALLED | 0.19.0 | License + advisories + bans |
| `semgrep` | ✅ INSTALLED | 1.156.0 | Static analysis — current |

No blockers on security tooling. All required tools are present.

---

## 6. LiteLLM Proxy (localhost:4000)

| Check | Status | Detail |
|-------|--------|--------|
| Reachable | ✅ REACHABLE | HTTP 200 (Swagger UI served at `/`) |
| Auth | ⚠️ 401 | `GET /health` returns 401 — API key required |
| Models endpoint | ⚠️ 401 | Key needed for model listing |

**Assessment:** LiteLLM proxy is **running and accessible** at `localhost:4000`. The 401 on `/health` is expected when no API key is passed in the request — the proxy itself is alive. The Swagger UI responds at `/`. This is **NOT a blocking issue**; agents that call LiteLLM will use their configured API key from the environment.

---

## 7. MCP Connectivity

| Server | Status | Notes |
|--------|--------|-------|
| Perplexity MCP | ✅ ACCESSIBLE | Available via OpenClaw's built-in tools (confirmed working in this session) |
| Context7 MCP | ✅ ACCESSIBLE | Available via OpenClaw's built-in tools (confirmed working in this session) |
| `mcporter` CLI | ⚠️ NOT IN PATH | Binary not found on `$PATH`; skill directory exists at `/opt/homebrew/lib/node_modules/openclaw/skills/mcporter/` but contains only `SKILL.md` — no binary |

**Assessment:** MCP servers are reachable via OpenClaw's native tool bridge. The `mcporter` CLI is not installed as a standalone binary, but this does not block the pipeline since agents access MCP through OpenClaw's tool layer directly. Advisory only.

---

## 8. Supporting Infrastructure

| Tool | Status | Version | Notes |
|------|--------|---------|-------|
| `git` | ✅ INSTALLED | 2.50.1 | |
| `gh` CLI | ✅ INSTALLED | 2.83.2 | |
| Docker | ✅ INSTALLED | 28.5.2 | Required if using `cross` for cross-compilation |
| `direnv` | ✅ INSTALLED | 2.35.0 | Environment management — ready |
| `just` | ✅ INSTALLED | 1.43.1 | Task runner — ready |
| `vhs` | ✅ INSTALLED | 0.10.0 | Demo recording — ready |

### Additional Tools Found (cargo install list)
| Tool | Version | Notes |
|------|---------|-------|
| `cargo-insta` | 1.46.3 | Snapshot testing — useful for CLI output tests |
| `cargo-machete` | 0.9.1 | Detects unused dependencies |
| `cargo-depgraph-check` | 0.1.0 | Dependency graph analysis |
| `taplo-cli` | 0.9.3 | TOML formatter/linter — useful for Cargo.toml |
| `git-cliff` | 2.12.0 | CHANGELOG generation — useful for release pipeline |

---

## Blocking Issues

| # | Issue | Severity | Blocks |
|---|-------|----------|--------|
| 1 | Cross-compilation driver missing (`cross`, `cargo-zigbuild`, or cross-linkers) | **BLOCKING** | Any multi-platform binary build |
| 2 | Cross-compile targets not added to stable toolchain (4 of 5 missing) | **BLOCKING** | Multi-platform binary build |

---

## Recommendations

### Immediate (before Phase 3 implementation)

1. **Add rustup cross-compile targets to stable:**
   ```bash
   rustup target add x86_64-apple-darwin x86_64-unknown-linux-musl \
     aarch64-unknown-linux-musl x86_64-pc-windows-gnu
   ```
   Note: Use `musl` variants (not `gnu`) for the `< 25MB` static binary requirement.

2. **Install `cross` (recommended — Docker is already available):**
   ```bash
   cargo install cross --locked
   ```

3. **Pin Rust toolchain via `rust-toolchain.toml`** in project root (not yet present):
   ```toml
   [toolchain]
   channel = "1.94.0"
   components = ["rustfmt", "clippy"]
   targets = [
     "aarch64-apple-darwin",
     "x86_64-apple-darwin",
     "x86_64-unknown-linux-musl",
     "aarch64-unknown-linux-musl",
     "x86_64-pc-windows-gnu"
   ]
   ```

4. **Resolve Homebrew/rustup PATH conflict:** Ensure `$HOME/.cargo/bin` comes before `/opt/homebrew/bin` in `$PATH`, or remove the Homebrew Rust installation to avoid version confusion. One authoritative `rustc` reduces confusion.

### Before Phase 5 (Formal Hardening)

5. **Install `cargo-kani`:**
   ```bash
   cargo install --locked kani-verifier
   cargo kani setup
   ```

6. **Install `cargo-mutants`:**
   ```bash
   cargo install --locked cargo-mutants
   ```

7. **Add `proptest` as dev-dependency in `Cargo.toml`** when it is created:
   ```toml
   [dev-dependencies]
   proptest = "1.10"
   ```

### Nice-to-Have

8. **Consider `cargo-zigbuild` as alternative to `cross`** if you want to avoid Docker dependency in CI:
   ```bash
   cargo install cargo-zigbuild --locked
   brew install zig
   ```

---

## Go / No-Go Assessment

| Phase | Status |
|-------|--------|
| Phase 1 (Spec Crystallization) | ✅ GO — No toolchain required |
| Phase 2 (Story Decomposition) | ✅ GO — No toolchain required |
| Phase 3 (Implementation / TDD) | ⚠️ **CONDITIONAL GO** — Core Rust toolchain ready; cross-compile blockers don't affect local `cargo test` / `cargo build` for host target. Fix cross-compile before final build. |
| Phase 5 (Formal Hardening) | ⚠️ **CONDITIONAL GO** — cargo-fuzz ready (nightly present); cargo-kani and cargo-mutants need installation. |

---

*Report generated by DX Engineer subagent. No tools were installed. This is a read-only preflight assessment.*
