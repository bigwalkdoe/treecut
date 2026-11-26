# Development & Validation

This document describes how to validate `treecut` locally and what CI enforces.

## Local validation

```bash
# Default test suite (same as CI)
cargo test --locked

# MCP-only feature build (same as CI)
cargo check --locked --no-default-features --features mcp

# Lints (CI runs with -Dwarnings)
cargo clippy --all-targets --all-features

# Formatting
cargo fmt --check
```

## What tests cover

`tests/core_tests.rs` includes:

- **Cache invalidation**: SQLite mtime cache behavior (`sqlite_mtime_cache_invalidation`)
- **Git diff slicing**: changed symbol detection in temp git repos (`git2_diff_slice`, `diff_slice_stats`)
- **MCP stdio**: initialize/tools-list handshake and package version reporting (`jsonrpc_stdio_handshake`, `mcp_initialize_reports_package_version`)
- **CLI smoke**: `--help` and `--version` (`cli_help_and_version_smoke`)
- **Language skeleton fixtures**: JavaScript, C#, Ruby, PHP, Swift, Bash
- **Symbol slicing**: JS/C# symbol slices, alias resolution (Rust/TS/Python), batch slicing, dedup, stats
- **v0.4.0 traversal/filtering**: depth/direction, scope/include

## CI

GitHub Actions (`.github/workflows/ci.yml`) runs on push/PR to `main` and tags `v*`:

- **check** (ubuntu/macos/windows):
  - `cargo clippy --all-targets --all-features`
  - `cargo test --locked`
  - `cargo check --locked --no-default-features --features mcp`
- **build-release** (on version tags): builds release binaries for Linux/macOS/Windows targets and uploads artifacts
- **publish-release** (on version tags): publishes GitHub Release with generated notes and attached artifacts

> Note: release notes on GitHub may also be curated manually; keep `CHANGELOG.md` as the human-maintained source of notable changes.

## Adding a new language or behavior

1. Add/update parser support in `src/core/parser.rs`.
2. Add fixtures under `tests/fixtures/<lang>/`.
3. Add skeleton/slice tests in `tests/core_tests.rs`.
4. Update `README.md` language/support tables and examples.
5. Run local validation commands above.
