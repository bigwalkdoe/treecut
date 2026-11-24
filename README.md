<div align="center">

# `treecut`

**Precision AST & Call-Graph Context Slicing for AI Coding Agents**

[![Crates.io](https://img.shields.io/crates/v/treecut.svg)](https://crates.io/crates/treecut)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Built With Rust](https://img.shields.io/badge/built_with-Rust-CE422B.svg)](https://rust-lang.org)
[![Tree-sitter](https://img.shields.io/badge/tree--sitter-12_languages-2ea44f.svg)](https://tree-sitter.github.io)
[![MCP Compatible](https://img.shields.io/badge/MCP-compatible-8A2BE2.svg)](https://modelcontextprotocol.io)
[![SQLite Cached](https://img.shields.io/badge/cache-SQLite_mtime-003B57.svg)](https://sqlite.org)

Stop dumping 1,500-line files into your LLM's context window.<br/>
`treecut` parses your codebase in milliseconds and extracts only the exact definitions, callers, and Git-diff blast radius your agent needs.

[Quickstart](#quickstart) • [MCP Setup](#mcp-server-setup) • [CLI Usage](#cli-usage) • [Architecture](#architecture)

</div>

<div align="center">
  <img src="assets/demo.gif" alt="treecut CLI & AST slicing demo" width="850" />
</div>

---

## Why `treecut`?

AI coding agents lose accuracy and burn tokens when forced to `grep` blindly or read entire files just to trace one function. **`treecut`** solves context bloat with a single, zero-config Rust binary:

* **Surgical Symbol Slicing (`slice_symbol`)**: Extracts a target function, struct, or class definition alongside every cross-file call site and enclosing signature.
* **Git-Diff Blast Radius (`slice_changed_symbols`)**: Intersects `git diff` hunks with parsed AST spans to isolate modified symbols and automatically pull in their callers.
* **Compressed Repo Maps (`repo_skeleton`)**: Generates token-budgeted architectural outlines with function bodies stripped out.
* **Incremental SQLite `mtime` Cache**: Indexes unchanged files in `.treecut/cache.db` so repeated slices across large repositories run in single-digit milliseconds.
* **12 Languages Out of the Box**: Native `tree-sitter` support for **Rust**, **Python**, **TypeScript / TSX**, **Go**, **Java**, **C / C++**, **JavaScript / JSX**, **C#**, **Ruby**, **PHP**, **Swift**, and **Bash / Shell**.
* **Dual-Mode Binary**: Runs as a headless `stdio` **MCP Server** inside your editor or as a standalone **Agentic CLI** in your terminal.

---

## Quickstart

### Install via Cargo

```bash
cargo install treecut
```

### Build from Source

```bash
git clone https://github.com/bigwalkdoe/treecut.git
cd treecut
cargo install --path .
```

> **Minimal MCP-only build** (excludes HTTP/LLM client dependencies):
> ```bash
> cargo install --path . --no-default-features --features mcp
> ```

---

## MCP Server Setup

`treecut mcp` communicates over standard `stdio` JSON-RPC 2.0 with zero background daemons or API keys required.

### OpenCode (`opencode.json`)

```json
{
  "$schema": "https://opencode.ai",
  "mcp": {
    "treecut": {
      "type": "local",
      "command": ["treecut", "mcp"],
      "enabled": true
    }
  }
}
```

### Claude Code (`~/.claude.json` or `.mcp.json`)

```json
{
  "mcpServers": {
    "treecut": {
      "command": "treecut",
      "args": ["mcp"]
    }
  }
}
```

### Cursor (`.cursor/mcp.json`)

```json
{
  "mcpServers": {
    "treecut": {
      "command": "treecut",
      "args": ["mcp"]
    }
  }
}
```

### Exposed MCP Tools

| Tool | Purpose | Key Arguments |
| :--- | :--- | :--- |
| `repo_skeleton` | Compressed codebase signature map | `root`, `max_depth`, `max_chars` |
| `slice_symbol` | Full definition + cross-file call sites | `symbol`, `root`, `context_lines`, `max_chars` |
| `slice_changed_symbols` | Modified AST symbols in Git diff + callers | `root`, `base_ref`, `include_callers`, `max_chars` |
| `read_node` | Line-bounded precision source reader | `path`, `start_line`, `end_line` |

---

## CLI Usage

Use `treecut` directly in terminal pipelines, Git hooks, or standalone agentic debugging sessions.

### Deterministic Context Commands (No LLM Required)

```bash
# Generate a compressed repository skeleton (up to 12k chars)
treecut map --depth 3 --max-chars 12000

# Slice a symbol's definition and all call sites across the repo
treecut slice verify_token

# Extract only functions modified in uncommitted changes + their callers
treecut diff-slice

# Compare against a base branch before opening a PR
treecut diff-slice --base main
```

### Agentic Terminal Mode (`treecut ask`)

`treecut ask` runs a multi-turn ReAct tool loop using any OpenAI-compatible endpoint (defaults to local **Ollama** at `http://localhost:11434/v1`).

```bash
# Ask architectural questions backed by live AST slicing
treecut ask "trace how session tokens are validated and where they expire"

# Pipe compiler or test errors directly into the agent
cargo test 2>&1 | treecut ask "diagnose this failure using the minimal call chain"

# Use OpenRouter or a custom model
LLM_API_KEY="sk-or-..." treecut ask \
  --base-url "https://openrouter.ai" \
  --model "anthropic/claude-sonnet-4" \
  "review my current git diff for edge cases"
```

---

## Architecture

`treecut` is structured around a pure, UI-agnostic Rust core shared identically by both interface layers:

```text
                 ┌──────────────────────────┐
                 │     treecut (Binary)     │
                 └────────────┬─────────────┘
                              │
               ┌──────────────┴──────────────┐
               ▼                             ▼
   ┌───────────────────────┐     ┌───────────────────────┐
   │   MCP Server (rmcp)   │     │  Agentic CLI (clap)   │
   │  JSON-RPC over stdio  │     │ ReAct Loop & Pipes    │
   └───────────┬───────────┘     └───────────┬───────────┘
               └──────────────┬──────────────┘
                              ▼
┌──────────────────────────────────────────────────────────┐
│                     Pure Core Engine                     │
│  • ignore Walker      • Tree-sitter (12 Languages)       │
│  • git2 Diff Mapper   • SQLite mtime Cache (.treecut/)   │
└──────────────────────────────────────────────────────────┘
```

* **Zero `stdout` Pollution**: All diagnostics and `tracing` logs route strictly to `stderr`, guaranteeing clean JSON-RPC framing in MCP mode.
* **Automatic Cache Hygiene**: Creates `.treecut/cache.db` alongside a local `.treecut/.gitignore` (`*`) so your working tree stays clean.

---

## Contributing

Contributions are welcome—especially new `tree-sitter` language queries, benchmark suites, and edge-case fixtures.

```bash
# Run the full test suite (AST parsers, SQLite cache invalidation, git2 diffs, & MCP stdio handshake)
cargo test
```

## License

Dual-licensed under either of:

* **Apache License, Version 2.0** ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org)
* **MIT License** ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org)

at your option.
