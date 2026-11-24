<h1 align="center">TREECUT ☤</h1>

<p align="center">
  <strong>Local-first AST, SQLite-cached, and Git-diff context slicer for AI coding agents — dispatched through one binary.</strong>
</p>

<p align="center">
  <a href="LICENSE"><img alt="License: MIT OR Apache-2.0" src="https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-22c55e?style=for-the-badge"></a>
  <a href="Cargo.toml"><img alt="Version 0.1.0" src="https://img.shields.io/badge/Version-0.1.0-0ea5e9?style=for-the-badge"></a>
  <a href="Cargo.toml"><img alt="Rust 2024" src="https://img.shields.io/badge/Rust-2024-CE422B?style=for-the-badge&logo=rust&logoColor=white"></a>
  <a href="https://www.rust-lang.org/"><img alt="Platform Linux" src="https://img.shields.io/badge/Platform-Linux-FCC624?style=for-the-badge&logo=linux&logoColor=111827"></a>
</p>

<p align="center">
  <a href="https://github.com/bigwalkdoe/treecut/actions/workflows/ci.yml"><img alt="CI workflow" src="https://img.shields.io/github/actions/workflow/status/bigwalkdoe/treecut/ci.yml?branch=main&style=flat-square&label=ci"></a>
</p>

---

## Contents

- [▸ What treecut is](#-what-treecut-is)
- [✦ Feature map](#-feature-map)
- [⚡ Quick start](#-quick-start)
- [🔧 MCP tools](#-mcp-tools)
- [🏗 Architecture](#-architecture)
- [📜 License](#-license)

## ▸ What treecut is

treecut is a consolidated developer tool for **AST slicing**, **SQLite-cached repo maps**, and **Git-diff impact analysis**. Instead of dumping whole files into an LLM context, it extracts only the symbols that matter, plus their blast radius.

Runs in two modes over one shared core:

- **MCP server** (`treecut mcp`) — stdio JSON-RPC 2.0 for AI hosts (OpenCode, Cursor, Claude Code).
- **Agentic CLI** (`treecut ask` / `slice` / `map` / `diff-slice`) — deterministic extraction or a ReAct loop backed by any OpenAI-compatible endpoint.

## ✦ Feature map

| Area | What it covers |
|------|----------------|
| **Parsing** | Rust, Python, TypeScript/TSX, Go, Java, C/C++ via tree-sitter |
| **Caching** | Incremental SQLite cache keyed on `(mtime, size)` in `.treecut/cache.db` |
| **Slicing** | Symbol definitions + cross-file call sites with context lines |
| **Diff impact** | Modified symbols from git diff + impacted callers (blast radius) |
| **Traversal** | Gitignore-aware via the `ignore` crate |
| **Degradation** | Unparseable/binary files skipped with `tracing::debug!` |
| **MCP** | 4 tools over stdio JSON-RPC with strict stdout purity |
| **CLI** | `map`, `slice`, `diff-slice`, `ask` subcommands |

## ⚡ Quick start

```bash
cargo install --path .
treecut map . --depth 4
treecut slice my_func .
treecut diff-slice . --base HEAD~1
treecut mcp
```

### MCP config

OpenCode (`opencode.json`):

```json
{
  "mcpServers": {
    "treecut": { "command": "treecut", "args": ["mcp"] }
  }
}
```

Claude Code:

```json
{
  "mcpServers": {
    "treecut": { "command": "treecut", "args": ["mcp"] }
  }
}
```

## 🔧 MCP tools

| Tool | Description |
|------|-------------|
| `repo_skeleton` | Compressed repo map with top-level declarations and line numbers |
| `slice_symbol` | Full definitions + cross-file call sites for a symbol |
| `slice_changed_symbols` | Modified symbols from a git diff + impacted callers |
| `read_node` | Read a specific line range with line numbers |

## 🏗 Architecture

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

## 📜 License

MIT OR Apache-2.0
