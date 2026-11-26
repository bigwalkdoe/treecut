# Contributing to treecut

Thanks for contributing to `treecut`.

## Development setup

- Rust stable toolchain (edition 2024 project)
- Git (for diff-slice tests/features)

```bash
git clone https://github.com/bigwalkdoe/treecut.git
cd treecut
cargo test --locked
```

## Common checks

```bash
# Full test suite
cargo test --locked

# Lints (CI uses --all-targets --all-features)
cargo clippy --all-targets --all-features

# Format check
cargo fmt --check

# MCP-only build (no CLI/HTTP deps)
cargo check --locked --no-default-features --features mcp
```

## Project layout

- `src/main.rs` - CLI entrypoint and argument parsing
- `src/mcp.rs` - stdio JSON-RPC MCP server
- `src/core/` - pure core engine (walker, parser, slicer, skeleton, git diff, cache, models)
- `tests/core_tests.rs` - core/integration tests
- `tests/fixtures/` - language and behavior fixtures

## Validation expectations

When changing behavior, add or update tests that cover the affected path:

- **Parser/skeleton changes**: add fixture-based tests under `tests/fixtures/` and assertions in `tests/core_tests.rs`.
- **Slicer/call-graph changes**: cover definitions, call sites, alias resolution, depth/direction, scope/include, and dedup behavior.
- **Git diff behavior**: use temp git repos in tests (see existing `git2_diff_slice` pattern).
- **MCP/CLI changes**: add smoke/integration tests that invoke the binary (`CARGO_BIN_EXE_treecut`).

Before opening a PR, run at least:

```bash
cargo test --locked
cargo check --locked --no-default-features --features mcp
```

## Documentation

- Update `README.md` for user-facing CLI/MCP behavior.
- Update `CHANGELOG.md` for notable changes.
- Design/spec docs live in `docs/superpowers/specs/`.

## Pull requests

- Keep PRs focused and include tests for behavior changes.
- Avoid unrelated refactors in bugfix/feature PRs.
- Do not commit machine-specific local config (e.g. `opencode.json` is ignored).
