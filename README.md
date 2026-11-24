# treecut

**Local-first AST, SQLite-cached, and Git-diff context slicer for AI coding agents.**

`treecut` is a single Rust binary that extracts precise, dependency-aware code context for LLMs. It compiles into two modes over one shared core engine:

1. **MCP Server** (`treecut mcp`) — exposes tools over stdio JSON-RPC 2.0 for AI hosts (OpenCode, Cursor, Claude Code).
2. **Agentic CLI** (`treecut ask` / `slice` / `map` / `diff-slice`) — deterministic extraction commands or a ReAct terminal agent backed by any OpenAI-compatible endpoint (Ollama, OpenRouter).

## Features

- Multi-language AST parsing: Rust, Python, TypeScript/TSX, Go, Java, C/C++
- Incremental SQLite cache keyed on `(mtime, size)` — skips re-parsing unchanged files
- Git-diff impact slicing: modified symbols + blast radius (callers)
- Gitignore-aware traversal via the `ignore` crate
- Graceful degradation on unparseable/binary files
- Strict stdout purity in MCP mode (logs go to stderr)

## Install

```bash
cargo install --path .
```

## Usage

```bash
# MCP server (stdio)
treecut mcp

# Repository skeleton
treecut map . --depth 4 --max-chars 12000

# Slice a symbol
treecut slice my_func .

# Diff-aware slice (pre-commit review)
treecut diff-slice . --base HEAD~1

# Agentic ask
treecut ask "What does add() do?" --path .
```

## MCP Tools

| Tool | Description |
|------|-------------|
| `repo_skeleton` | Compressed repo map with top-level declarations and line numbers |
| `slice_symbol` | Full definitions + cross-file call sites for a symbol |
| `slice_changed_symbols` | Modified symbols from git diff + impacted callers |
| `read_node` | Read a specific line range with line numbers |

## Architecture

```
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
├── mcp.rs           # rmcp stdio server
└── agent.rs         # ReAct CLI agent loop
```

## License

MIT OR Apache-2.0
