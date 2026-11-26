# Outreach & Distribution

Ready-to-use copy for listing treecut in MCP directories and posting the demo.

## MCP Registry (done)

- **Status:** Published
- **Name:** `io.github.bigwalkdoe/treecut`
- **Version:** `0.4.3`
- **Registry:** https://registry.modelcontextprotocol.io

## Directory submissions (manual/web)

### Copy/paste listing

```text
treecut — local-first AST & call-graph context slicer for AI coding agents. Tree-sitter parsing, SQLite mtime cache, git-diff blast radius, transitive traversal (depth/direction), monorepo scope filtering, and token-savings stats. Dual MCP stdio server + CLI.

Repo: https://github.com/bigwalkdoe/treecut
Crates: https://crates.io/crates/treecut
Docs: https://docs.rs/treecut
Install: cargo install treecut

MCP config:
{
  "mcpServers": {
    "treecut": {
      "command": "treecut",
      "args": ["mcp"]
    }
  }
}
```

### Targets

- **Glama:** https://glama.ai/mcp/servers (web submission; create listing first)
- **Smithery:** https://smithery.ai/ (web submission/publish flow)
- **punkpeye/awesome-mcp-servers:** https://github.com/punkpeye/awesome-mcp-servers  
  - Note: CONTRIBUTING requires a Glama listing backlink before PR merge.
- **wong2/awesome-mcp-servers:** https://github.com/wong2/awesome-mcp-servers
- **TensorBlock/awesome-mcp-servers:** https://github.com/TensorBlock/awesome-mcp-servers

## Demo post (manual/web)

- **Asset:** `assets/reel.gif` (9:16, generated via `vhs reel.tape`)
- **Caption:**

```text
treecut v0.4.x — stop dumping whole files into agent context. AST symbol slicing + transitive call-graph traversal (--depth 2 --direction both) + git-diff blast radius + token-savings stats. Rust, MCP + CLI.

https://github.com/bigwalkdoe/treecut
https://crates.io/crates/treecut
```

- **Targets:** X, r/rust, r/LocalLLaMA, YouTube Shorts/Reels

## Release/ops checklist

- [x] v0.4.3 GitHub Release + cargo-dist artifacts green
- [x] crates.io published (0.4.3)
- [x] Homebrew tap updated (0.4.3)
- [x] MCP Registry published (io.github.bigwalkdoe/treecut@0.4.3)
- [ ] HOMEBREW_TAP_TOKEN verified on next tag (permissions fixed; rerun not done on 0.4.2 failure)
- [ ] Glama listing
- [ ] Smithery listing
- [ ] Awesome MCP list PR(s)
- [ ] Demo post(s)
