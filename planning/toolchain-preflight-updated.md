# Toolchain Preflight — Updated Status

**Updated:** 2026-03-29  
**Project:** forge-mcp  
**Platform:** Darwin arm64 (aarch64-apple-darwin)

---

## ✅ Resolved: Rust Version Ambiguity

A `rust-toolchain.toml` has been committed to the project root:

```toml
[toolchain]
channel = "stable"
components = ["rustfmt", "clippy", "rust-src"]
```

This pins all developers and CI to rustup's **stable** toolchain (currently
1.94.1 as of 2026-03-26), preventing accidental use of Homebrew's Rust.

---

## Tool Status

### Core Rust Toolchain

| Tool | Version | Status |
|------|---------|--------|
| rustc (rustup stable) | 1.94.1 (e408947bf 2026-03-25) | ✅ Installed |
| cargo (rustup stable) | 1.94.1 | ✅ Installed |
| rustfmt | 1.8.0 | ✅ Installed |
| clippy | 0.1.94 | ✅ Installed |
| rust-src component | — | ✅ Installed |

### Cross-Compile Targets (rustup stable)

| Target | Status |
|--------|--------|
| aarch64-apple-darwin | ✅ Installed (native) |
| x86_64-apple-darwin | ✅ Installed |
| x86_64-unknown-linux-musl | ✅ Installed |
| aarch64-unknown-linux-musl | ✅ Installed |
| x86_64-pc-windows-gnu | ✅ Installed |

### Build Tools

| Tool | Version | Status |
|------|---------|--------|
| cross | 0.2.5 | ✅ Installed (`cargo install cross --locked`) |

### Phase 5 Verification Tools

| Tool | Version | Status |
|------|---------|--------|
| cargo-kani (kani-verifier) | 0.67.0 | ✅ Installed + Setup Complete |
| kani nightly toolchain | 1.93.0-nightly (2025-11-21) | ✅ Installed by `cargo kani setup` |
| cargo-mutants | 27.0.0 | ✅ Installed (`cargo install cargo-mutants --locked`) |

### Previously Installed (from initial preflight)

| Tool | Status |
|------|--------|
| git | ✅ |
| gh CLI | ✅ |
| docker | ✅ |
| direnv | ✅ |
| just | ✅ |
| lefthook | ✅ |
| semgrep | ✅ |
| cargo-audit | ✅ |
| cargo-deny | ✅ |

---

## Previously Deferred — Now Resolved

| Item | Resolution |
|------|-----------|
| Homebrew vs rustup ambiguity | ✅ Fixed via `rust-toolchain.toml` |
| `cross` not installed | ✅ Installed v0.2.5 |
| Missing cross-compile targets | ✅ All 4 targets added |
| `cargo-kani` not installed | ✅ Installed v0.67.0 + setup complete |
| `cargo-mutants` not installed | ✅ Installed v27.0.0 |

---

## Still Missing / Deferred

| Tool | Reason |
|------|--------|
| `cargo-fuzz` | Not yet installed — needed for Phase 5 fuzzing. Install with: `cargo install cargo-fuzz` |
| VHS (demo recording) | CLI demo tool — Phase 6 evidence. Install with Homebrew when needed. |
| `mcporter` | MCP CLI bridge — install as ClawHub skill when needed. |

---

## Notes

- `rust-toolchain.toml` uses `channel = "stable"` — this resolves to the
  latest stable at install time. For reproducible CI, consider pinning to a
  specific version (e.g., `channel = "1.94.1"`) before Phase 3.
- `cross` v0.2.5 warns about missing package metadata when run outside a
  Cargo workspace — this is expected and harmless; it falls back to the host
  cargo for the check.
- Kani's required nightly toolchain (`nightly-2025-11-21`) is managed by
  rustup and isolated from the project's stable toolchain.
