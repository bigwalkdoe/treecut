# treecut

[![CI](https://github.com/bigwalkdoe/treecut/actions/workflows/ci.yml/badge.svg)](https://github.com/bigwalkdoe/treecut/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-2024-orange.svg)](Cargo.toml)

**treecut is an open-source local-first AST and Git-diff context slicer for AI coding agents.**  
It compiles into a single binary that serves both an MCP server (stdio JSON-RPC) and an agentic CLI, sharing one pure-Rust core engine.

> **Active development.** treecut v0.1.0 is under active development. Core slicing, caching, and diff flows are functional; the ReAct agent loop and additional language coverage are expanding.

## Why treecut

Most AI coding tools dump entire files into context. treecut extracts only what matters:

- **Symbol-level slicing** — definitions + cross-file call sites, not whole files.
- **Git-diff impact slicing** — modified symbols plus their blast radius (callers).
- **Incremental SQLite cache** — keyed on `(mtime, size)` so unchanged files are never re-parsed.
- **Gitignore-aware traversal** — respects `.gitignore`, `.ignore`, and hidden dirs.
- **Graceful degradation** — unparseable/binary files are skipped, never crashing a scan.

## What's in the box

| Area | State |
|------|-------|
| Multi-language AST (Rust, Python, TS/TSX, Go, Java, C/C++) | ✅ |
| SQLite mtime/size cache (`.treecut/cache.db`) | ✅ |
| Git-diff modified symbols + callers | ✅ |
| MCP stdio server (4 tools) | ✅ |
| Agentic CLI (`map`, `slice`, `diff-slice`, `ask`) | ✅ |
| ReAct tool-calling agent loop | 🚧 |

## Quick start

```bash
cargo install --path .
treecut map . --depth 4
treecut slice my_func .
treecut diff-slice . --base HEAD~1
treecut mcp
```

## MCP tools

| Tool | Description |
|------|-------------|
| `repo_skeleton` | Compressed repo map with top-level declarations and line numbers |
| `slice_symbol` | Full definitions + cross-file call sites for a symbol |
| `slice_changed_symbols` | Modified symbols from a git diff + impacted callers |
| `read_node` | Read a specific line range with line numbers |

## Architecture

```text
src/
├── main.rs          # CLI router & stderr tracing
├── lib.rs           # Re-exports core
├── core/            # Pure-Rust engine (no UI/network coupling)
│   ├── models.rs    # Shared tool schemas (JsonSchema)
│   ├── cache.rs     # SQLite mtime/size cache
│   ├── walker.rs    # Gitignore-aware discovery
│   ├── parser.rs    # Tree-sitter extractors
│   ├── slicer.rs    # Symbol + dependency slicer
│   ├── skeleton.rs  # Repo map generator
│   └── git_diff.rs  # Diff -> modified symbols + blast radius
├── mcp.rs           # stdio JSON-RPC server
└── agent.rs         # ReAct CLI agent loop
```

## License

MIT OR Apache-2.0
